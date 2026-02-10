//! Lua bytecode cache for faster subsequent loads.
//!
//! Caches compiled Lua 5.1 bytecode to disk, keyed by content hash.
//! On cache hit, the Lua parser/compiler is skipped entirely — bytecode
//! loads directly into the VM with no parsing overhead.
//!
//! Cache entries are invalidated automatically when file content changes
//! (different hash). Invalid bytecode (Lua version change, corruption)
//! triggers automatic recompilation via the fallback path.
//!
//! Disable with `WOW_SIM_NO_BYTECODE_CACHE=1`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::OnceLock;

const CACHE_DIR: &str = ".cache/lua-bytecode";

/// Check if bytecode caching is disabled via environment variable.
/// Result is cached after first check.
pub fn is_disabled() -> bool {
    static DISABLED: OnceLock<bool> = OnceLock::new();
    *DISABLED.get_or_init(|| {
        std::env::var("WOW_SIM_NO_BYTECODE_CACHE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// Compute a cache key from file content and chunk name.
///
/// Including chunk_name ensures files with identical content but different
/// paths get separate cache entries (Lua 5.1 bytecode embeds the chunk name
/// at compile time).
pub fn content_hash(bytes: &[u8], chunk_name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    chunk_name.hash(&mut hasher);
    hasher.finish()
}

fn cache_path(hash: u64) -> PathBuf {
    PathBuf::from(CACHE_DIR).join(format!("{hash:016x}.luac"))
}

/// Load cached bytecode for the given content hash.
pub fn get(hash: u64) -> Option<Vec<u8>> {
    std::fs::read(cache_path(hash)).ok()
}

/// Save compiled bytecode to cache.
pub fn put(hash: u64, bytecode: &[u8]) {
    let path = cache_path(hash);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, bytecode);
}
