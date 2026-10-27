//! Headless batch mode: `nlp_studio convert [options] <files…>`.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::engine::settings::{
    ExportFormat, ExportSettings, FilmBase, Lut, RollMode, ToneProfile, WbMode,
};
use crate::engine::sidecar::{self, RollData, Sidecar};
use crate::engine::{decode, export, frame, pipeline};

const HELP: &str = "NLP Studio — negative conversion

USAGE:
  nlp_studio                          Launch the GUI
  nlp_studio convert [options] FILES  Convert negatives and write positive copies
  nlp_studio scan-prep FILES          Level + gamma 2.2 encode linear TIF scans

CONVERT OPTIONS:
  --roll               Analyse all files as one roll
  --roll-process NAME  Pro Lab (default) | Basic Lab | Darkroom Paper
  --tone NAME          Tone profile (e.g. \"LAB - Standard\", \"Cinematic - Rich\")
  --wb NAME            White balance (Auto-Neutral, Kodak, Fuji, ...)
  --lut NAME           None | Natural | Frontier | Crystal | Pakon
  --border PCT         Border buffer percent (default 5)
  --auto-crop          Crop and straighten each frame to its detected frame
  --jpg                Write JPEG instead of 16-bit TIFF
  --long-edge PX       Resize so the long edge is at most PX (default: full size)
  --name TEMPLATE      File name: {name} {suffix} {n} {n:03} {stock} {iso} {date}
                       (default \"{name}{suffix}\")
  --out DIR            Output folder (default: <source>/Positives)
";

fn find<T: Copy>(all: &[T], label: impl Fn(T) -> &'static str, name: &str) -> Result<T> {
    let n = name.to_ascii_lowercase();
    all.iter()
        .copied()
        .find(|v| label(*v).to_ascii_lowercase() == n)
        .with_context(|| format!("unknown value '{name}'"))
}

pub fn run(args: Vec<String>) -> Result<()> {
    let mut it = args.into_iter();
    let cmd = it.next().unwrap_or_default();
    match cmd.as_str() {
        "convert" => convert(it.collect()),
        "scan-prep" => {
            for f in it {
                println!("{}", export::tif_scan_prep(Path::new(&f))?.display());
            }
            Ok(())
        }
        "synth" => {
            let out = it.next().unwrap_or_else(|| "synthetic-negative.tif".into());
            synth_negative(
                Path::new(&out),
                it.next().map(|v| v.parse().unwrap_or(0)).unwrap_or(0),
            )
        }
        _ => {
            print!("{HELP}");
            Ok(())
        }
    }
}

fn convert(args: Vec<String>) -> Result<()> {
    let mut s = sidecar::load_defaults().settings;
    let mut ex = ExportSettings {
        format: ExportFormat::Tiff,
        ..Default::default()
    };
    let mut roll = false;
    let mut out: Option<PathBuf> = None;
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let mut val = || {
            i += 1;
            args.get(i).cloned().context("missing option value")
        };
        match a.as_str() {
            "--roll" => roll = true,
            "--roll-process" => s.roll_mode = find(RollMode::ALL, RollMode::label, &val()?)?,
            "--tone" => s.tone_profile = find(ToneProfile::ALL, ToneProfile::label, &val()?)?,
            "--wb" => s.wb_mode = find(WbMode::ALL, WbMode::label, &val()?)?,
            "--lut" => s.lut = find(Lut::ALL, Lut::label, &val()?)?,
            "--border" => s.border_buffer = val()?.parse()?,
            "--auto-crop" => s.auto_crop = true,
            "--jpg" => ex.format = ExportFormat::Jpeg,
            "--long-edge" => ex.long_edge = val()?.parse()?,
            "--name" => ex.name_template = val()?,
            "--out" => out = Some(PathBuf::from(val()?)),
            f if f.starts_with("--") => bail!("unknown option {f}\n{HELP}"),
            f => files.push(PathBuf::from(f)),
        }
        i += 1;
    }
    if files.is_empty() {
        bail!("no input files\n{HELP}");
    }
    // `createRoll` only runs for more than one frame.
    let roll = roll && files.len() > 1;
    // Analyse every frame on its proxy.
    let mut frames = Vec::new();
    for f in &files {
        let src = decode::load(f, Some(1200)).with_context(|| format!("load {}", f.display()))?;
        let base = pipeline::resolve_base(&src, &s, None);
        let mut fs = s.clone();
        if fs.auto_crop {
            match frame::detect(&src, &fs, base) {
                Some(d) => (fs.crop, fs.straighten) = (d.crop, d.straighten),
                None => eprintln!("warning: no frame edge found in {}", f.display()),
            }
        }
        frames.push((f.clone(), src, base, fs));
    }
    let roll_base = (roll && s.film_base == FilmBase::Auto).then(|| {
        let mut b = [0.0f32; 3];
        for c in 0..3 {
            let mut v: Vec<f32> = frames.iter().map(|f| f.2[c]).collect();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            b[c] = v[v.len() / 2];
        }
        b
    });
    let mut analyses = Vec::new();
    for (_, src, base, fs) in &frames {
        let base = roll_base.unwrap_or(*base);
        let neg = pipeline::prepare_negative(src, fs, base);
        analyses.push(pipeline::analyze(
            &neg,
            fs,
            base,
            pipeline::frame_ev(&src.info),
        ));
    }
    let roll_data = roll.then(|| {
        // `L22_1`: with more than 1 EV between frames there is no prompt in
        // batch mode, so continue without exposure data.
        let common_ev = pipeline::common_ev(&analyses).filter(|&c| {
            let spread = pipeline::ev_spread(&analyses, c);
            if spread > 1.0 {
                eprintln!(
                    "warning: frames differ by {spread:.1} EV in their exposure metadata; \
                     converting the roll without exposure data"
                );
            }
            spread <= 1.0
        });
        let (agg, excluded) = pipeline::aggregate_roll(&analyses, common_ev);
        RollData {
            id: format!("roll-{}", std::process::id()),
            origins: agg.origins,
            base: roll_base.unwrap_or_else(|| analyses[0].base),
            frames: files.len(),
            auto_wb: Some(agg.auto_wb),
            multipliers: Some(agg.multipliers),
            common_ev: agg.common_ev,
            excluded: excluded
                .iter()
                .map(|i| files[*i].display().to_string())
                .collect(),
        }
    });
    if let Some(r) = &roll_data {
        s.use_roll = true;
        if !r.excluded.is_empty() {
            println!("roll outliers excluded from aggregate: {:?}", r.excluded);
        }
    }
    if let Some(o) = &out {
        std::fs::create_dir_all(o)?;
        ex.output_dir = Some(o.clone());
    }
    // Keep film metadata already entered for each scan.
    let metas: Vec<_> = files
        .iter()
        .map(|f| sidecar::load(f).map(|sc| sc.metadata).unwrap_or_default())
        .collect();
    let items: Vec<_> = files.iter().map(PathBuf::as_path).zip(&metas).collect();
    let targets = export::batch_paths(&items, &ex);
    for ((((f, _, _, fs), a), meta), target) in
        frames.iter().zip(analyses).zip(&metas).zip(&targets)
    {
        let mut fs = fs.clone();
        fs.use_roll = s.use_roll;
        let wb = roll_data
            .as_ref()
            .and_then(|r| r.auto_wb)
            .unwrap_or(a.auto_wb);
        if let Some((t, n)) = pipeline::wb_for_mode(fs.wb_mode, &wb) {
            fs.wb_temp = t;
            fs.wb_tint = n;
        }
        let target = export::export_frame(
            f,
            &fs,
            Some(&a),
            roll_data.as_ref(),
            meta,
            &ex,
            target,
        )?;
        println!("{} -> {}", f.display(), target.display());
        let sc = Sidecar {
            settings: fs,
            analysis: Some(a),
            roll: roll_data.clone(),
            metadata: meta.clone(),
            ..Default::default()
        };
        sidecar::save(f, &sc)?;
    }
    Ok(())
}

/// Generate a synthetic colour negative (orange mask, H&D response) for testing.
fn synth_negative(path: &Path, variant: u32) -> Result<()> {
    let (w, h) = (900usize, 600usize);
    let patches: [[f32; 3]; 12] = [
        [0.45, 0.32, 0.25], // skin
        [0.12, 0.25, 0.55], // sky
        [0.15, 0.35, 0.10], // foliage
        [0.60, 0.10, 0.08], // red
        [0.70, 0.60, 0.10], // yellow
        [0.10, 0.10, 0.40], // blue
        [0.05, 0.05, 0.05],
        [0.18, 0.18, 0.18],
        [0.40, 0.40, 0.40],
        [0.75, 0.75, 0.75],
        [0.30, 0.15, 0.40],
        [0.20, 0.45, 0.45],
    ];
    let exposure = 1.0 + variant as f32 * 0.35;
    let base = [0.92f32, 0.62, 0.38];
    let gamma = [0.62f32, 0.66, 0.70];
    let mut px = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        for x in 0..w {
            let border = x < 30 || x >= w - 30 || y < 30 || y >= h - 30;
            let scene = if border {
                None
            } else if y < 120 {
                let t = (x - 30) as f32 / (w - 60) as f32;
                Some([t * t * 0.9 + 0.01; 3])
            } else {
                let (cx, cy) = ((x - 30) * 6 / (w - 60), (y - 120) * 2 / (h - 150));
                Some(patches[(cy.min(1) * 6 + cx.min(5)) % 12])
            };
            let t = match scene {
                None => base,
                Some(e) => [0, 1, 2]
                    .map(|c| base[c] * ((e[c] * exposure + 0.004) / 0.004).powf(-gamma[c] * 0.42)),
            };
            px[y * w + x] = t.map(crate::engine::color::srgb_encode);
        }
    }
    let icc = crate::engine::color::icc_for(crate::engine::settings::OutputSpace::Srgb);
    export::write_tiff(
        path,
        w,
        h,
        &px,
        true,
        true,
        &icc,
        &Default::default(),
        &Default::default(),
    )?;
    println!("wrote {}", path.display());
    Ok(())
}
