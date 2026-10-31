// SPDX-License-Identifier: GPL-3.0-or-later
//! Film-scan negotiation and the read loop, against the simulated flatbed;
//! `libsane_test_backend` runs against the real libsane where it's installed.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use sane::fake::{DEVICE_NAME, FakeLog};
use sane::{Device, Error, FakeConfig, FakeSane, FilmScan, Sane, Value};

/// The simulated device through [`Sane`], plus its log.
fn device(config: FakeConfig) -> (Device, Arc<Mutex<FakeLog>>) {
    let fake = FakeSane::new(config);
    let log = fake.log();
    let mut sane = Sane::from_backend(Box::new(fake));
    (sane.open(DEVICE_NAME).unwrap(), log)
}

fn job(dpi: u32, area: [f64; 4]) -> FilmScan {
    FilmScan {
        source: None,
        dpi,
        area,
    }
}

fn scan(dev: &mut Device, j: &FilmScan) -> sane::SaneImage {
    dev.scan_film(j, &AtomicBool::new(false), &mut |_| {})
        .unwrap()
}

#[test]
fn lists_the_simulated_device() {
    let mut sane = Sane::simulated(FakeConfig::default());
    let d = sane.devices().unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(
        (d[0].name.as_str(), d[0].model.as_str()),
        (DEVICE_NAME, "Simulated flatbed")
    );
    assert!(sane.open("other:device").is_err());
}

#[test]
fn film_setup_picks_the_transparency_unit_and_neutral_options() {
    let (mut dev, log) = device(FakeConfig::default());
    assert_eq!(dev.film_source().as_deref(), Some("Transparency Unit"));
    // Before: flatbed area, film type inactive.
    assert!(!dev.option("film-type").unwrap().active());
    dev.prepare_film(None).unwrap();
    assert_eq!(
        dev.get("source").unwrap(),
        Value::Str("Transparency Unit".into())
    );
    assert_eq!(dev.get("mode").unwrap(), Value::Str("Color".into()));
    assert_eq!(dev.get("depth").unwrap(), Value::Int(16));
    assert_eq!(
        dev.get("film-type").unwrap(),
        Value::Str("Positive Film".into())
    );
    assert_eq!(dev.get("custom-gamma").unwrap(), Value::Bool(false));
    // The source change reloaded the options: the area is now the unit's, film type active.
    assert_eq!(log.lock().unwrap().reloads, 1);
    assert!(dev.option("film-type").unwrap().active());
    let a = dev.area_mm().unwrap();
    assert!(
        (a[2] - 63.5).abs() < 1e-3 && (a[3] - 228.6).abs() < 1e-3,
        "{a:?}"
    );
    // An explicit source wins over the film guess.
    dev.prepare_film(Some("Flatbed")).unwrap();
    assert_eq!(dev.get("source").unwrap(), Value::Str("Flatbed".into()));
}

#[test]
fn resolutions_and_sources_come_from_the_device() {
    let (dev, _) = device(FakeConfig::default());
    assert_eq!(
        dev.resolutions(),
        [75, 100, 150, 300, 600, 1200, 2400, 3200, 6400]
    );
    assert_eq!(dev.sources(), ["Flatbed", "Transparency Unit"]);
}

fn mm(dev: &mut Device, name: &str) -> f64 {
    match dev.get(name).unwrap() {
        Value::Fixed(v) => v,
        v => panic!("{v:?}"),
    }
}

#[test]
fn normalised_area_maps_to_millimetres() {
    let (mut dev, _) = device(FakeConfig::default());
    dev.prepare_film(None).unwrap();
    dev.set_window(300, [0.25, 0.5, 0.75, 1.0]).unwrap();
    assert!(
        (mm(&mut dev, "tl-x") - 15.875).abs() < 1e-3
            && (mm(&mut dev, "br-x") - 47.625).abs() < 1e-3
    );
    assert!(
        (mm(&mut dev, "tl-y") - 114.3).abs() < 1e-3 && (mm(&mut dev, "br-y") - 228.6).abs() < 1e-3
    );
    // Reversed corners are put in order.
    dev.set_window(300, [0.75, 1.0, 0.25, 0.5]).unwrap();
    assert!(
        (mm(&mut dev, "tl-x") - 15.875).abs() < 1e-3 && (mm(&mut dev, "br-y") - 228.6).abs() < 1e-3
    );
}

/// The frame area of the first frame on the strip, normalised to the unit.
const FRAME1: [f64; 4] = [19.75 / 63.5, 10.0 / 228.6, 43.75 / 63.5, 46.0 / 228.6];

#[test]
fn film_scan_reads_linear_positive_16_bit_rgb() {
    let (mut dev, log) = device(FakeConfig::default());
    let img = scan(&mut dev, &job(300, FRAME1));
    // 24 × 36 mm at 300 dpi.
    assert_eq!((img.width, img.height, img.dpi), (283, 425, 300));
    assert_eq!(img.rgb.len(), img.width * img.height * 3);
    // Linear and not inverted: the negative is dense (dark) where the scene was bright,
    // and red carries the orange mask: R > G > B everywhere.
    let px = |x: usize, y: usize| &img.rgb[(y * img.width + x) * 3..][..3];
    let p = px(10, 200);
    assert!(p[0] > p[1] && p[1] > p[2], "{p:?}");
    let (left, right) = (px(5, 400)[1], px(275, 400)[1]);
    assert!(
        left > right * 4,
        "falloff across the frame: {left} vs {right}"
    );
    let l = log.lock().unwrap();
    assert_eq!(l.starts, 1);
    assert_eq!(l.cancels, 1, "a scan ends with sane_cancel");
}

#[test]
fn three_pass_gray_8_bit_and_padding_decode_alike() {
    let reference = scan(&mut device(FakeConfig::default()).0, &job(150, FRAME1));
    let three = scan(
        &mut device(FakeConfig {
            three_pass: true,
            max_read: 1000,
            ..Default::default()
        })
        .0,
        &job(150, FRAME1),
    );
    assert_eq!(three, reference);
    let padded = scan(
        &mut device(FakeConfig {
            line_padding: 6,
            unknown_lines: true,
            ..Default::default()
        })
        .0,
        &job(150, FRAME1),
    );
    assert_eq!(padded, reference);

    // 8-bit: samples expand as v << 8 | v, within one 8-bit step of the 16-bit scan.
    let (mut dev, _) = device(FakeConfig::default());
    dev.prepare_film(None).unwrap();
    dev.set("depth", Value::Int(8)).unwrap();
    dev.set_window(150, FRAME1).unwrap();
    let img8 = dev.scan(&AtomicBool::new(false), &mut |_| {}).unwrap();
    assert_eq!(
        (img8.width, img8.height),
        (reference.width, reference.height)
    );
    assert!(img8.rgb.iter().all(|v| v >> 8 == v & 0xFF));
    assert!(
        img8.rgb
            .iter()
            .zip(&reference.rgb)
            .all(|(a, b)| (*a as i32 - *b as i32).abs() <= 257)
    );

    // Gray: one channel replicated.
    let (mut dev, _) = device(FakeConfig::default());
    dev.prepare_film(None).unwrap();
    dev.set("mode", Value::Str("Gray".into())).unwrap();
    dev.set_window(150, FRAME1).unwrap();
    let g = dev.scan(&AtomicBool::new(false), &mut |_| {}).unwrap();
    assert!(g.rgb.chunks_exact(3).all(|p| p[0] == p[1] && p[1] == p[2]));
    assert!(
        g.rgb
            .chunks_exact(3)
            .zip(reference.rgb.chunks_exact(3))
            .all(|(a, b)| a[1] == b[1])
    );
}

#[test]
fn progress_rises_to_one() {
    let (mut dev, _) = device(FakeConfig {
        three_pass: true,
        max_read: 4096,
        ..Default::default()
    });
    let mut seen = Vec::new();
    dev.scan_film(&job(150, FRAME1), &AtomicBool::new(false), &mut |p| {
        seen.push(p)
    })
    .unwrap();
    assert!(seen.len() > 3);
    assert!(seen.windows(2).all(|w| w[0] <= w[1] + 1e-9), "{seen:?}");
    assert_eq!(seen.last().copied(), Some(1.0));
    assert!(
        seen.iter().any(|&p| (0.3..0.4).contains(&p)),
        "three-pass progress spans the frames"
    );
}

#[test]
fn cancel_mid_read_stops_and_cancels_the_scan() {
    let (mut dev, log) = device(FakeConfig {
        max_read: 4096,
        ..Default::default()
    });
    let cancel = AtomicBool::new(false);
    let r = dev.scan_film(&job(300, FRAME1), &cancel, &mut |p| {
        if p > 0.2 {
            cancel.store(true, Ordering::SeqCst);
        }
    });
    assert!(matches!(r, Err(Error::Cancelled)), "{r:?}");
    assert_eq!(log.lock().unwrap().cancels, 1);
    // The device is usable again.
    cancel.store(false, Ordering::SeqCst);
    assert!(
        dev.scan_film(&job(150, FRAME1), &cancel, &mut |_| {})
            .is_ok()
    );
}

#[test]
fn lineart_is_refused_with_a_clear_message() {
    let (mut dev, _) = device(FakeConfig::default());
    let err = dev.scan(&AtomicBool::new(false), &mut |_| {}).unwrap_err();
    assert!(err.to_string().contains("1-bit"), "{err}");
}

/// Against the real libsane's `test` backend (`brew install sane-backends`, or
/// the distribution package). The test points SANE at a private config that
/// enables only that backend, so the system's `dll.conf` is left alone. Run
/// with `cargo test -p sane --release -- --ignored`.
#[test]
#[ignore]
fn libsane_test_backend() {
    let conf = std::env::temp_dir().join(format!("nlp-sane-conf-{}", std::process::id()));
    std::fs::create_dir_all(&conf).unwrap();
    std::fs::write(conf.join("dll.conf"), "test\n").unwrap();
    // SAFETY: set before libsane is loaded, and this ignored test runs alone.
    unsafe { std::env::set_var("SANE_CONFIG_DIR", &conf) };

    let mut sane = Sane::load().expect("libsane");
    let devices = sane.devices().unwrap();
    let name = devices
        .iter()
        .find(|d| d.name.starts_with("test"))
        .map(|d| d.name.clone())
        .expect("test backend device");
    let mut dev = sane.open(&name).unwrap();

    // Film setup on a real backend: colour, 16 bit; a window in mm from the normalised area.
    let job = FilmScan {
        source: None,
        dpi: 75,
        area: [0.0, 0.0, 0.5, 0.5],
    };
    let img = dev
        .scan_film(&job, &AtomicBool::new(false), &mut |_| {})
        .unwrap();
    assert_eq!(dev.get("mode").unwrap(), Value::Str("Color".into()));
    assert_eq!(dev.get("depth").unwrap(), Value::Int(16));
    assert!(
        img.width > 0 && img.height > 0,
        "{}×{}",
        img.width,
        img.height
    );
    assert_eq!(img.rgb.len(), img.width * img.height * 3);
    assert!(dev.resolutions().contains(&75));

    // SANE's own three-pass mode (separate R, G and B frames) assembles to the same size.
    dev.set("three-pass", Value::Bool(true)).unwrap();
    let three = dev.scan(&AtomicBool::new(false), &mut |_| {}).unwrap();
    assert_eq!((three.width, three.height), (img.width, img.height));
    assert_eq!(three.rgb.len(), img.rgb.len());

    // Cancel stops a real scan.
    dev.set("three-pass", Value::Bool(false)).unwrap();
    let cancel = AtomicBool::new(true);
    assert!(matches!(
        dev.scan(&cancel, &mut |_| {}),
        Err(Error::Cancelled)
    ));
    let _ = std::fs::remove_dir_all(&conf);
}
