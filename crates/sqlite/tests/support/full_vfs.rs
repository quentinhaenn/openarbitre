//! Fault VFS for E-ST02-2 (plan T-12, DA F-7): delegates to the default VFS and, per VFS name,
//! meters `xWrite` bytes against a budget (`SQLITE_FULL` once spent) and can fail `xSync`.
//! Test-only unsafe FFI (archi R-2). One VFS per test; its state is leaked (process-global registry).

use std::ffi::{CString, c_int, c_void};
use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use arbitre_sqlite::Options;
use rusqlite::ffi;

struct Inner {
    budget: u64,
    spent: bool,
    refused: usize,
    log: Vec<usize>,
    fail_sync: bool,
}

struct State {
    inner: Mutex<Inner>,
    /// Default VFS, as an address (raw pointers are not `Sync`).
    delegate: usize,
}

impl State {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Handle on one registered fault VFS.
pub struct FullVfs {
    name: String,
    state: &'static State,
}

/// Delegate's `szOsFile`, identical for every fault VFS (same default VFS).
static DELEGATE_SIZE: AtomicUsize = AtomicUsize::new(0);

/// Offset of `Extra` inside a file allocation.
fn extra_offset() -> usize {
    DELEGATE_SIZE.load(Ordering::Relaxed).next_multiple_of(16)
}

impl FullVfs {
    /// Registers `full-<test>-<n>`; the budget starts unlimited.
    pub fn register(test: &str) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let name = format!("full-{test}-{}", N.fetch_add(1, Ordering::Relaxed));
        // SAFETY: a null name asks for the default VFS; the result is only stored.
        let delegate = unsafe { ffi::sqlite3_vfs_find(std::ptr::null()) };
        assert!(!delegate.is_null(), "no default VFS");
        let state: &'static State = Box::leak(Box::new(State {
            inner: Mutex::new(Inner {
                budget: u64::MAX,
                spent: false,
                refused: 0,
                log: Vec::new(),
                fail_sync: false,
            }),
            delegate: delegate as usize,
        }));
        // SAFETY: `delegate` is a live, registered VFS; copied by value.
        let mut vfs = unsafe { *delegate };
        DELEGATE_SIZE.store(usize::try_from(vfs.szOsFile).unwrap(), Ordering::Relaxed);
        vfs.szOsFile = i32::try_from(extra_offset() + size_of::<Extra>()).unwrap();
        vfs.zName = CString::new(name.clone()).unwrap().into_raw();
        vfs.pAppData = std::ptr::from_ref(state).cast_mut().cast();
        vfs.xOpen = Some(x_open);
        vfs.pNext = std::ptr::null_mut();
        // Delegated methods ignore `pVfs`, and the rest is not needed: version 1.
        vfs.iVersion = 1;
        let vfs = Box::leak(Box::new(vfs));
        // SAFETY: `vfs` is leaked, so it outlives every connection using it.
        assert_eq!(unsafe { ffi::sqlite3_vfs_register(vfs, 0) }, ffi::SQLITE_OK);
        Self { name, state }
    }

    pub fn options(&self) -> Options {
        Options {
            vfs: Some(self.name.clone()),
            ..Options::default()
        }
    }

    /// Sets the byte budget and clears the counters and the write log.
    pub fn set_budget(&self, bytes: u64) {
        let mut s = self.state.lock();
        (s.budget, s.spent, s.refused, s.log) = (bytes, false, 0, Vec::new());
    }

    pub fn unlimited(&self) {
        self.set_budget(u64::MAX);
    }

    /// `xWrite` calls refused with `SQLITE_FULL` since the last `set_budget`.
    pub fn refused(&self) -> usize {
        self.state.lock().refused
    }

    /// Lengths of the `xWrite` calls accepted since the last `set_budget`.
    pub fn log(&self) -> Vec<usize> {
        self.state.lock().log.clone()
    }

    pub fn fail_sync(&self, on: bool) {
        self.state.lock().fail_sync = on;
    }
}

/// Per-file data placed after the delegate's file struct: the file's own `io_methods` copy.
#[repr(C)]
struct Extra {
    orig: *const ffi::sqlite3_io_methods,
    state: *const State,
    methods: ffi::sqlite3_io_methods,
}

/// # Safety
/// `file` was opened by `x_open`.
unsafe fn extra(file: *mut ffi::sqlite3_file) -> *mut Extra {
    // SAFETY: the allocation holds `Extra` at this offset (see `register`).
    unsafe { file.cast::<u8>().add(extra_offset()).cast() }
}

unsafe extern "C" fn x_open(
    vfs: *mut ffi::sqlite3_vfs,
    name: ffi::sqlite3_filename,
    file: *mut ffi::sqlite3_file,
    flags: c_int,
    out_flags: *mut c_int,
) -> c_int {
    // SAFETY: SQLite passes our own VFS, whose `pAppData` is the leaked `State`.
    let state: &State = unsafe { &*(*vfs).pAppData.cast::<State>() };
    let delegate = state.delegate as *mut ffi::sqlite3_vfs;
    // SAFETY: `delegate` is the live default VFS.
    let open = unsafe { (*delegate).xOpen.unwrap() };
    // SAFETY: `file` has `szOsFile` bytes (delegate's + `Extra`); the delegate fills its part.
    let rc = unsafe { open(delegate, name, file, flags, out_flags) };
    // SAFETY: reading the `pMethods` the delegate just set.
    let orig = unsafe { (*file).pMethods };
    if rc != ffi::SQLITE_OK || orig.is_null() {
        return rc;
    }
    // SAFETY: `orig` is the delegate's valid method table, copied by value.
    let mut methods = unsafe { *orig };
    methods.xWrite = Some(x_write);
    methods.xSync = Some(x_sync);
    // SAFETY: `Extra` lies inside our allocation; the copy lives as long as the file.
    unsafe {
        let e = extra(file);
        e.write(Extra {
            orig,
            state,
            methods,
        });
        (*file).pMethods = &raw const (*e).methods;
    }
    rc
}

unsafe extern "C" fn x_write(
    file: *mut ffi::sqlite3_file,
    buf: *const c_void,
    amt: c_int,
    offset: ffi::sqlite3_int64,
) -> c_int {
    // SAFETY: SQLite only calls this on files opened by `x_open`.
    let (orig, state) = unsafe { ((*extra(file)).orig, &*(*extra(file)).state) };
    {
        let mut s = state.lock();
        let n = usize::try_from(amt).unwrap();
        if s.spent || n as u64 > s.budget {
            (s.spent, s.refused) = (true, s.refused + 1);
            return ffi::SQLITE_FULL;
        }
        s.budget -= n as u64;
        s.log.push(n);
    }
    // SAFETY: forwards the same arguments to the delegate's `xWrite`.
    unsafe { ((*orig).xWrite.unwrap())(file, buf, amt, offset) }
}

unsafe extern "C" fn x_sync(file: *mut ffi::sqlite3_file, flags: c_int) -> c_int {
    // SAFETY: SQLite only calls this on files opened by `x_open`.
    let (orig, state) = unsafe { ((*extra(file)).orig, &*(*extra(file)).state) };
    if state.lock().fail_sync {
        return ffi::SQLITE_IOERR_FSYNC;
    }
    // SAFETY: forwards the same arguments to the delegate's `xSync`.
    unsafe { ((*orig).xSync.unwrap())(file, flags) }
}
