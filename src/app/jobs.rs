//! Background work (decoding, analysis, export) on the rayon pool.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
        frames: Vec<ConvertedFrame>,
        roll: Option<RollData>,
        errors: Vec<ConvertError>,
        /// The app asked for it by itself (see `auto.rs`).
        auto: bool,
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
    /// A roll's details for the Projects screen, ahead of its frames.
    RollSummary {
        dir: PathBuf,
        exists: bool,
        info: Option<sidecar::RollInfo>,
        count: usize,
        /// Unix seconds of the newest sidecar.
        edited: Option<u64>,
    },
    /// A few rendered frames of a roll for the Projects screen.
    RollCover {
        dir: PathBuf,
        frames: Vec<egui::ColorImage>,
    },
    Message(String),
    /// From the scanner worker (`scanner.rs`) with that worker's id: events
    /// of a worker the panel has let go are dropped.
    Scan(u64, super::scanner::ScanEvent),
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

    /// For long-lived workers that report through the job channel.
    pub fn sender(&self) -> Sender<JobResult> {
        self.tx.clone()
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

/// The scans a roll folder holds, in frame order.
pub fn folder_scans(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            // `file_type` comes with the listing; `Path::is_file` is a
            // stat per entry, slow on a network drive.
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_ok_and(|t| t.is_file() || t.is_symlink()))
                .map(|e| e.path())
                .filter(|p| {
                    decode::is_folder_scan(p) && !p.to_string_lossy().ends_with("-prepped.tif")
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    files
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
    // Written as native f32, as on this (little-endian) host.
    let mut data = vec![[0.0f32; 3]; h.width * h.height];
    bytemuck::cast_slice_mut::<[f32; 3], u8>(&mut data).copy_from_slice(body);
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
    out.extend_from_slice(bytemuck::cast_slice(&s.data));
    // Write beside and rename, so a reader never sees a half-written file.
    let Some(name) = p.file_name() else { return };
    let tmp = p.with_file_name(format!("{}.tmp", name.to_string_lossy()));
    if std::fs::File::create(&tmp)
        .and_then(|mut f| f.write_all(&out))
        .is_ok()
    {
        let _ = std::fs::rename(&tmp, p);
    }
}

static NETWORK_FOLDER: AtomicBool = AtomicBool::new(false);

/// Note whether the open folder is on a network drive, where waiting on the
/// share, not the CPU, limits loading.
pub fn set_network_folder(dir: &Path) {
    NETWORK_FOLDER.store(is_network_path(dir), Ordering::Relaxed);
}

/// How many proxy decodes to keep in flight.
pub fn proxy_concurrency() -> usize {
    let cpu = (rayon::current_num_threads() / 2).max(2);
    if NETWORK_FOLDER.load(Ordering::Relaxed) {
        cpu.max(6)
    } else {
        cpu
    }
}

#[cfg(unix)]
fn is_network_path(p: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(c) = std::ffi::CString::new(p.as_os_str().as_bytes()) else {
        return false;
    };
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c.as_ptr(), &mut st) } != 0 {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        let name: Vec<u8> = st
            .f_fstypename
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        matches!(
            name.as_slice(),
            b"smbfs" | b"nfs" | b"afpfs" | b"webdav" | b"cifs"
        )
    }
    #[cfg(target_os = "linux")]
    {
        // smb2, cifs, nfs, ncpfs
        matches!(
            st.f_type as u64,
            0xFE53_4D42 | 0xFF53_4D42 | 0x6969 | 0x564C
        )
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = st;
        false
    }
}

#[cfg(not(unix))]
fn is_network_path(p: &Path) -> bool {
    // UNC paths (\\server\share).
    p.to_string_lossy().starts_with(r"\\")
}

/// Load the analysis/thumbnail proxy, using the disk cache when possible.
pub fn load_proxy(path: &Path) -> anyhow::Result<SourceImage> {
    let cache = sidecar::proxy_cache_path(path);
    if let Some(img) = cache.as_deref().and_then(read_cache_hit) {
        return Ok(img);
    }
    let img = decode::load(path, Some(PROXY_SIZE))?;
    if let Some(c) = &cache {
        write_cache(c, &img);
        trim_cache(c.parent(), sidecar::PROXY_EXT, CACHE_CAP);
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

fn editor_cache_path(image: &Path) -> Option<PathBuf> {
    sidecar::proxy_cache_path(image).map(|c| c.with_extension("ed"))
}

/// Load the editor preview, from the local disk cache when possible, so
/// reopening a frame on a network drive does not read the whole file again.
pub fn load_editor(path: &Path) -> anyhow::Result<SourceImage> {
    let cache = editor_cache_path(path);
    if let Some(img) = cache.as_deref().and_then(read_cache_hit) {
        return Ok(img);
    }
    let img = decode::load(path, Some(EDITOR_SIZE))?;
    if let Some(c) = &cache {
        write_cache(c, &img);
        trim_cache(c.parent(), "ed", CACHE_CAP);
    }
    Ok(img)
}

/// Size cap for each kind of file in the proxy cache: analysis proxies
/// (about 8 MB a frame) and editor previews (about 32 MB a frame).
const CACHE_CAP: u64 = 2 << 30;

/// Read a cache file and mark it as used, so trimming drops the least
/// recently used files rather than the least recently written.
fn read_cache_hit(p: &Path) -> Option<SourceImage> {
    let img = read_cache(p)?;
    if let Ok(f) = std::fs::File::options().write(true).open(p) {
        let _ = f.set_modified(std::time::SystemTime::now());
    }
    Some(img)
}

/// Keep the files with extension `ext` in the cache `dir` under `cap` bytes,
/// dropping the least recently used first. Also sweeps what earlier versions
/// left there (proxies named `.json`, grid thumbnails, which now live in each
/// roll's `NLP/`) and temporary files of writes that never finished.
fn trim_cache(dir: Option<&Path>, ext: &str, cap: u64) {
    let Some(rd) = dir.and_then(|d| std::fs::read_dir(d).ok()) else {
        return;
    };
    let now = std::time::SystemTime::now();
    let mut files = Vec::new();
    for e in rd.filter_map(|e| e.ok()) {
        let p = e.path();
        let Some(x) = p.extension().and_then(|x| x.to_str()) else {
            continue;
        };
        let Ok(m) = e.metadata() else { continue };
        let Ok(modified) = m.modified() else { continue };
        let stale_tmp = x == "tmp"
            && now.duration_since(modified).is_ok_and(|d| d.as_secs() > 3600);
        if x == "json" || x == "thumb" || stale_tmp {
            let _ = std::fs::remove_file(&p);
        } else if x == ext {
            files.push((modified, m.len(), p));
        }
    }
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    files.sort();
    for (_, len, p) in files {
        if total <= cap {
            break;
        }
        if std::fs::remove_file(p).is_ok() {
            total -= len;
        }
    }
}

fn thumb_cache_path(image: &Path) -> Option<PathBuf> {
    sidecar::thumb_path(image)
}

/// The cached grid thumbnail source, if any: about 1 MB against the proxy's 8 MB.
pub fn read_thumb(image: &Path) -> Option<SourceImage> {
    read_cache(&thumb_cache_path(image)?)
}

/// The cached proxy, if any, without ever decoding the image.
pub fn read_cached_proxy(image: &Path) -> Option<SourceImage> {
    read_cache_hit(&sidecar::proxy_cache_path(image)?)
}

/// Load the proxy and its thumbnail source, caching the thumbnail if it
/// was not yet.
pub fn load_proxy_and_thumb(path: &Path) -> anyhow::Result<(SourceImage, SourceImage)> {
    let proxy = load_proxy(path)?;
    let thumb = make_thumb(&proxy);
    if let Some(c) = thumb_cache_path(path) {
        if !c.exists() {
            let _ = std::fs::create_dir_all(sidecar::project_dir(path));
            write_cache(&c, &thumb);
            sidecar::remove_stale_thumbs(path, &c);
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
    /// The frame's `Frame::convert_ticket` when this conversion was asked
    /// for; the result applies only while it is still the frame's ticket.
    pub ticket: u64,
    /// An automatic conversion (see `auto.rs`), not one the user asked for.
    pub auto: bool,
}

/// A frame of a conversion that couldn't be read.
pub struct ConvertError {
    pub path: PathBuf,
    pub ticket: u64,
    pub message: String,
}

/// One frame of a finished conversion.
pub struct ConvertedFrame {
    pub path: PathBuf,
    pub ticket: u64,
    pub src: Arc<SourceImage>,
    pub analysis: Analysis,
    /// The frame geometry, for frames converted with auto-crop.
    pub detected: Option<Detected>,
}

/// Analyse frames individually or as a roll. `use_exposure` matches the
/// roll's frames by their EXIF exposure (`commonEV`).
pub fn convert(items: Vec<ConvertItem>, as_roll: bool, use_exposure: bool) -> JobResult {
    use rayon::prelude::*;
    let tickets: HashMap<PathBuf, u64> = items.iter().map(|it| (it.path.clone(), it.ticket)).collect();
    let ticket = |p: &PathBuf| tickets.get(p).copied().unwrap_or_default();
    let auto = !items.is_empty() && items.iter().all(|it| it.auto);
    type Loaded = (PathBuf, Arc<SourceImage>, Settings);
    let loaded: Vec<Result<Loaded, ConvertError>> = items
        .into_par_iter()
        .map(|it| {
            let src = match it.proxy {
                Some(p) => p,
                None => Arc::new(load_proxy(&it.path).map_err(|e| ConvertError {
                    message: format!("{}: {e:#}", it.path.display()),
                    path: it.path.clone(),
                    ticket: it.ticket,
                })?),
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
            auto,
        };
    }
    // Crop before analysing, so the analysis sees only the image. A frame
    // whose detection finds nothing, or something far from the rest, takes
    // the geometry the auto-cropped frames share (`frame::settle`).
    let found: Vec<Option<Option<Detected>>> = ok
        .par_iter()
        .map(|(_, src, s)| wants_auto_crop(s).then(|| auto_crop(src, s)))
        .collect();
    let shapes = frame::roll_shapes(ok.iter().zip(&found).filter_map(|((_, src, s), f)| {
        let d = match f {
            Some(d) => *d,
            None => s.auto_crop.then_some(Detected {
                crop: s.crop,
                straighten: s.straighten,
            }),
        };
        d.map(|d| (frame::is_landscape(src, s), d))
    }));
    let detected: Vec<Option<Detected>> = ok
        .iter_mut()
        .zip(found)
        .map(|((_, src, s), f)| {
            let shared = shapes.of(frame::is_landscape(src, s));
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
                    ticket: ticket(&path),
                    path,
                    proxy: Some(src),
                    settings,
                    auto,
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
        .map(|(((path, src, _), analysis), detected)| ConvertedFrame {
            ticket: ticket(&path),
            path,
            src,
            analysis,
            detected,
        })
        .collect();
    JobResult::Converted {
        frames,
        roll,
        errors,
        auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn file(dir: &Path, name: &str, len: usize, age_secs: u64) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, vec![0u8; len]).unwrap();
        let f = std::fs::File::options().write(true).open(&p).unwrap();
        f.set_modified(SystemTime::now() - Duration::from_secs(age_secs)).unwrap();
        p
    }

    #[test]
    fn cache_trim_drops_least_recently_used_and_legacy_files() {
        let dir = std::env::temp_dir().join(format!("nlp-cache-trim-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let oldest = file(&dir, "a.prx", 100, 400);
        let old = file(&dir, "b.prx", 100, 300);
        let used = file(&dir, "c.prx", 100, 500);
        let new = file(&dir, "d.prx", 100, 10);
        let editor = file(&dir, "a.ed", 1000, 1000);
        let legacy = file(&dir, "e.json", 100, 10);
        let thumb = file(&dir, "f.thumb", 100, 10);
        let stale_tmp = file(&dir, "g.prx.tmp", 100, 7200);
        let fresh_tmp = file(&dir, "h.prx.tmp", 100, 10);

        // A cache hit marks the oldest file as just used (it is not a valid
        // proxy, so touch it the way `read_cache_hit` does).
        std::fs::File::options()
            .write(true)
            .open(&used)
            .unwrap()
            .set_modified(SystemTime::now())
            .unwrap();

        trim_cache(Some(&dir), "prx", 250);
        assert!(!oldest.exists() && !old.exists());
        assert!(used.exists() && new.exists());
        assert!(editor.exists(), "other kinds are trimmed by their own cap");
        assert!(!legacy.exists() && !thumb.exists() && !stale_tmp.exists());
        assert!(fresh_tmp.exists(), "a write in progress is left alone");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
