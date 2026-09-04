use std::io;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::fs::{File, OpenOptions};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

const LOCK_FILE_NAME: &str = "wow-ui-sim-test-workloads.lock";

#[derive(Clone, Copy)]
pub(crate) enum LockMode {
    Shared,
    Exclusive,
}

#[cfg(target_os = "linux")]
pub(crate) struct Permit {
    _local: LocalPermit,
    _file: File,
}

#[cfg(target_os = "linux")]
enum LocalPermit {
    Shared {
        _guard: RwLockReadGuard<'static, ()>,
    },
    Exclusive {
        _guard: RwLockWriteGuard<'static, ()>,
    },
}

#[cfg(not(target_os = "linux"))]
pub(crate) struct Permit;

pub(crate) fn acquire_at(path: &Path, mode: LockMode) -> io::Result<Permit> {
    acquire_at_mode(path, mode)
}

pub(crate) fn with_shared_lock<T>(body: impl FnOnce() -> T) -> T {
    let path = default_lock_path();
    with_lock_at(&path, LockMode::Shared, body)
}

pub(crate) fn with_lock_at<T>(path: &Path, mode: LockMode, body: impl FnOnce() -> T) -> T {
    let _permit = acquire_at_mode(path, mode)
        .unwrap_or_else(|error| panic!("acquire {} workload gate: {error}", mode.description()));
    body()
}

#[cfg(target_os = "linux")]
fn acquire_at_mode(path: &Path, mode: LockMode) -> io::Result<Permit> {
    let local = acquire_local(mode);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    lock(&file, mode)?;
    Ok(Permit {
        _local: local,
        _file: file,
    })
}

#[cfg(not(target_os = "linux"))]
fn acquire_at_mode(_path: &Path, _mode: LockMode) -> io::Result<Permit> {
    Ok(Permit)
}

pub(crate) fn default_lock_path() -> PathBuf {
    std::env::temp_dir().join(LOCK_FILE_NAME)
}

#[cfg(target_os = "linux")]
fn acquire_local(mode: LockMode) -> LocalPermit {
    static LOCAL_GATE: OnceLock<RwLock<()>> = OnceLock::new();
    let gate = LOCAL_GATE.get_or_init(|| RwLock::new(()));
    match mode {
        LockMode::Shared => LocalPermit::Shared {
            _guard: gate.read().unwrap_or_else(|poisoned| poisoned.into_inner()),
        },
        LockMode::Exclusive => LocalPermit::Exclusive {
            _guard: gate
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        },
    }
}

impl LockMode {
    #[cfg(target_os = "linux")]
    fn operation(self) -> libc::c_int {
        match self {
            Self::Shared => libc::LOCK_SH,
            Self::Exclusive => libc::LOCK_EX,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Shared => "shared",
            Self::Exclusive => "exclusive",
        }
    }
}

#[cfg(target_os = "linux")]
fn lock(file: &File, mode: LockMode) -> io::Result<()> {
    loop {
        let result = unsafe { libc::flock(file.as_raw_fd(), mode.operation()) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
