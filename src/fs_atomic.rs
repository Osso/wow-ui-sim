//! Atomic replacement for persisted simulator state.
//!
//! SavedVariables and CVar files are read by other simulator processes while a
//! writer is active. Writing in place lets a reader parse a truncated prefix,
//! so writes go to a sibling temporary file and replace the directory entry.

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(1);

/// Replace `path` with `bytes` so readers see either the previous complete file
/// or the new one. `std::fs::rename` replaces an existing file on Unix and on
/// Windows, where it maps to `MoveFileExW`.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state");
    let temp_id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
    let temp_path = parent.join(format!(".{file_name}.tmp-{}-{temp_id}", std::process::id()));

    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(bytes)?;
        std::fs::rename(&temp_path, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}
