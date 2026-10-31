// SPDX-License-Identifier: GPL-3.0-or-later
//! Host numerics (merge, registration, warps, IR flatten) and the 7600i v1
//! vendor jobs match pyopticfilm.

mod common;

use ndarray::{Array2, Array3, Axis, s};
use opticfilm::device::replay_data::replay_profile;
use opticfilm::pass_align::{
    align_pass_to_reference, align_pass_to_reference_banded, estimate_pass_shift,
};
use opticfilm::registration::{warp_cubic, warp_linear};
use opticfilm::scan::calib_gl128::{
    IR_FLATTEN_BORDER_INSET, IR_FLATTEN_MAX_GAIN, IR_SIDECAR_TARGET_LEVEL, flatten_ir_columns,
    flatten_ir_image_columns,
};
use opticfilm::scan::exposure_merge::{MergeStats, merge_exposures_result, merge_n_passes};
use opticfilm::scan::me_exposure::{LongExposureParams, select_long_exposure};
use serde_json::Value;

fn scene(h: usize, w: usize) -> Vec<[f64; 3]> {
    let mut v = Vec::with_capacity(h * w);
    for y in 0..h {
        for x in 0..w {
            let (xf, yf) = (x as f64, y as f64);
            let check = ((x / 60 + y / 45) % 2) as f64;
            let base = 9000.0 + 7000.0 * (xf / 37.0).sin() * (yf / 53.0).cos() + 4000.0 * check;
            v.push([base, base * 0.8 + 2000.0, base * 0.6 + 4000.0]);
        }
    }
    v
}

fn u16img(h: usize, w: usize, f: impl Fn(usize, usize) -> f64) -> Array3<u16> {
    Array3::from_shape_fn((h, w, 3), |(y, x, c)| {
        f(y * w + x, c).round_ties_even().clamp(0.0, 65535.0) as u16
    })
}

fn roll(a: &Array3<u16>, dy: isize, dx: isize) -> Array3<u16> {
    let (h, w, _) = a.dim();
    Array3::from_shape_fn(a.dim(), |(y, x, c)| {
        a[[
            (y as isize - dy).rem_euclid(h as isize) as usize,
            (x as isize - dx).rem_euclid(w as isize) as usize,
            c,
        ]]
    })
}

/// Exact sha match, or report the strided-sample difference.
fn check(
    name: &str,
    want: &Value,
    got_shape: &[usize],
    got: &[u16],
    tol: i32,
    fails: &mut Vec<String>,
) {
    let ws: Vec<usize> = want["shape"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    assert_eq!(ws, got_shape, "{name}: shape");
    if common::sha256_u16(got.iter().copied()) == want["sha256"].as_str().unwrap() {
        return;
    }
    let step = want["step"].as_u64().unwrap() as usize;
    let sample = common::u16s(&common::unzlib_b64(want["sample"].as_str().unwrap()));
    let (h, w) = (got_shape[0], got_shape[1]);
    let c = if got_shape.len() == 3 {
        got_shape[2]
    } else {
        1
    };
    let mut k = 0;
    let (mut n, mut maxd) = (0, 0);
    for y in (0..h).step_by(step) {
        for x in (0..w).step_by(step) {
            for ch in 0..c {
                let d = (got[(y * w + x) * c + ch] as i32 - sample[k] as i32).abs();
                if d > 0 {
                    n += 1;
                    maxd = maxd.max(d);
                }
                k += 1;
            }
        }
    }
    let msg = format!("{name}: not bit-exact; {n} of {k} sampled values differ, max |d| = {maxd}");
    eprintln!("{msg}");
    if maxd > tol {
        fails.push(msg);
    }
}

#[test]
fn numerics_match_python() {
    let fx = common::read_json(&common::fixtures().join("numerics.json"));
    let probe = common::noise(4, 3);
    for (a, b) in probe.iter().zip(fx["noise_probe"].as_array().unwrap()) {
        assert_eq!(*a, b.as_f64().unwrap(), "noise generator");
    }
    let cases = &fx["cases"];
    let (h, w) = (600usize, 700usize);
    let sc = scene(h, w);
    let n1 = common::noise(h * w * 3, 1);
    let n2 = common::noise(h * w * 3, 2);
    let short = u16img(h, w, |i, c| sc[i][c] + 60.0 * n1[i * 3 + c]);
    let long = u16img(h, w, |i, c| sc[i][c] * 3.0 + 120.0 * n2[i * 3 + c]);
    let shifted = roll(&long, 1, 2);
    let mut fails = Vec::new();
    let flat = |a: &Array3<u16>| a.iter().copied().collect::<Vec<u16>>();

    let m =
        merge_exposures_result(&short, &long, 14000, 42000, Some((0.0, 0.0)), None, None).unwrap();
    check(
        "merge_noshift",
        &cases["merge_noshift"]["rgb"],
        m.rgb.shape(),
        &flat(&m.rgb),
        1,
        &mut fails,
    );
    if let Some(MergeStats::Fusion(st)) = &m.fusion_stats {
        let fitted: Vec<bool> = cases["merge_noshift"]["fitted"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_bool().unwrap())
            .collect();
        assert_eq!(st.noise_fitted.as_ref().unwrap(), &fitted);
        for (k, key) in [(0, "alpha"), (1, "beta"), (2, "ratio")] {
            let got = match k {
                0 => st.noise_alpha.clone(),
                1 => st.noise_beta.clone(),
                _ => st.exposure_ratio_per_channel.clone(),
            }
            .unwrap();
            for (g, wv) in got
                .iter()
                .zip(cases["merge_noshift"][key].as_array().unwrap())
            {
                let wv = wv.as_f64().unwrap();
                assert!(
                    (g - wv).abs() <= 1e-4 * wv.abs().max(1.0),
                    "{key}: {g} vs {wv}"
                );
            }
        }
    }
    let frames: Vec<Array3<u16>> = (0..3)
        .map(|k| {
            let n = common::noise(h * w * 3, 10 + k);
            u16img(h, w, |i, c| sc[i][c] + 80.0 * n[i * 3 + c])
        })
        .collect();
    let mp = merge_n_passes(&frames, None, None).unwrap();
    check(
        "merge_n3",
        &cases["merge_n3"]["rgb"],
        mp.rgb.shape(),
        &flat(&mp.rgb),
        1,
        &mut fails,
    );

    let close = |a: (f64, f64), b: &Value, what: &str| {
        let b = b.as_array().unwrap();
        let (bx, by) = (b[0].as_f64().unwrap(), b[1].as_f64().unwrap());
        assert!(
            (a.0 - bx).abs() < 1e-3 && (a.1 - by).abs() < 1e-3,
            "{what}: {a:?} vs ({bx}, {by})"
        );
    };
    close(
        estimate_pass_shift(&short, &roll(&short, -2, 3)),
        &cases["shift_est"],
        "shift_est",
    );
    close(
        estimate_pass_shift(&short, &shifted),
        &cases["shift_est_long"],
        "shift_est_long",
    );
    let (al, sh) = align_pass_to_reference(&short, &shifted, None).unwrap();
    close(sh, &cases["align"]["shift"], "align shift");
    check(
        "align",
        &cases["align"]["rgb"],
        al.shape(),
        &flat(&al),
        1,
        &mut fails,
    );
    let tall = ndarray::concatenate(
        Axis(0),
        &[short.view(), short.view(), short.view(), short.view()],
    )
    .unwrap();
    let (alb, sb) = align_pass_to_reference_banded(&tall, &roll(&tall, 0, 1)).unwrap();
    close(sb, &cases["banded"]["shift"], "banded shift");
    check(
        "banded",
        &cases["banded"]["rgb"],
        alb.shape(),
        &flat(&alb),
        1,
        &mut fails,
    );
    let wl = warp_linear(&short, 1.3, -0.6);
    check(
        "warp_linear",
        &cases["warp_linear"],
        wl.shape(),
        &flat(&wl),
        0,
        &mut fails,
    );
    let wc = warp_cubic(&short, -2.25, 0.4);
    check(
        "warp_cubic",
        &cases["warp_cubic"],
        wc.shape(),
        &flat(&wc),
        0,
        &mut fails,
    );

    let d = select_long_exposure(short.view(), 14000, &LongExposureParams::default());
    let sl = &cases["select_long"];
    assert_eq!(d.selected, sl["selected"].as_i64().unwrap());
    assert_eq!(d.proposed, sl["proposed"].as_i64().unwrap());
    assert_eq!(d.reason, sl["reason"].as_str().unwrap());

    let ir: Array2<u16> = short.slice(s![.., .., 1]).to_owned();
    let white: Vec<i64> = (0..w)
        .map(|x| (20000.0 + 3000.0 * (x as f64 / 90.0).sin()) as i64)
        .collect();
    let f1 = flatten_ir_columns(ir.view(), &white, None, None).unwrap();
    check(
        "flatten_ir_columns",
        &cases["flatten_ir_columns"],
        f1.shape(),
        &f1.iter().copied().collect::<Vec<_>>(),
        0,
        &mut fails,
    );
    let f2 = flatten_ir_image_columns(
        ir.view(),
        90.0,
        Some(IR_SIDECAR_TARGET_LEVEL),
        None,
        IR_FLATTEN_BORDER_INSET,
        IR_FLATTEN_MAX_GAIN,
    )
    .unwrap();
    check(
        "flatten_ir_image",
        &cases["flatten_ir_image"],
        f2.shape(),
        &f2.iter().copied().collect::<Vec<_>>(),
        0,
        &mut fails,
    );
    assert!(fails.is_empty(), "{fails:#?}");
}

#[test]
fn vendor_jobs_and_derived_infrared_match_python() {
    use sha2::{Digest, Sha256};
    let want = common::read_json(&common::fixtures().join("replay_profiles.json"));
    for (key, h) in want.as_object().unwrap() {
        let (mode, dpi) = key.split_once('_').unwrap();
        let p = replay_profile("OpticFilm 7600i (v1)", mode, dpi.parse().unwrap()).unwrap();
        let text = serde_json::to_string(&p).unwrap();
        let got: String = Sha256::digest(text.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(got, h.as_str().unwrap(), "{key}");
    }
}
