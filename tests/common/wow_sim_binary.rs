//! Path to the `wow-sim` executable that belongs to the test build.
//!
//! Cargo sets `CARGO_BIN_EXE_wow-sim` at compile time for every integration
//! test target, so the path always points into the target directory that built
//! the tests. That includes the shared slot target directories used by
//! `scripts/slot-build.py`; a hard-coded `<manifest>/target/debug/wow-sim`
//! does not exist there.

use std::path::PathBuf;

pub fn path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_wow-sim"))
}
