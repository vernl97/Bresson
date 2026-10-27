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
  --roll-process NAME  Lab (default: density per frame) | Darkroom (one exposure)
  --tone NAME          Tone profile: Standard | Soft | Hard | Rich | Flat | Linear
  --wb NAME            White balance (Auto-Neutral, Kodak, Fuji, ...)
  --lut NAME           None | Natural | Frontier | Crystal | Pakon
  --border PCT         Border buffer percent (default 5)
  --auto-crop          Crop and straighten each frame to its detected frame
  --flat FILE          Flat-field frame (bare light or clear film leader, shot like the negatives)
  --jpg                Write JPEG instead of 16-bit TIFF
  --long-edge PX       Resize so the long edge is at most PX (default: full size)
  --name TEMPLATE      File name: {name} {suffix} {n} {n:03} {stock} {iso} {date}
                       (default \"{name}{suffix}\")
  --out DIR            Output folder (default: <source>/Positives)
";

/// The choice whose label is `name`, or starts with `name` and a space
/// ("Flat" for "Flat (for grading)").
fn find<T: Copy>(all: &[T], label: impl Fn(T) -> &'static str, name: &str) -> Result<T> {
    let n = name.to_ascii_lowercase();
    all.iter()
        .copied()
        .find(|v| {
            let l = label(*v).to_ascii_lowercase();
            l == n || l.starts_with(&format!("{n} "))
        })
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
                it.next().map(|v| v.parse().unwrap_or(0.0)).unwrap_or(0.0),
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
            "--flat" => s.flat_field = Some(PathBuf::from(val()?)),
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
    // A roll needs more than one frame.
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
        // With more than 1 EV between frames there is no prompt in batch
        // mode, so continue without exposure data.
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
            levels: agg.levels,
            balance: agg.balance,
            base: roll_base.unwrap_or_else(|| analyses[0].base),
            frames: files.len(),
            auto_wb: Some(agg.auto_wb),
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
        let target = export::export_frame(f, &fs, Some(&a), roll_data.as_ref(), meta, &ex, target)?;
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

/// Write the physically modelled ColorChecker negative (`testchart`),
/// exposed `ev` stops over normal, as a linear-light 16-bit TIFF.
fn synth_negative(path: &Path, ev: f64) -> Result<()> {
    let chart = crate::engine::testchart::negative(ev);
    let src = &chart.src;
    let px: Vec<[f32; 3]> = src
        .data
        .iter()
        .map(|p| {
            crate::engine::color::mul3(&src.to_melissa, *p).map(crate::engine::color::srgb_encode)
        })
        .collect();
    let icc = crate::engine::color::icc_for(crate::engine::settings::OutputSpace::Srgb);
    export::write_tiff(
        path,
        src.width,
        src.height,
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
