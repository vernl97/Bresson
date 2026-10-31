// SPDX-License-Identifier: GPL-3.0-or-later
//! Port of the pure-function parts of `tests/test_asic_shading.py`:
//! shading table packing, gain law, validators, AFE gain/offset maths.

mod common;

use opticfilm::device::model_by_name;
use opticfilm::scan::bringup::{GeometryProfile, bringup_scan_geometry, preview_safe_scan_area};
use opticfilm::scan::calib_gl128::*;
use opticfilm::scan::calibrate::colour_shading_failure_message;
use opticfilm::scan::geometry::compute_geometry;

#[test]
fn colour_shading_failure_message_mapping() {
    assert!(colour_shading_failure_message("dark mean 12000 > 3000").contains("AFE black level"));
    assert!(
        colour_shading_failure_message(
            "ch0 white≈dark (w=100 d=200); lamp-off dark still bright (dark mean 22000)"
        )
        .contains("illumination")
    );
    assert!(
        colour_shading_failure_message("white strip looks like film (not clear home field)")
            .contains("clear home field")
    );
    assert!(colour_shading_failure_message("carriage not at home").contains("park at home"));
    assert!(colour_shading_failure_message("ch0 range 10 < 500").contains("power-cycle"));
}

#[test]
fn pick_shading_dark_layout_keeps_equal_mean_and_flips_when_truly_dark() {
    let primary = [[1000, 2000, 3000], [30000, 20000, 10000]];
    let alternate = [[7000, 7000, 7000], [15000, 15000, 15000]];
    let (chosen, planar) = pick_shading_dark_layout(
        &primary,
        &alternate,
        false,
        COLOR_SHADING_DARK_MEAN_MAX,
        COLOR_SHADING_LAYOUT_MEAN_IMPROVE_MIN,
    );
    assert!(!planar);
    assert_eq!(chosen[0], [1000, 2000, 3000]);
    let primary = [[20000, 21000, 19000]; 4];
    let alternate = [[800, 900, 850]; 4];
    let (chosen, planar) = pick_shading_dark_layout(
        &primary,
        &alternate,
        false,
        COLOR_SHADING_DARK_MEAN_MAX,
        COLOR_SHADING_LAYOUT_MEAN_IMPROVE_MIN,
    );
    assert!(planar);
    assert_eq!(chosen[0], [800, 900, 850]);
}

#[test]
fn shading_settle_constants_match_silverfast() {
    assert_eq!(COLOR_SHADING_DARK_SETTLE_S, 0.5);
    assert_eq!(IR_SHADING_DARK_SETTLE_S, 0.5);
    assert_eq!(COLOR_SHADING_DARK_MEAN_MAX, 3000.0);
    assert_eq!(IR_SHADING_WHITE_MEAN_MIN, 10000.0);
    assert_eq!(COLOR_SHADING_WHITE_MEAN_MIN, 20000.0);
}

#[test]
fn shading_table_covers_the_full_image_window() {
    let se = model_by_name("plustek-opticfilm-8200i-se").unwrap();
    for dpi in [1800, 3600] {
        let (area, _) = preview_safe_scan_area(&se, dpi, 0.0);
        let raw = compute_geometry(dpi, &se, Some(area)).unwrap();
        let raw_n = shading_acquire_width(
            raw.pixel_startx,
            raw.pixel_endx,
            raw.register_dpiset,
            se.optical_resolution,
        )
        .unwrap();
        assert_eq!(
            shading_entry_count(declared_shading_size(raw_n as usize)).unwrap() as i64,
            raw_n
        );
        let (g, _) = bringup_scan_geometry(&se, dpi, GeometryProfile::PreviewSafe).unwrap();
        let n = shading_acquire_width(
            g.pixel_startx,
            g.pixel_endx,
            g.register_dpiset,
            se.optical_resolution,
        )
        .unwrap();
        assert_eq!(
            (n, g.pixels, g.pixel_endx),
            (raw_n, raw.pixels, raw.pixel_endx)
        );
    }
}

#[test]
fn declared_shading_size_matches_capture_sessions() {
    assert_eq!(declared_shading_size(1728), 21064);
    assert_eq!(declared_shading_size(2478), 30208);
    assert_eq!(declared_shading_size(4956), 60416);
    assert_eq!(shading_width_for_resolution(1200), 1728);
    assert_eq!(shading_width_for_resolution(1800), 2478);
    for size in [21064, 30208, 60416] {
        assert_eq!(
            declared_shading_size(shading_entry_count(size).unwrap()),
            size
        );
    }
}

#[test]
fn pack_shading_table_pads_every_512_byte_block() {
    let n = 2478;
    let blob = make_unity_white_table(
        &vec![[927, 1039, 1171]; n],
        Some(declared_shading_size(n)),
        true,
    )
    .unwrap();
    assert_eq!(blob.len(), 30208);
    let gains: Vec<i64> = (0..blob.len())
        .step_by(4)
        .map(|o| u16::from_le_bytes([blob[o + 2], blob[o + 3]]) as i64)
        .collect();
    for (i, &g) in gains.iter().enumerate() {
        let in_pad = i % (SHADING_BLOCK_BYTES / 4) >= SHADING_BLOCK_DATA_PAIRS;
        assert_eq!(g, if in_pad { 0 } else { SHADING_GAIN_UNITY }, "pair {i}");
    }
    assert_eq!(
        gains.iter().filter(|&&g| g == SHADING_GAIN_UNITY).count(),
        n * 3
    );
    let rows = unpack_shading_table(&blob);
    assert_eq!(
        rows[0],
        ShadingEntry {
            dark: vec![927, 1039, 1171],
            gain: vec![SHADING_GAIN_UNITY; 3]
        }
    );
    assert_eq!(rows[n - 1].dark, vec![927, 1039, 1171]);
}

#[test]
fn shading_gain_inverts_the_white_profile() {
    let t = SHADING_GAIN_TARGET;
    assert_eq!(shading_gain_for_white(t as f64, t), SHADING_GAIN_UNITY);
    assert_eq!(
        shading_gain_for_white(t as f64 / 2.0, t),
        SHADING_GAIN_UNITY * 2
    );
    assert!(shading_gain_for_white(50000.0, t) > shading_gain_for_white(57000.0, t));
    assert_eq!(shading_gain_for_white(1.0, t), SHADING_GAIN_MAX);
    assert_eq!(shading_gain_for_white(1_000_000.0, t), SHADING_GAIN_MIN);
    let g = shading_gains_from_white(&[[50000, 55000, 56000]], t);
    assert!(g[0][0] > g[0][1] && g[0][1] > g[0][2]);
}

#[test]
fn measured_shading_table_stores_gains_not_whites() {
    let n = 256;
    let blob = build_measured_shading_table(
        &vec![[927, 1039, 1171]; n],
        &vec![[49610, 55067, 56009]; n],
        true,
        Some(declared_shading_size(n)),
        SHADING_GAIN_TARGET,
    )
    .unwrap();
    let rows = unpack_shading_table(&blob);
    assert_eq!(rows[0].dark, vec![927, 1039, 1171]);
    for (g, want) in rows[0].gain.iter().zip([10822, 9750, 9586]) {
        assert!((g - want).abs() <= 2);
        let r = *g as f64 / SHADING_GAIN_UNITY as f64;
        assert!((1.05..=2.0).contains(&r));
    }
    assert_eq!(rows[n - 1].gain, rows[0].gain);
}

#[test]
fn session04_capture_gains_match_silverfast() {
    let fx = common::read_json(
        &common::fixtures()
            .join("data")
            .join("session04_shading_gain.json"),
    );
    assert_eq!(fx["unity_gain"].as_i64().unwrap(), SHADING_GAIN_UNITY);
    let targets: Vec<i64> = fx["sf_target"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert_eq!(targets[0], SHADING_GAIN_TARGET);
    for col in fx["columns"].as_array().unwrap() {
        for (c, &t) in targets.iter().enumerate() {
            let got = shading_gain_for_white(col["white_unity"][c].as_f64().unwrap(), t);
            let want = col["coeff"][c].as_i64().unwrap();
            assert!(
                (got - want).abs() <= 16,
                "{} ch{c}: {got} vs {want}",
                col["x"]
            );
        }
    }
}

#[test]
fn validate_color_shading_strips() {
    let (ok, reason) =
        validate_color_shading_strip(&[[800, 900, 850]; 32], &[[20000, 52000, 50000]; 32], None);
    assert!(!ok && reason.contains("film"));
    assert_eq!(
        validate_color_shading_strip(&[[927, 1039, 1171]; 32], &[[49610, 55067, 56009]; 32], None),
        (true, "ok".into())
    );
    let (ok, reason) =
        validate_color_shading_strip(&[[800, 900, 850]; 32], &[[65535; 3]; 32], None);
    assert!(!ok && reason.contains("rail"));
    let (ok, reason) =
        validate_color_shading_strip(&[[800, 900, 850]; 32], &[[12000, 11500, 11000]; 32], None);
    assert!(!ok && reason.contains("white mean"));
    let (ok, reason) = validate_color_shading_strip(
        &[[5000, 5500, 4800]; 32],
        &[[50000, 52000, 51000]; 32],
        None,
    );
    assert!(ok && reason.contains("soft dark"));
}

#[test]
fn validate_color_shading_gains_band() {
    let (ok, reason) = validate_color_shading_gains(&shading_gains_from_white(
        &[[49610, 55067, 56009]; 32],
        SHADING_GAIN_TARGET,
    ));
    assert!(ok && reason.contains("median gain"));
    let (ok, reason) = validate_color_shading_gains(&[[SHADING_GAIN_UNITY; 3]; 32]);
    assert!(!ok && reason.contains("already at target"));
    let (ok, reason) = validate_color_shading_gains(&[[SHADING_GAIN_MAX; 3]; 32]);
    assert!(!ok && reason.contains("too dim"));
}

#[test]
fn coarse_gains_match_silverfast() {
    let sf = AFE_SF_WHITE_TARGETS_COLOR;
    assert_eq!(
        coarse_gains_from_wide_means([24768.0, 35190.0, 30620.0], sf, AFE_GAIN_MAX),
        [35, 25, 32]
    );
    assert_eq!(
        coarse_gains_from_wide_means([27316.0, 39037.0, 33349.0], sf, AFE_GAIN_MAX),
        [31, 20, 28]
    );
    assert_eq!(
        coarse_gains_from_wide_means([27028.0, 37931.0, 31495.0], sf, AFE_GAIN_MAX),
        [32, 22, 31]
    );
    assert_eq!(
        coarse_gains_from_wide_means(
            [32244.0, 27261.0, 28848.0],
            AFE_SF_WHITE_TARGETS_IR,
            AFE_GAIN_MAX
        ),
        [28, 35, 33]
    );
    assert_eq!(
        coarse_gains_from_wide_means(
            [26821.0, 36750.0, 32191.0],
            AFE_WHITE_TARGETS_COLOR,
            AFE_GAIN_MAX
        ),
        [35, 25, 32]
    );
    assert_eq!(
        coarse_gains_from_wide_means(
            [48199.0, 41725.0, 44274.0],
            AFE_WHITE_TARGETS_IR,
            AFE_GAIN_MAX
        ),
        [5, 15, 11]
    );
}

#[test]
fn search_afe_codes_respects_code_min() {
    let floor = 0x10;
    let codes = search_afe_codes(
        [0x80; 3],
        0xFF,
        0x3000 as f64,
        9,
        512.0,
        true,
        &mut |_| Ok([40000.0; 3]),
        floor,
    )
    .unwrap();
    assert_eq!(codes, [floor; 3]);
}

#[test]
fn afe_offset_sign_magnitude_round_trip() {
    for v in -255..=255 {
        assert_eq!(decode_afe_offset(encode_afe_offset(v)), v);
    }
    assert_eq!(encode_afe_offset(-106), 0x16A);
    assert_eq!(encode_afe_offset(400), 0xFF);
}

#[test]
fn validate_rejects_starved_channel() {
    const { assert!(COLOR_SHADING_CHANNEL_WHITE_MIN >= 8000.0) };
    let (ok, reason) =
        validate_color_shading_strip(&[[900, 1000, 1100]; 32], &[[55000, 56000, 4000]; 32], None);
    assert!(!ok && reason.contains("ch2 white"), "{reason}");
}

// --- host calibration (ImagePipeline) ---

use ndarray::{Array2, Array3, s};
use opticfilm::scan::pipeline::{
    HOST_CALIB_BORDER_INSET, HOST_CALIB_HIGHLIGHT_CEILING, HOST_CALIB_HIGHLIGHT_PERCENTILE,
    HOST_CALIB_PEAK_PERCENTILE, HOST_CALIB_PEAK_TARGET, ImagePipeline,
};

fn pipe() -> ImagePipeline {
    ImagePipeline::new(model_by_name("plustek-opticfilm-8200i-se").unwrap())
}

fn rgb_of(h: usize, w: usize, rgb: [u16; 3]) -> Array3<u16> {
    Array3::from_shape_fn((h, w, 3), |(_, _, c)| rgb[c])
}

fn percentile(a: impl Iterator<Item = u16>, q: f64) -> f64 {
    let mut v: Vec<f64> = a.map(f64::from).collect();
    v.sort_by(f64::total_cmp);
    let pos = q / 100.0 * (v.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    v[lo] + (v[hi] - v[lo]) * (pos - lo as f64)
}

fn chrome_border(rgb: &mut Array3<u16>, n: usize) {
    let n = n as isize;
    rgb.slice_mut(s![..n, .., ..]).fill(65535);
    rgb.slice_mut(s![-n.., .., ..]).fill(65535);
    rgb.slice_mut(s![.., ..n, ..]).fill(65535);
    rgb.slice_mut(s![.., -n.., ..]).fill(65535);
}

/// Home-chrome stretch leaves film-window peaks low: makeup to near white.
#[test]
fn host_calib_exposure_makeup_lifts_dim_film_window() {
    let w = 32;
    // Raw: mid film ~8000, bright film ~16000; dark ~1000, home white ~50000.
    let mut rgb = rgb_of(16, w, [8000; 3]);
    rgb.slice_mut(s![.., w / 4..3 * w / 4, ..]).fill(16000);
    let dark = Array2::from_elem((w, 3), 1000.0f32);
    let white = Array2::from_elem((w, 3), 50000.0f32);
    let out = pipe()
        .apply_host_calib(
            rgb,
            &dark,
            &white,
            HOST_CALIB_PEAK_TARGET,
            HOST_CALIB_PEAK_PERCENTILE,
            true,
        )
        .unwrap();
    assert!(percentile(out.iter().copied(), 99.7) >= HOST_CALIB_PEAK_TARGET as f64 * 0.95);
    // Midtones must lift with the peak (not only clip edges).
    assert!(percentile(out.iter().copied(), 50.0) > 20000.0);
}

/// Holder chrome brighter than film inset must not stay above film peak.
#[test]
fn host_calib_border_clamp_caps_hot_margins() {
    let mut rgb = rgb_of(100, 100, [20000; 3]);
    rgb.slice_mut(s![10..90, 10..90, ..]).fill(25000);
    chrome_border(&mut rgb, 4);
    let out =
        pipe().clamp_border_highlights(rgb, HOST_CALIB_BORDER_INSET, HOST_CALIB_PEAK_PERCENTILE);
    let max = |a: ndarray::ArrayView3<u16>| a.iter().copied().max().unwrap();
    assert!(max(out.slice(s![..4, .., ..])) <= max(out.slice(s![10..90, 10..90, ..])));
    let centre = out.slice(s![40..60, 40..60, ..]);
    assert_eq!(
        centre.iter().map(|&v| v as u64).sum::<u64>() / centre.len() as u64,
        25000
    );
}

/// A joint ceiling leaves the margin above a dim channel's own film peak
/// (session 004: green film peak 19306 vs joint p99.7 27432).
#[test]
fn border_clamp_ceiling_is_per_channel() {
    let levels = [27000u16, 19300, 27600];
    let mut rgb = rgb_of(200, 200, levels);
    chrome_border(&mut rgb, 6);
    let out =
        pipe().clamp_border_highlights(rgb, HOST_CALIB_BORDER_INSET, HOST_CALIB_PEAK_PERCENTILE);
    for (c, &expected) in levels.iter().enumerate() {
        assert_eq!(
            out.slice(s![20..180, 20..180, c])
                .iter()
                .copied()
                .max()
                .unwrap(),
            expected
        );
        assert!(out.slice(s![..6, .., c]).iter().copied().max().unwrap() <= expected + 1);
    }
}

/// The makeup must be scalar: a per-channel lift neutralizes the mask.
#[test]
fn expose_film_base_keeps_the_orange_mask_ratios() {
    // Session 004 film-window levels: base at 42% of scale, green 0.71x of red.
    let out = pipe().expose_film_base(
        rgb_of(120, 120, [27046, 19306, 27695]),
        "test",
        HOST_CALIB_PEAK_TARGET,
        HOST_CALIB_PEAK_PERCENTILE,
        false,
        HOST_CALIB_HIGHLIGHT_CEILING,
        HOST_CALIB_HIGHLIGHT_PERCENTILE,
    );
    let peak = *out.iter().max().unwrap() as f64;
    assert!(
        (0.98..=1.02).contains(&(peak / HOST_CALIB_PEAK_TARGET as f64)),
        "{peak}"
    );
    let mean = |c| {
        out.slice(s![.., .., c])
            .iter()
            .map(|&v| v as f64)
            .sum::<f64>()
            / (120.0 * 120.0)
    };
    assert!((mean(1) / mean(0) - 19306.0 / 27046.0).abs() < 0.005);
}
