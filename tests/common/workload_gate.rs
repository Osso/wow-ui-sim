use std::io;
use std::path::Path;

use super::workload_gate_core;
pub(crate) use super::workload_gate_core::Permit;

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Shared,
    Exclusive,
}

pub(crate) fn acquire(mode: Mode) -> io::Result<Permit> {
    let path = workload_gate_core::default_lock_path();
    acquire_at(&path, mode)
}

pub(crate) fn acquire_at(path: &Path, mode: Mode) -> io::Result<Permit> {
    workload_gate_core::acquire_at(path, mode.is_exclusive())
}

pub(crate) fn with_lock<T>(mode: Mode, body: impl FnOnce() -> T) -> T {
    let path = workload_gate_core::default_lock_path();
    with_lock_at(&path, mode, body)
}

pub(crate) fn with_lock_at<T>(path: &Path, mode: Mode, body: impl FnOnce() -> T) -> T {
    workload_gate_core::with_lock_at(path, mode.is_exclusive(), body)
}

impl Mode {
    fn is_exclusive(self) -> bool {
        matches!(self, Self::Exclusive)
    }
}
