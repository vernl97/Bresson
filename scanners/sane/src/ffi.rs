// SPDX-License-Identifier: GPL-3.0-or-later
//! The SANE 1.0 C API (`sane/sane.h`) and libsane, loaded at runtime.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use crate::{
    Backend, Constraint, DeviceInfo, Error, Frame, Handle, OptionInfo, OptionType, Parameters,
    Result, Status, Unit, Value,
};

pub type SaneWord = c_int;
pub type SaneHandle = *mut c_void;

#[repr(C)]
pub struct SaneDevice {
    pub name: *const c_char,
    pub vendor: *const c_char,
    pub model: *const c_char,
    pub kind: *const c_char,
}

#[repr(C)]
pub struct SaneRange {
    pub min: SaneWord,
    pub max: SaneWord,
    pub quant: SaneWord,
}

#[repr(C)]
pub struct SaneOptionDescriptor {
    pub name: *const c_char,
    pub title: *const c_char,
    pub desc: *const c_char,
    pub value_type: c_int,
    pub unit: c_int,
    pub size: SaneWord,
    pub cap: SaneWord,
    pub constraint_type: c_int,
    /// Union of `string_list`, `word_list` and `range` pointers.
    pub constraint: *const c_void,
}

#[repr(C)]
#[derive(Default)]
pub struct SaneParameters {
    pub format: c_int,
    pub last_frame: SaneWord,
    pub bytes_per_line: SaneWord,
    pub pixels_per_line: SaneWord,
    pub lines: SaneWord,
    pub depth: SaneWord,
}

const ACTION_GET: c_int = 0;
const ACTION_SET: c_int = 1;

type AuthCallback = Option<unsafe extern "C" fn(*const c_char, *mut c_char, *mut c_char)>;

struct Fns {
    init: unsafe extern "C" fn(*mut SaneWord, AuthCallback) -> c_int,
    get_devices: unsafe extern "C" fn(*mut *const *const SaneDevice, SaneWord) -> c_int,
    open: unsafe extern "C" fn(*const c_char, *mut SaneHandle) -> c_int,
    close: unsafe extern "C" fn(SaneHandle),
    get_option_descriptor:
        unsafe extern "C" fn(SaneHandle, SaneWord) -> *const SaneOptionDescriptor,
    control_option:
        unsafe extern "C" fn(SaneHandle, SaneWord, c_int, *mut c_void, *mut SaneWord) -> c_int,
    get_parameters: unsafe extern "C" fn(SaneHandle, *mut SaneParameters) -> c_int,
    start: unsafe extern "C" fn(SaneHandle) -> c_int,
    read: unsafe extern "C" fn(SaneHandle, *mut u8, SaneWord, *mut SaneWord) -> c_int,
    cancel: unsafe extern "C" fn(SaneHandle),
}

/// libsane with `sane_init` done, once per process ([`LibSane::load`]).
///
/// It is never unloaded and `sane_exit` is never called: the dll backend would
/// `dlclose` the device backends, whose thread-local destructors then run from
/// unmapped code when a thread exits.
struct Lib {
    fns: Fns,
    _lib: libloading::Library,
}

/// The process's libsane, loaded on first use; `Err` says why it isn't available.
static LIB: OnceLock<std::result::Result<Arc<Lib>, String>> = OnceLock::new();

/// SANE isn't thread-safe: every call into libsane holds this lock.
static CALLS: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    CALLS.lock().unwrap_or_else(|e| e.into_inner())
}

/// Where libsane is looked for: `$NLP_SANE_LIB`, then the platform's usual names.
pub fn candidates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::env::var_os("NLP_SANE_LIB")
        .map(PathBuf::from)
        .into_iter()
        .collect();
    if cfg!(target_os = "macos") {
        out.extend(
            [
                "/opt/homebrew/lib/libsane.1.dylib",
                "/usr/local/lib/libsane.1.dylib",
                "libsane.1.dylib",
            ]
            .map(PathBuf::from),
        );
    } else {
        out.extend(
            [
                "libsane.so.1",
                "/usr/lib/x86_64-linux-gnu/libsane.so.1",
                "/usr/lib64/libsane.so.1",
            ]
            .map(PathBuf::from),
        );
    }
    out
}

fn status(call: &'static str, code: c_int) -> Result<()> {
    match Status::from_code(code) {
        Status::Good => Ok(()),
        s => Err(Error::Status { call, status: s }),
    }
}

fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: SANE strings are NUL-terminated and live at least as long as the call.
    unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned()
}

/// The real libsane.
pub struct LibSane {
    lib: Arc<Lib>,
}

// SAFETY: SANE is used from one thread at a time (the scanner worker); the
// types only move to that thread once.
unsafe impl Send for LibSane {}

impl LibSane {
    /// The process's libsane: loaded and `sane_init`ed on the first call, shared after.
    pub fn load() -> Result<Self> {
        match LIB.get_or_init(Self::load_once) {
            Ok(lib) => Ok(Self { lib: lib.clone() }),
            Err(e) => Err(Error::Unavailable(e.clone())),
        }
    }

    fn load_once() -> std::result::Result<Arc<Lib>, String> {
        let mut tried = Vec::new();
        for path in candidates() {
            // SAFETY: loading libsane runs its initialisers, which have no preconditions.
            match unsafe { libloading::Library::new(&path) } {
                Ok(lib) => return Self::init_library(lib).map_err(|e| e.to_string()),
                Err(e) => tried.push(format!("{}: {e}", path.display())),
            }
        }
        Err(format!("libsane not found ({})", tried.join("; ")))
    }

    fn init_library(lib: libloading::Library) -> Result<Arc<Lib>> {
        // SAFETY: the symbols have the SANE 1.0 signatures declared in `Fns`.
        let fns = unsafe {
            macro_rules! sym {
                ($name:literal) => {
                    *lib.get($name).map_err(|e| {
                        Error::Unavailable(format!(
                            "libsane lacks {}: {e}",
                            String::from_utf8_lossy($name)
                        ))
                    })?
                };
            }
            Fns {
                init: sym!(b"sane_init"),
                get_devices: sym!(b"sane_get_devices"),
                open: sym!(b"sane_open"),
                close: sym!(b"sane_close"),
                get_option_descriptor: sym!(b"sane_get_option_descriptor"),
                control_option: sym!(b"sane_control_option"),
                get_parameters: sym!(b"sane_get_parameters"),
                start: sym!(b"sane_start"),
                read: sym!(b"sane_read"),
                cancel: sym!(b"sane_cancel"),
            }
        };
        let mut version: SaneWord = 0;
        let _g = lock();
        // SAFETY: valid out pointer; no authorisation callback.
        status("sane_init", unsafe { (fns.init)(&mut version, None) })?;
        Ok(Arc::new(Lib { fns, _lib: lib }))
    }
}

impl Backend for LibSane {
    fn devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let _g = lock();
        let mut list: *const *const SaneDevice = std::ptr::null();
        // SAFETY: valid out pointer; the list stays valid until the next call.
        status("sane_get_devices", unsafe {
            (self.lib.fns.get_devices)(&mut list, 0)
        })?;
        let mut out = Vec::new();
        let mut i = 0;
        while !list.is_null() {
            // SAFETY: a NULL-terminated array of device pointers.
            let d = unsafe { *list.add(i) };
            if d.is_null() {
                break;
            }
            // SAFETY: non-null device record.
            let d = unsafe { &*d };
            out.push(DeviceInfo {
                name: cstr(d.name),
                vendor: cstr(d.vendor),
                model: cstr(d.model),
                kind: cstr(d.kind),
            });
            i += 1;
        }
        Ok(out)
    }

    fn open(&mut self, name: &str) -> Result<Box<dyn Handle>> {
        let c = CString::new(name).map_err(|_| Error::Invalid(format!("device name {name:?}")))?;
        let mut h: SaneHandle = std::ptr::null_mut();
        let _g = lock();
        // SAFETY: valid name and out pointer.
        status("sane_open", unsafe {
            (self.lib.fns.open)(c.as_ptr(), &mut h)
        })?;
        Ok(Box::new(LibHandle {
            lib: self.lib.clone(),
            h,
        }))
    }
}

struct LibHandle {
    lib: Arc<Lib>,
    h: SaneHandle,
}

// SAFETY: see `LibSane`.
unsafe impl Send for LibHandle {}

impl Drop for LibHandle {
    fn drop(&mut self) {
        let _g = lock();
        // SAFETY: open handle, closed once.
        unsafe { (self.lib.fns.close)(self.h) }
    }
}

impl LibHandle {
    fn raw(&self, i: i32) -> Option<&SaneOptionDescriptor> {
        // SAFETY: SANE returns NULL past the last option; the descriptor lives until options reload.
        unsafe { (self.lib.fns.get_option_descriptor)(self.h, i).as_ref() }
    }
}

impl Handle for LibHandle {
    fn option_count(&mut self) -> Result<i32> {
        let _g = lock();
        let mut n: SaneWord = 0;
        let mut info = 0;
        // SAFETY: option 0 is the option count, an Int.
        status("sane_control_option", unsafe {
            (self.lib.fns.control_option)(
                self.h,
                0,
                ACTION_GET,
                (&mut n as *mut SaneWord).cast(),
                &mut info,
            )
        })?;
        Ok(n)
    }

    fn descriptor(&mut self, i: i32) -> Option<OptionInfo> {
        let _g = lock();
        let d = self.raw(i)?;
        let value_type = OptionType::from_code(d.value_type);
        let words = (d.size as usize / std::mem::size_of::<SaneWord>()).max(1);
        // SAFETY: the union member matches `constraint_type`; lists end with NULL
        // (strings) or start with their length (words).
        let constraint = unsafe {
            match d.constraint_type {
                1 if !d.constraint.is_null() => {
                    let r = &*(d.constraint as *const SaneRange);
                    Constraint::Range {
                        min: r.min,
                        max: r.max,
                        quant: r.quant,
                    }
                }
                2 if !d.constraint.is_null() => {
                    let w = d.constraint as *const SaneWord;
                    let n = *w as usize;
                    Constraint::WordList((1..=n).map(|k| *w.add(k)).collect())
                }
                3 if !d.constraint.is_null() => {
                    let s = d.constraint as *const *const c_char;
                    let mut v = Vec::new();
                    let mut k = 0;
                    while !(*s.add(k)).is_null() {
                        v.push(cstr(*s.add(k)));
                        k += 1;
                    }
                    Constraint::StringList(v)
                }
                _ => Constraint::None,
            }
        };
        Some(OptionInfo {
            index: i,
            name: cstr(d.name),
            title: cstr(d.title),
            value_type,
            unit: Unit::from_code(d.unit),
            size: d.size.max(0) as usize,
            words,
            cap: d.cap as u32,
            constraint,
        })
    }

    fn get(&mut self, opt: &OptionInfo) -> Result<Value> {
        let mut buf = vec![0u8; opt.size.max(4)];
        let mut info = 0;
        let _g = lock();
        // SAFETY: the buffer holds the option's declared size.
        status("sane_control_option", unsafe {
            (self.lib.fns.control_option)(
                self.h,
                opt.index,
                ACTION_GET,
                buf.as_mut_ptr().cast(),
                &mut info,
            )
        })?;
        let word = SaneWord::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
        Ok(match opt.value_type {
            OptionType::Bool => Value::Bool(word != 0),
            OptionType::Int => Value::Int(word),
            OptionType::Fixed => Value::Fixed(crate::fixed_to_f64(word)),
            OptionType::String => Value::Str(cstr(buf.as_ptr().cast())),
            _ => Value::Button,
        })
    }

    fn set(&mut self, opt: &OptionInfo, value: &Value) -> Result<u32> {
        let mut buf = vec![0u8; opt.size.max(4)];
        match value {
            Value::Bool(b) => buf[..4].copy_from_slice(&(*b as SaneWord).to_ne_bytes()),
            Value::Int(v) => buf[..4].copy_from_slice(&v.to_ne_bytes()),
            Value::Fixed(v) => buf[..4].copy_from_slice(&crate::f64_to_fixed(*v).to_ne_bytes()),
            Value::Str(s) => {
                let b = s.as_bytes();
                let n = b.len().min(buf.len().saturating_sub(1));
                buf[..n].copy_from_slice(&b[..n]);
                buf[n] = 0;
            }
            Value::Button => {}
        }
        let mut info = 0;
        let _g = lock();
        // SAFETY: the buffer holds the option's declared size.
        status("sane_control_option", unsafe {
            (self.lib.fns.control_option)(
                self.h,
                opt.index,
                ACTION_SET,
                buf.as_mut_ptr().cast(),
                &mut info,
            )
        })?;
        Ok(info as u32)
    }

    fn parameters(&mut self) -> Result<Parameters> {
        let mut p = SaneParameters::default();
        let _g = lock();
        // SAFETY: valid out pointer.
        status("sane_get_parameters", unsafe {
            (self.lib.fns.get_parameters)(self.h, &mut p)
        })?;
        Ok(Parameters {
            format: Frame::from_code(p.format),
            last_frame: p.last_frame != 0,
            bytes_per_line: p.bytes_per_line.max(0) as usize,
            pixels_per_line: p.pixels_per_line.max(0) as usize,
            lines: (p.lines >= 0).then_some(p.lines as usize),
            depth: p.depth.max(0) as u32,
        })
    }

    fn start(&mut self) -> Result<()> {
        let _g = lock();
        // SAFETY: open handle.
        status("sane_start", unsafe { (self.lib.fns.start)(self.h) })
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut n: SaneWord = 0;
        let max = buf.len().min(i32::MAX as usize) as SaneWord;
        let _g = lock();
        // SAFETY: `buf` holds `max` bytes.
        status("sane_read", unsafe {
            (self.lib.fns.read)(self.h, buf.as_mut_ptr(), max, &mut n)
        })?;
        Ok(n.max(0) as usize)
    }

    fn cancel(&mut self) {
        let _g = lock();
        // SAFETY: open handle.
        unsafe { (self.lib.fns.cancel)(self.h) }
    }
}
