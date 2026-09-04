use std::io;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use std::fs::{File, OpenOptions};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

const LOCK_FILE_NAME: &str = "wow-ui-sim-test-workloads.lock";

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

pub(crate) fn acquire_at(path: &Path, exclusive: bool) -> io::Result<Permit> {
    acquire_at_mode(path, exclusive)
}

pub(crate) fn default_lock_path() -> PathBuf {
    std::env::temp_dir().join(LOCK_FILE_NAME)
}

pub(crate) fn with_shared_lock<T>(body: impl FnOnce() -> T) -> T {
    let path = default_lock_path();
    with_lock_at(&path, false, body)
}

pub(crate) fn with_lock_at<T>(path: &Path, exclusive: bool, body: impl FnOnce() -> T) -> T {
    let _permit = acquire_at(path, exclusive).unwrap_or_else(|error| {
        panic!(
            "acquire {} workload gate: {error}",
            workload_description(exclusive)
        )
    });
    body()
}

#[cfg(target_os = "linux")]
fn acquire_at_mode(path: &Path, exclusive: bool) -> io::Result<Permit> {
    let local = acquire_local(exclusive);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;
    lock(&file, exclusive)?;
    Ok(Permit {
        _local: local,
        _file: file,
    })
}

#[cfg(not(target_os = "linux"))]
fn acquire_at_mode(_path: &Path, _exclusive: bool) -> io::Result<Permit> {
    Ok(Permit)
}

#[cfg(target_os = "linux")]
fn acquire_local(exclusive: bool) -> LocalPermit {
    static LOCAL_GATE: OnceLock<RwLock<()>> = OnceLock::new();
    let gate = LOCAL_GATE.get_or_init(|| RwLock::new(()));
    if exclusive {
        LocalPermit::Exclusive {
            _guard: gate
                .write()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    } else {
        LocalPermit::Shared {
            _guard: gate.read().unwrap_or_else(|poisoned| poisoned.into_inner()),
        }
    }
}

fn workload_description(exclusive: bool) -> &'static str {
    if exclusive {
        "exclusive"
    } else {
        "shared"
    }
}

#[cfg(target_os = "linux")]
fn lock(file: &File, exclusive: bool) -> io::Result<()> {
    let operation = if exclusive {
        libc::LOCK_EX
    } else {
        libc::LOCK_SH
    };
    loop {
        let result = unsafe { libc::flock(file.as_raw_fd(), operation) };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}
