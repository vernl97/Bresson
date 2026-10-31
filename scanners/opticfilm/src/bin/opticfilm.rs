// SPDX-License-Identifier: GPL-3.0-or-later
//! `opticfilm` — scan from a Plustek OpticFilm scanner (or its mock).

use std::path::PathBuf;
use std::process::ExitCode;

use opticfilm::device::{known_models, model_by_name};
use opticfilm::image::save_gray16_tiff;
use opticfilm::scan::me_exposure::MeLongMode;
use opticfilm::usb::device::list_devices;
use opticfilm::{Error, ScanControl, ScanMethod, ScanMode, ScanRequest, Scanner};

const USAGE: &str = "\
usage: opticfilm <command> [options]

commands:
  list                      USB scanners on the bus (does not open them)
  models                    every model the driver knows, and its support level
  status                    open the scanner and print its status register
  scan                      scan one frame to a 16-bit TIFF
  calibrate                 measure (and cache) shading for a resolution
  buttons                   print QuickScan / IntelliScan presses (GL128)

options:
  --device ID               device id from `list` (default: the first scan-ready one)
  --fake MODEL              use the mock scanner for MODEL (name or suffix, e.g. 8200i-se)
  --dpi N                   resolution (default 1800)
  --mode color|infrared     scan mode (default color)
  --ir                      colour + infrared dust plane
  --me                      multi-exposure (adaptive long pass)
  --me-fixed                multi-exposure with the model's fixed long pass
  --passes N                Multi-Pass repeats (GL128, 1-9)
  --exposure N              manual single-pass REG_EXPOSURE (debug)
  --area X1,Y1,X2,Y2        normalized crop over the film window
  --no-calib                skip shading calibration
  --out PATH                output TIFF (default scan.tif; IR goes to PATH_IR.tif)
  -v                        log to stderr
";

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, r: &log::Record) {
        eprintln!("{} {}: {}", r.level(), r.target(), r.args());
    }
    fn flush(&self) {}
}

#[derive(Default)]
struct Args {
    cmd: String,
    device: Option<String>,
    fake: Option<String>,
    req: ScanRequest,
    out: Option<PathBuf>,
}

fn parse() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let mut a = Args { cmd: it.next().ok_or("missing command")?, ..Default::default() };
    let val = |it: &mut dyn Iterator<Item = String>, flag: &str| it.next().ok_or(format!("{flag} needs a value"));
    while let Some(f) = it.next() {
        match f.as_str() {
            "--device" => a.device = Some(val(&mut it, &f)?),
            "--fake" => a.fake = Some(val(&mut it, &f)?),
            "--dpi" => a.req.resolution = val(&mut it, &f)?.parse().map_err(|_| "bad --dpi")?,
            "--mode" => a.req.mode = ScanMode::parse(&val(&mut it, &f)?).map_err(|e| e.to_string())?,
            "--ir" => a.req.infrared = true,
            "--me" => a.req.multi_exposure = true,
            "--me-fixed" => {
                a.req.multi_exposure = true;
                a.req.me_long_mode = MeLongMode::Fixed;
            }
            "--passes" => a.req.n_passes = val(&mut it, &f)?.parse().map_err(|_| "bad --passes")?,
            "--exposure" => a.req.single_pass_exposure = Some(val(&mut it, &f)?.parse().map_err(|_| "bad --exposure")?),
            "--area" => {
                let v: Vec<f64> = val(&mut it, &f)?.split(',').map(|s| s.trim().parse::<f64>()).collect::<Result<_, _>>().map_err(|_| "bad --area")?;
                let [x1, y1, x2, y2] = v[..] else { return Err("--area needs four numbers".into()) };
                a.req.area = Some([x1, y1, x2, y2]);
            }
            "--no-calib" => a.req.apply_calib = false,
            "--out" => a.out = Some(PathBuf::from(val(&mut it, &f)?)),
            "-v" => {
                let _ = log::set_logger(&Logger);
                log::set_max_level(log::LevelFilter::Info);
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(a)
}

fn support(scan_ready: bool, name: &str) -> &'static str {
    match (scan_ready, name) {
        (true, _) => "hardware tested",
        (false, "plustek-opticfilm-8200i") => "protocol validated (scan locked)",
        _ => "experimental (scan locked)",
    }
}

fn open(a: &Args) -> Result<Scanner, Error> {
    match &a.fake {
        Some(name) => {
            let model = model_by_name(name)
                .or_else(|| known_models().iter().find(|m| m.name.ends_with(&format!("-{name}"))).cloned())
                .ok_or_else(|| Error::Value(format!("unknown model {name:?} (see `opticfilm models`)")))?;
            let cache = std::env::temp_dir().join(format!("opticfilm-fake-{}.json", std::process::id()));
            Scanner::open_fake(model, None, Some(cache))
        }
        None => Scanner::open(a.device.as_deref(), None),
    }
}

fn run(a: Args) -> Result<(), Error> {
    if a.fake.is_some() {
        // The mock has no real motor or lamp: skip their settle waits.
        opticfilm::clock::use_virtual(0.0);
    }
    match a.cmd.as_str() {
        "list" => {
            let devs = list_devices()?;
            if devs.is_empty() {
                println!("no Plustek scanners found");
            }
            for d in devs {
                let model = opticfilm::device::model_for_device(d.product_id, d.bcd_device).map(|m| m.model.to_string()).unwrap_or_else(|_| "unknown".into());
                println!("{}  {}  bcd={:04x}  {}", d.device_id(), model, d.bcd_device, d.asic_hint());
            }
        }
        "models" => {
            for m in known_models() {
                println!("{:30} {:05} {:04x}:{:04x}  {}", m.name, m.asic, m.usb_vendor_id, m.usb_product_id, support(m.scan_ready, m.name));
            }
        }
        "status" => {
            let mut s = open(&a)?;
            println!("{} ({})", s.model().model, s.device_id());
            println!("{:?}", s.status()?);
        }
        "scan" => {
            let mut s = open(&a)?;
            s.warmup(true, true)?;
            let mut last = -1i64;
            let mut ctl = ScanControl {
                progress: Some(Box::new(move |p: f64| {
                    let pct = (p * 100.0) as i64;
                    if pct / 5 != last / 5 {
                        last = pct;
                        eprint!("\r{pct:3}%");
                    }
                })),
                ..Default::default()
            };
            let img = s.scan(&a.req, &mut ctl)?;
            eprintln!();
            let out = a.out.clone().unwrap_or_else(|| PathBuf::from("scan.tif"));
            let path = img.save_tiff(&out)?;
            println!("{} ({}x{} px, {} dpi)", path.display(), img.rgb.dim().1, img.rgb.dim().0, img.dpi);
            if let Some(ir) = &img.ir {
                let stem = path.with_extension("");
                let ir_path = save_gray16_tiff(ir, format!("{}_IR.tif", stem.display()), img.dpi)?;
                println!("{}{}", ir_path.display(), if img.ir_tracks_image() == Some(true) { " (follows the picture: not a dust map)" } else { "" });
            }
        }
        "calibrate" => {
            let mut s = open(&a)?;
            let e = s.calibrate(a.req.resolution, a.req.mode, true, a.req.area, None)?;
            println!(
                "{} {} dpi: {} px, {}",
                e.method.as_str(),
                e.resolution,
                e.pixels,
                if e.asic_shading { "ASIC shading table" } else { "host dark/white" }
            );
        }
        "buttons" => {
            let mut s = open(&a)?;
            s.lamp_on(ScanMethod::Transparency).ok();
            eprintln!("press a scanner button (Ctrl-C to stop)");
            loop {
                for b in s.button_state(Some(250))? {
                    println!("{}", b.as_str());
                }
            }
        }
        other => return Err(Error::Value(format!("unknown command {other:?}\n\n{USAGE}"))),
    }
    Ok(())
}

fn main() -> ExitCode {
    let a = match parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match run(a) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
