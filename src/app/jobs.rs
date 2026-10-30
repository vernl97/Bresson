//! Background work (decoding, analysis, export) on the rayon pool.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::engine::decode::{self, SourceImage, SourceInfo};
use crate::engine::frame::{self, Detected};
use crate::engine::pipeline::{self, Analysis};
use crate::engine::settings::{CropAspect, Settings};
use crate::engine::sidecar::{self, RollData};

/// Long edge of the per-frame proxy used for thumbnails and analysis.
pub const PROXY_SIZE: usize = 1024;
/// Long edge of the editor preview.
pub const EDITOR_SIZE: usize = 2048;

pub enum JobResult {
    Proxy {
        path: PathBuf,
        /// The proxy and its grid thumbnail source.
        result: Result<(Arc<SourceImage>, SourceImage), String>,
    },
    /// The cached grid thumbnail source, ahead of the proxy; `None` on a miss.
    Thumb {
        path: PathBuf,
        thumb: Option<SourceImage>,
    },
    Editor {
        path: PathBuf,
        result: Result<Arc<SourceImage>, String>,
    },
    Converted {
        /// The frame geometry, for frames converted with auto-crop.
        frames: Vec<(PathBuf, Arc<SourceImage>, Analysis, Option<Detected>)>,
        roll: Option<RollData>,
        errors: Vec<String>,
    },
    Exported {
        path: PathBuf,
        result: Result<PathBuf, String>,
    },
    ContactSheet(Result<PathBuf, String>),
    /// The roll's frames differ by more than 1 EV in their exposure
    /// metadata; ask before converting without it (`L22_1`).
    ExposureVariance {
        items: Vec<ConvertItem>,
        spread: f64,
    },
    Message(String),
}

pub struct Jobs {
    tx: Sender<JobResult>,
    pub rx: Receiver<JobResult>,
    ctx: egui::Context,
    pub active: Arc<AtomicUsize>,
}

impl Jobs {
    pub fn new(ctx: egui::Context) -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            ctx,
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    pub fn busy(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }

    pub fn spawn(&self, f: impl FnOnce() -> JobResult + Send + 'static) {
        let tx = self.tx.clone();
        let ctx = self.ctx.clone();
        let active = self.active.clone();
        active.fetch_add(1, Ordering::Relaxed);
        rayon::spawn(move || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
                .unwrap_or_else(|_| JobResult::Message("A background task crashed".into()));
            let _ = tx.send(r);
            active.fetch_sub(1, Ordering::Relaxed);
            ctx.request_repaint();
        });
    }
}

// ---- proxy cache -----------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct ProxyHeader {
    width: usize,
    height: usize,
    to_melissa: [[f32; 3]; 3],
    as_shot_wb: [f32; 3],
    is_raw: bool,
    info: SourceInfo,
}

const MAGIC: &[u8; 8] = b"NLPPRX04";

fn read_cache(p: &Path) -> Option<SourceImage> {
    let mut f = std::fs::File::open(p).ok()?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).ok()?;
    if buf.len() < 12 || &buf[..8] != MAGIC {
        return None;
    }
    let hl = u32::from_le_bytes(buf[8..12].try_into().ok()?) as usize;
    let h: ProxyHeader = serde_json::from_slice(buf.get(12..12 + hl)?).ok()?;
    let body = buf.get(12 + hl..)?;
    if body.len() != h.width * h.height * 12 {
        return None;
    }
    let data = body
        .chunks_exact(12)
        .map(|c| [0, 1, 2].map(|i| f32::from_le_bytes(c[i * 4..i * 4 + 4].try_into().unwrap())))
        .collect();
    Some(SourceImage {
        width: h.width,
        height: h.height,
        data,
        to_melissa: h.to_melissa,
        as_shot_wb: h.as_shot_wb,
        is_raw: h.is_raw,
        info: h.info,
    })
}

fn write_cache(p: &Path, s: &SourceImage) {
    let h = ProxyHeader {
        width: s.width,
        height: s.height,
        to_melissa: s.to_melissa,
        as_shot_wb: s.as_shot_wb,
        is_raw: s.is_raw,
        info: s.info.clone(),
    };
    let Ok(hj) = serde_json::to_vec(&h) else {
        return;
    };
    let mut out = Vec::with_capacity(12 + hj.len() + s.data.len() * 12);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(hj.len() as u32).to_le_bytes());
    out.extend_from_slice(&hj);
    for px in &s.data {
        for v in px {
            out.extend_from_slice(&v.to_le_bytes());
        }
    }
    if let Ok(mut f) = std::fs::File::create(p) {
        let _ = f.write_all(&out);
    }
}

/// Load the analysis/thumbnail proxy, using the disk cache when possible.
pub fn load_proxy(path: &Path) -> anyhow::Result<SourceImage> {
    let cache = sidecar::proxy_cache_path(path);
    if let Some(c) = &cache {
        if let Some(img) = read_cache(c) {
            return Ok(img);
        }
    }
    let img = decode::load(path, Some(PROXY_SIZE))?;
    if let Some(c) = &cache {
        write_cache(c, &img);
    }
    Ok(img)
}

/// Downscale a proxy to the grid thumbnail source.
pub fn make_thumb(p: &SourceImage) -> SourceImage {
    let (data, width, height) = decode::downscale(&p.data, p.width, p.height, super::THUMB_SIZE);
    SourceImage {
        data,
        width,
        height,
        to_melissa: p.to_melissa,
        as_shot_wb: p.as_shot_wb,
        is_raw: p.is_raw,
        info: p.info.clone(),
    }
}

fn thumb_cache_path(image: &Path) -> Option<PathBuf> {
    sidecar::proxy_cache_path(image).map(|c| c.with_extension("thumb"))
}

/// The cached grid thumbnail source, if any: about 1 MB against the proxy's 8 MB.
pub fn read_thumb(image: &Path) -> Option<SourceImage> {
    read_cache(&thumb_cache_path(image)?)
}

/// Load the proxy and its thumbnail source, caching the thumbnail if it
/// was not yet.
pub fn load_proxy_and_thumb(path: &Path) -> anyhow::Result<(SourceImage, SourceImage)> {
    let proxy = load_proxy(path)?;
    let thumb = make_thumb(&proxy);
    if let Some(c) = thumb_cache_path(path) {
        if !c.exists() {
            write_cache(&c, &thumb);
        }
    }
    Ok((proxy, thumb))
}

// ---- conversion ------------------------------------------------------------------

/// Whether a frame converted with `auto_crop` has no crop or straighten of
/// its own yet.
fn wants_auto_crop(s: &Settings) -> bool {
    s.auto_crop && s.crop == [0.0; 4] && s.straighten == 0.0
}

/// Frame detection for a frame that [`wants_auto_crop`], free-form: the crop
/// follows the frame rather than the aspect ratio.
pub fn auto_crop(src: &SourceImage, s: &Settings) -> Option<Detected> {
    if !wants_auto_crop(s) {
        return None;
    }
    let s = &Settings {
        crop_aspect: CropAspect::Free,
        ..s.clone()
    };
    frame::detect(src, s, pipeline::resolve_base(src, s, None))
}

pub struct ConvertItem {
    pub path: PathBuf,
    pub proxy: Option<Arc<SourceImage>>,
    pub settings: Settings,
}

/// Analyse frames individually or as a roll. `use_exposure` matches the
/// roll's frames by their EXIF exposure (`commonEV`).
pub fn convert(items: Vec<ConvertItem>, as_roll: bool, use_exposure: bool) -> JobResult {
    use rayon::prelude::*;
    let loaded: Vec<Result<(PathBuf, Arc<SourceImage>, Settings), String>> = items
        .into_par_iter()
        .map(|it| {
            let src = match it.proxy {
                Some(p) => p,
                None => Arc::new(
                    load_proxy(&it.path).map_err(|e| format!("{}: {e:#}", it.path.display()))?,
                ),
            };
            Ok((it.path, src, it.settings))
        })
        .collect();
    let mut errors = Vec::new();
    let mut ok: Vec<(PathBuf, Arc<SourceImage>, Settings)> = loaded
        .into_iter()
        .filter_map(|r| r.map_err(|e| errors.push(e)).ok())
        .collect();
    if ok.is_empty() {
        return JobResult::Converted {
            frames: vec![],
            roll: None,
            errors,
        };
    }
    // Crop before analysing, so the analysis sees only the image. A frame
    // whose detection finds nothing, or something far from the rest, takes
    // the geometry the auto-cropped frames share (`frame::settle`).
    let found: Vec<Option<Option<Detected>>> = ok
        .par_iter()
        .map(|(_, src, s)| wants_auto_crop(s).then(|| auto_crop(src, s)))
        .collect();
    let shared: Vec<Detected> = ok
        .iter()
        .zip(&found)
        .filter_map(|((_, _, s), f)| match f {
            Some(d) => *d,
            None => s.auto_crop.then_some(Detected {
                crop: s.crop,
                straighten: s.straighten,
            }),
        })
        .collect();
    let shared = frame::roll_geometry(&shared);
    let detected: Vec<Option<Detected>> = ok
        .iter_mut()
        .zip(found)
        .map(|((_, _, s), f)| {
            if let Some(d) = f.and_then(|f| frame::settle(f, shared)) {
                (s.crop, s.straighten) = (d.crop, d.straighten);
            }
            // Also report geometry detected by an earlier pass (the exposure
            // variance prompt re-runs the conversion with it).
            s.auto_crop.then_some(Detected {
                crop: s.crop,
                straighten: s.straighten,
            })
        })
        .collect();
    // `createRoll` only runs for more than one frame.
    let as_roll = as_roll && ok.len() > 1;
    let bases: Vec<[f32; 3]> = ok
        .par_iter()
        .map(|(_, src, s)| pipeline::resolve_base(src, s, None))
        .collect();
    let roll_base = as_roll.then(|| pipeline::roll_base(&bases));
    let analyses: Vec<Analysis> = ok
        .par_iter()
        .zip(bases.par_iter())
        .map(|((_, src, s), base)| {
            let base = if as_roll && s.film_base == crate::engine::settings::FilmBase::Auto {
                pipeline::base_at_exposure(roll_base.unwrap(), *base)
            } else {
                *base
            };
            let neg = pipeline::prepare_negative(src, s, base);
            pipeline::analyze(&neg, s, base, pipeline::frame_ev(&src.info))
        })
        .collect();
    let common_ev = (as_roll && use_exposure)
        .then(|| pipeline::common_ev(&analyses))
        .flatten();
    if let Some(c) = common_ev {
        let spread = pipeline::ev_spread(&analyses, c);
        if spread > 1.0 {
            let items = ok
                .into_iter()
                .map(|(path, src, settings)| ConvertItem {
                    path,
                    proxy: Some(src),
                    settings,
                })
                .collect();
            return JobResult::ExposureVariance { items, spread };
        }
    }
    let roll = as_roll.then(|| {
        let (agg, excluded) = pipeline::aggregate_roll(&analyses, common_ev);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        RollData {
            auto_wb: Some(agg.auto_wb),
            common_ev: agg.common_ev,
            id: format!("roll-{stamp}"),
            levels: agg.levels,
            balance: agg.balance,
            base: roll_base.unwrap(),
            frames: ok.len(),
            excluded: excluded
                .iter()
                .map(|i| {
                    ok[*i]
                        .0
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default()
                })
                .collect(),
        }
    });
    let frames = ok
        .into_iter()
        .zip(analyses)
        .zip(detected)
        .map(|(((p, src, _), a), d)| (p, src, a, d))
        .collect();
    JobResult::Converted {
        frames,
        roll,
        errors,
    }
}
