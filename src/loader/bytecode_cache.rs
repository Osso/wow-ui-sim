//! Lua bytecode cache for faster subsequent loads.
//!
//! Stores compiled Lua 5.1 bytecode on disk, keyed by content hash. Warm
//! startup can then skip reparsing and recompiling loader chunks entirely.
//!
//! Pack headers validate both the locked Rilua compiler revision and the
//! simulator's slot/whitelist ABI. Compiler updates invalidate persisted code
//! automatically, including corrections that leave the bytecode format unchanged.
//! Unknown-compiler loose files and legacy cache keys are never imported.

use crate::lua_api::hot_literals::WHITELIST_VERSION;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

const PACK_FILE: &str = "pack.bin";
// Version 3 adds the compiler identity to the existing whitelist ABI header.
const PACK_MAGIC: &[u8; 8] = b"WOWBC003";
const RILUA_COMPILER_REVISION: &str = env!("WOW_SIM_RILUA_COMPILER_REVISION");
const PACK_VERSION_END: usize = PACK_MAGIC.len() + 4;
const PACK_HEADER_LEN: usize = PACK_VERSION_END + RILUA_COMPILER_REVISION.len();
const PACK_ENTRY_HEADER_LEN: usize = 8 + 4;
const MAX_PACK_SIZE: u64 = 768 * 1024 * 1024;
static NEXT_TEMP_PACK_ID: AtomicU64 = AtomicU64::new(1);
static BYTECODE_CACHE_MODE: AtomicU8 = AtomicU8::new(CacheMode::WRITABLE);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CacheMode {
    Writable,
    ParentBypass,
    ReadOnly,
}

impl CacheMode {
    const WRITABLE: u8 = 0;
    const PARENT_BYPASS: u8 = 1;
    const READ_ONLY: u8 = 2;

    fn current() -> Self {
        match BYTECODE_CACHE_MODE.load(Ordering::Acquire) {
            Self::WRITABLE => Self::Writable,
            Self::PARENT_BYPASS => Self::ParentBypass,
            Self::READ_ONLY => Self::ReadOnly,
            value => panic!("invalid bytecode cache mode {value}"),
        }
    }

    fn allows_reads(self) -> bool {
        self != Self::ParentBypass
    }

    fn allows_writes(self) -> bool {
        self == Self::Writable
    }
}

pub(crate) fn enter_parent_bypass_mode() {
    BYTECODE_CACHE_MODE.store(CacheMode::PARENT_BYPASS, Ordering::Release);
}

pub(crate) fn enter_read_only_mode() {
    BYTECODE_CACHE_MODE.store(CacheMode::READ_ONLY, Ordering::Release);
}

pub(crate) fn release_prefork_parent_memory() -> Result<usize, String> {
    if is_disabled() {
        return Ok(0);
    }

    let mut state = cache_state()
        .lock()
        .map_err(|_| "release prefork bytecode cache memory: cache lock poisoned".to_string())?;
    if CacheMode::current() == CacheMode::ParentBypass {
        return seal_parent_bypass_state(&mut state);
    }
    release_loaded_pack_memory(&mut state)
}

fn seal_parent_bypass_state(state: &mut CacheState) -> Result<usize, String> {
    if state.initialized || !state.values.is_empty() || !state.index.is_empty() {
        return Err(
            "release prefork bytecode cache memory: parent bypass started after cache use"
                .to_string(),
        );
    }

    state.initialized = true;
    state.pack_exists = pack_path().is_some_and(|path| path.is_file());
    Ok(0)
}

fn release_loaded_pack_memory(state: &mut CacheState) -> Result<usize, String> {
    if !state.initialized {
        return Err("release prefork bytecode cache memory: cache is not initialized".to_string());
    }
    if !state.pack_exists {
        return Err("release prefork bytecode cache memory: pack is not loaded".to_string());
    }
    if state.values.is_empty() {
        return Err(
            "release prefork bytecode cache memory: in-memory pack was already released"
                .to_string(),
        );
    }

    let released_bytes = state.values.len();
    state.values = Vec::new();
    state.index = HashMap::new();
    Ok(released_bytes)
}

#[derive(Default)]
struct CacheState {
    initialized: bool,
    pack_exists: bool,
    values: Vec<u8>,
    index: HashMap<u64, (usize, usize)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PutResult {
    Stored,
    Unchanged,
    Skipped,
    Failed,
}

fn cache_state() -> &'static Mutex<CacheState> {
    static STATE: OnceLock<Mutex<CacheState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(CacheState::default()))
}

/// Check if bytecode caching is disabled.
/// Result is cached after first check.
pub fn is_disabled() -> bool {
    static DISABLED: OnceLock<bool> = OnceLock::new();
    *DISABLED.get_or_init(|| {
        if let Ok(enable) = std::env::var("WOW_SIM_ENABLE_BYTECODE_CACHE") {
            return !(enable == "1" || enable.eq_ignore_ascii_case("true"));
        }

        std::env::var("WOW_SIM_DISABLE_BYTECODE_CACHE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// Compute a cache key from file content and chunk name.
pub fn content_hash(bytes: &[u8], chunk_name: &str) -> u64 {
    content_hash_for_compiler(bytes, chunk_name, RILUA_COMPILER_REVISION)
}

fn content_hash_for_compiler(bytes: &[u8], chunk_name: &str, revision: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    chunk_name.hash(&mut hasher);
    WHITELIST_VERSION.hash(&mut hasher);
    revision.hash(&mut hasher);
    hasher.finish()
}

fn cache_dir() -> Option<PathBuf> {
    Some(
        dirs::cache_dir()?
            .join("wow-ui-sim")
            .join("lua-bytecode")
            .join(cache_namespace()),
    )
}

fn cache_namespace() -> String {
    cache_namespace_for_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn cache_namespace_for_manifest(manifest_dir: &str) -> String {
    let mut hasher = DefaultHasher::new();
    manifest_dir.hash(&mut hasher);
    format!("worktree-{:016x}", hasher.finish())
}

fn pack_path() -> Option<PathBuf> {
    Some(cache_dir()?.join(PACK_FILE))
}

/// Load cached bytecode and pass it to a callback.
///
/// Exact-key hits borrow directly from a compiler-validated pack.
pub fn with_cached_bytecode<R>(hash: u64, callback: impl FnOnce(&[u8]) -> R) -> Option<R> {
    let mode = CacheMode::current();
    if !mode.allows_reads() {
        return None;
    }

    let mut state = cache_state().lock().ok()?;
    let path = pack_path();
    ensure_loaded(&mut state, mode, path.as_deref());
    with_cached_bytecode_from_state(&state, hash, callback)
}

/// Save compiled bytecode to cache.
pub fn put(hash: u64, bytecode: &[u8]) -> PutResult {
    let mode = CacheMode::current();
    if !mode.allows_writes() {
        return PutResult::Skipped;
    }

    let mut state = match cache_state().lock() {
        Ok(state) => state,
        Err(_) => return PutResult::Failed,
    };
    let path = pack_path();
    ensure_loaded(&mut state, mode, path.as_deref());

    let Some(path) = path else {
        return PutResult::Failed;
    };
    store_entry_at_path_with_max(&mut state, &path, MAX_PACK_SIZE, mode, hash, bytecode)
}

fn ensure_loaded(state: &mut CacheState, mode: CacheMode, pack: Option<&Path>) {
    if state.initialized {
        return;
    }
    if let Some(pack) = pack {
        state.pack_exists = load_pack_from_path(state, pack, mode);
    }
    state.initialized = true;
}

fn load_pack_from_path(state: &mut CacheState, pack: &Path, mode: CacheMode) -> bool {
    load_pack_from_path_with_max(state, pack, MAX_PACK_SIZE, mode)
}

fn load_pack_from_path_with_max(
    state: &mut CacheState,
    pack: &Path,
    max_pack_size: u64,
    mode: CacheMode,
) -> bool {
    let Some((file, bytes)) = read_pack_within_limit(pack, max_pack_size, mode) else {
        return false;
    };
    let original_len = bytes.len();
    let Some(valid_len) = load_pack_bytes(state, bytes) else {
        // Unknown format, slot ABI, or compiler revision: never replay its chunks.
        // Remove it so the next write starts a clean file.
        drop(file);
        if mode.allows_writes() {
            let _ = std::fs::remove_file(pack);
        }
        return false;
    };

    if valid_len < original_len && mode.allows_writes() {
        drop(file);
        let _ = truncate_pack(pack, valid_len);
    }

    true
}

fn read_pack_within_limit(
    pack: &Path,
    max_pack_size: u64,
    mode: CacheMode,
) -> Option<(std::fs::File, Vec<u8>)> {
    let mut file = std::fs::File::open(pack).ok()?;
    if file.metadata().ok()?.len() > max_pack_size {
        remove_rejected_pack(file, pack, mode);
        return None;
    }

    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(max_pack_size.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > max_pack_size {
        remove_rejected_pack(file, pack, mode);
        return None;
    }
    Some((file, bytes))
}

fn remove_rejected_pack(file: std::fs::File, pack: &Path, mode: CacheMode) {
    drop(file);
    if mode.allows_writes() {
        let _ = std::fs::remove_file(pack);
    }
}

fn with_cached_bytecode_from_state<R>(
    state: &CacheState,
    hash: u64,
    callback: impl FnOnce(&[u8]) -> R,
) -> Option<R> {
    let (offset, len) = state.index.get(&hash).copied()?;
    Some(callback(&state.values[offset..offset + len]))
}

fn load_pack_bytes(state: &mut CacheState, mut bytes: Vec<u8>) -> Option<usize> {
    if bytes.len() < PACK_HEADER_LEN || &bytes[..PACK_MAGIC.len()] != PACK_MAGIC {
        return None;
    }
    let version_bytes: [u8; 4] = bytes[PACK_MAGIC.len()..PACK_VERSION_END]
        .try_into()
        .expect("pack whitelist ABI is four bytes");
    let compiler_revision = &bytes[PACK_VERSION_END..PACK_HEADER_LEN];
    if u32::from_le_bytes(version_bytes) != WHITELIST_VERSION
        || compiler_revision != RILUA_COMPILER_REVISION.as_bytes()
    {
        return None;
    }

    let mut index = HashMap::new();

    let mut pos = PACK_HEADER_LEN;
    while pos + PACK_ENTRY_HEADER_LEN <= bytes.len() {
        let entry_start = pos;
        let hash = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
        pos += 8;
        let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        if pos + len > bytes.len() {
            bytes.truncate(entry_start);
            state.values = bytes;
            state.index = index;
            return Some(entry_start);
        }
        index.insert(hash, (pos, len));
        pos += len;
    }

    bytes.truncate(pos);
    state.values = bytes;
    state.index = index;
    Some(pos)
}

fn store_entry_at_path_with_max(
    state: &mut CacheState,
    path: &Path,
    max_pack_size: u64,
    mode: CacheMode,
    hash: u64,
    bytecode: &[u8],
) -> PutResult {
    if !mode.allows_writes() {
        return PutResult::Skipped;
    }
    if cached_bytes_equal(state, hash, bytecode) {
        return PutResult::Unchanged;
    }

    let Some(new_pack_len) = single_entry_pack_len(bytecode.len()) else {
        return PutResult::Failed;
    };
    if new_pack_len > max_pack_size {
        return reject_oversized_entry(state, path);
    }

    let new_pack = single_entry_pack(hash, bytecode);
    if !state.pack_exists {
        return persist_replacement(state, path, new_pack);
    }

    let Some(projected_len) = read_projected_append_len(path, bytecode.len()) else {
        return PutResult::Failed;
    };
    if projected_len <= max_pack_size {
        return append_and_update_state(state, path, hash, bytecode);
    }

    rewrite_full_pack_for_entry(state, path, max_pack_size, hash, bytecode, new_pack)
}

fn reject_oversized_entry(state: &mut CacheState, path: &Path) -> PutResult {
    let _ = rewrite_and_replace_state(state, path, empty_pack());
    PutResult::Failed
}

fn rewrite_full_pack_for_entry(
    state: &mut CacheState,
    path: &Path,
    max_pack_size: u64,
    hash: u64,
    bytecode: &[u8],
    new_pack: Vec<u8>,
) -> PutResult {
    let compacted_len = compacted_pack_len_with_entry(state, hash, bytecode.len());
    if compacted_len.is_some_and(|len| len <= max_pack_size) {
        let compacted = compacted_pack_with_entry(state, hash, bytecode);
        return persist_replacement(state, path, compacted);
    }
    persist_replacement(state, path, new_pack)
}

fn cached_bytes_equal(state: &CacheState, hash: u64, bytecode: &[u8]) -> bool {
    state
        .index
        .get(&hash)
        .copied()
        .is_some_and(|(offset, len)| state.values[offset..offset + len] == *bytecode)
}

fn read_projected_append_len(path: &Path, bytecode_len: usize) -> Option<u64> {
    let current_len = std::fs::metadata(path).ok()?.len();
    current_len.checked_add(serialized_entry_len(bytecode_len)?)
}

fn single_entry_pack_len(bytecode_len: usize) -> Option<u64> {
    (PACK_HEADER_LEN as u64).checked_add(serialized_entry_len(bytecode_len)?)
}

fn serialized_entry_len(bytecode_len: usize) -> Option<u64> {
    u64::try_from(bytecode_len)
        .ok()?
        .checked_add(PACK_ENTRY_HEADER_LEN as u64)
}

fn compacted_pack_len_with_entry(
    state: &CacheState,
    hash: u64,
    bytecode_len: usize,
) -> Option<u64> {
    state
        .index
        .iter()
        .filter(|(existing_hash, _)| **existing_hash != hash)
        .try_fold(
            single_entry_pack_len(bytecode_len)?,
            |total, (_, (_, len))| total.checked_add(serialized_entry_len(*len)?),
        )
}

fn append_and_update_state(
    state: &mut CacheState,
    path: &Path,
    hash: u64,
    bytecode: &[u8],
) -> PutResult {
    let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(path) else {
        return PutResult::Failed;
    };
    let Ok(original_len) = file.metadata().map(|metadata| metadata.len()) else {
        return PutResult::Failed;
    };
    if write_pack_entry(&mut file, hash, bytecode).is_err() {
        drop(file);
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .and_then(|file| file.set_len(original_len));
        return PutResult::Failed;
    }

    let offset = state.values.len();
    state.values.extend_from_slice(bytecode);
    state.index.insert(hash, (offset, bytecode.len()));
    PutResult::Stored
}

fn persist_replacement(state: &mut CacheState, path: &Path, bytes: Vec<u8>) -> PutResult {
    match rewrite_and_replace_state(state, path, bytes) {
        Ok(()) => PutResult::Stored,
        Err(_) => PutResult::Failed,
    }
}

fn rewrite_and_replace_state(
    state: &mut CacheState,
    path: &Path,
    bytes: Vec<u8>,
) -> std::io::Result<()> {
    atomic_write_pack(path, &bytes)?;
    replace_state_with_pack(state, bytes);
    Ok(())
}

fn replace_state_with_pack(state: &mut CacheState, bytes: Vec<u8>) {
    let initialized = state.initialized;
    let mut replacement = CacheState::default();
    load_pack_bytes(&mut replacement, bytes).expect("new cache pack must be structurally valid");
    replacement.initialized = initialized;
    replacement.pack_exists = true;
    *state = replacement;
}

fn compacted_pack_with_entry(state: &CacheState, hash: u64, bytecode: &[u8]) -> Vec<u8> {
    let mut hashes: Vec<u64> = state
        .index
        .keys()
        .copied()
        .filter(|existing_hash| *existing_hash != hash)
        .collect();
    hashes.sort_unstable();

    let mut bytes = empty_pack();
    for existing_hash in hashes {
        let (offset, len) = state.index[&existing_hash];
        append_entry_bytes(
            &mut bytes,
            existing_hash,
            &state.values[offset..offset + len],
        );
    }
    append_entry_bytes(&mut bytes, hash, bytecode);
    bytes
}

fn single_entry_pack(hash: u64, bytecode: &[u8]) -> Vec<u8> {
    let mut bytes = empty_pack();
    append_entry_bytes(&mut bytes, hash, bytecode);
    bytes
}

fn empty_pack() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(PACK_HEADER_LEN);
    bytes.extend_from_slice(PACK_MAGIC);
    bytes.extend_from_slice(&WHITELIST_VERSION.to_le_bytes());
    bytes.extend_from_slice(RILUA_COMPILER_REVISION.as_bytes());
    bytes
}

fn append_entry_bytes(bytes: &mut Vec<u8>, hash: u64, bytecode: &[u8]) {
    bytes.extend_from_slice(&hash.to_le_bytes());
    bytes.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    bytes.extend_from_slice(bytecode);
}

fn atomic_write_pack(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;
    let temp_id = NEXT_TEMP_PACK_ID.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or(PACK_FILE);
    let temp_path = parent.join(format!(".{file_name}.tmp-{}-{temp_id}", std::process::id()));

    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp_path, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    result
}

fn truncate_pack(path: &Path, len: usize) -> std::io::Result<()> {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)?
        .set_len(len as u64)
}

fn write_pack_entry(file: &mut std::fs::File, hash: u64, bytecode: &[u8]) -> std::io::Result<()> {
    file.write_all(&hash.to_le_bytes())?;
    file.write_all(&(bytecode.len() as u32).to_le_bytes())?;
    file.write_all(bytecode)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Arbitrary sentinel hash used in the load-path tests. Any u64 will
    /// do; the value only exists so we can assert the entry landed at
    /// the expected offset/length in the in-memory cache state.
    const SENTINEL_HASH: u64 = 0xdead_beef_cafe_babe;

    fn synth_pack_bytes(magic: &[u8; 8], version: u32, entries: &[(u64, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(magic);
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(RILUA_COMPILER_REVISION.as_bytes());
        for (hash, bytecode) in entries {
            buf.extend_from_slice(&hash.to_le_bytes());
            buf.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
            buf.extend_from_slice(bytecode);
        }
        buf
    }

    fn pack_header_bytes() -> Vec<u8> {
        synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[])
    }

    fn serialized_pack_len(entries: &[&[u8]]) -> u64 {
        (PACK_HEADER_LEN
            + entries
                .iter()
                .map(|entry| PACK_ENTRY_HEADER_LEN + entry.len())
                .sum::<usize>()) as u64
    }

    fn assert_cached_bytes(state: &CacheState, hash: u64, expected: &[u8]) {
        let (offset, len) = state
            .index
            .get(&hash)
            .copied()
            .expect("expected hash in cache index");
        assert_eq!(&state.values[offset..offset + len], expected);
    }

    fn compile_cache_fixture(value: &str) -> Vec<u8> {
        use rilua::LuaApiMut;
        let mut lua = rilua::Lua::new().unwrap();
        let source = format!("return {value:?}");
        let function = LuaApiMut::load_bytes(&mut lua, source.as_bytes(), "@cache-fixture")
            .expect("compile fixture");
        crate::loader::bytecode::dump_function(lua.state_mut(), &function).unwrap()
    }

    fn replay_cache_fixture(state: &mut CacheState, hash: u64) -> Option<String> {
        use rilua::LuaApiMut;
        with_cached_bytecode_from_state(state, hash, |bytes| {
            let mut lua = rilua::Lua::new().unwrap();
            let function = LuaApiMut::load_bytes(&mut lua, bytes, "@cache-fixture").unwrap();
            let values = lua.call_function(&function, &[]).unwrap();
            String::from_utf8(lua.val_as_bytes(values[0]).unwrap().to_vec()).unwrap()
        })
    }

    #[test]
    fn compiler_identity_rejects_unversioned_pack_without_replay_or_readonly_mutation() {
        let (_dir, path) = temp_pack_path("compiler-unversioned");
        let bytecode = compile_cache_fixture("stale compiler executed");
        let hash = content_hash(b"fixture", "@cache-fixture");
        let mut old_pack = b"WOWBC002".to_vec();
        old_pack.extend_from_slice(&WHITELIST_VERSION.to_le_bytes());
        append_entry_bytes(&mut old_pack, hash, &bytecode);
        std::fs::write(&path, old_pack).unwrap();
        let before = snapshot_file(&path);

        let mut reopened = CacheState::default();
        let loaded = load_pack_from_path(&mut reopened, &path, CacheMode::ReadOnly);
        let replay = loaded
            .then(|| replay_cache_fixture(&mut reopened, hash))
            .flatten();
        assert_eq!(replay, None, "unknown compiler bytecode must never execute");
        assert!(!loaded);
        assert_eq!(snapshot_file(&path), before);
    }

    #[test]
    fn compiler_identity_rejects_different_compiler_on_reopen_and_replaces_on_write() {
        let (_dir, path) = temp_pack_path("compiler-mismatch");
        let hash = content_hash(b"fixture", "@cache-fixture");
        let stale = compile_cache_fixture("stale compiler executed");
        let mut pack = single_entry_pack(hash, &stale);
        // The previous format has no compiler identity; it must also miss.
        pack[PACK_MAGIC.len() + 4..PACK_HEADER_LEN].fill(b'0');
        std::fs::write(&path, pack).unwrap();
        let before = snapshot_file(&path);

        let mut readonly = CacheState::default();
        let loaded = load_pack_from_path(&mut readonly, &path, CacheMode::ReadOnly);
        let replay = loaded
            .then(|| replay_cache_fixture(&mut readonly, hash))
            .flatten();
        assert_eq!(replay, None, "foreign compiler chunk must not execute");
        assert_eq!(snapshot_file(&path), before);

        let mut writable = CacheState::default();
        assert!(!load_pack_from_path(
            &mut writable,
            &path,
            CacheMode::Writable
        ));
        let fresh = compile_cache_fixture("fresh compiler executed");
        assert_eq!(
            store_entry_at_path_with_max(
                &mut writable,
                &path,
                4096,
                CacheMode::Writable,
                hash,
                &fresh
            ),
            PutResult::Stored
        );
        let mut reopened = CacheState::default();
        assert!(load_pack_from_path(
            &mut reopened,
            &path,
            CacheMode::ReadOnly
        ));
        assert_eq!(
            replay_cache_fixture(&mut reopened, hash).as_deref(),
            Some("fresh compiler executed")
        );
    }

    #[test]
    fn compiler_identity_same_compiler_warm_reopen_preserves_stored_bytes() {
        let (_dir, path) = temp_pack_path("compiler-warm");
        let hash = content_hash(b"fixture", "@cache-fixture");
        let bytecode = compile_cache_fixture("same compiler warm hit");
        let mut writer = CacheState::default();
        assert_eq!(
            store_entry_at_path_with_max(
                &mut writer,
                &path,
                4096,
                CacheMode::Writable,
                hash,
                &bytecode
            ),
            PutResult::Stored
        );
        let before = snapshot_file(&path);
        let mut reader = CacheState::default();
        assert!(load_pack_from_path(&mut reader, &path, CacheMode::ReadOnly));
        assert_cached_bytes(&reader, hash, &bytecode);
        assert_eq!(
            replay_cache_fixture(&mut reader, hash).as_deref(),
            Some("same compiler warm hit")
        );
        assert_eq!(snapshot_file(&path), before);
    }

    #[test]
    fn compiler_identity_ignores_unversioned_loose_bytecode() {
        let (dir, path) = temp_pack_path("compiler-loose");
        let hash = content_hash(b"fixture", "@cache-fixture");
        let loose = dir.path().join(format!("{hash:016x}.luac"));
        std::fs::write(&loose, compile_cache_fixture("unversioned loose chunk")).unwrap();
        let before = snapshot_file(&loose);
        let mut reader = CacheState::default();
        ensure_loaded(&mut reader, CacheMode::ReadOnly, Some(&path));
        assert_eq!(replay_cache_fixture(&mut reader, hash), None);
        assert_eq!(snapshot_file(&loose), before);
        assert!(!path.exists());
    }

    #[test]
    fn load_pack_bytes_accepts_current_version_header() {
        let bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(SENTINEL_HASH, b"xyz")]);
        let expected_len = bytes.len();
        let payload_offset = PACK_HEADER_LEN + 8 + 4;
        let mut state = CacheState::default();
        assert_eq!(load_pack_bytes(&mut state, bytes), Some(expected_len));
        assert_eq!(
            state.index.get(&SENTINEL_HASH).copied(),
            Some((payload_offset, 3))
        );
        assert_eq!(&state.values[payload_offset..payload_offset + 3], b"xyz");
    }

    #[test]
    fn load_pack_bytes_indexes_payloads_in_place() {
        let bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(SENTINEL_HASH, b"xyz")]);
        let expected_len = bytes.len();
        let payload_offset = PACK_HEADER_LEN + 8 + 4;
        let mut state = CacheState::default();

        assert_eq!(load_pack_bytes(&mut state, bytes), Some(expected_len));
        assert_eq!(
            state.index.get(&SENTINEL_HASH).copied(),
            Some((payload_offset, 3))
        );
        assert_eq!(&state.values[payload_offset..payload_offset + 3], b"xyz");
    }

    #[test]
    fn load_pack_bytes_rejects_mismatched_whitelist_version() {
        let stale_version = WHITELIST_VERSION.wrapping_add(1);
        let bytes = synth_pack_bytes(PACK_MAGIC, stale_version, &[(1, b"z")]);
        let mut state = CacheState::default();
        assert_eq!(load_pack_bytes(&mut state, bytes), None);
        assert!(state.index.is_empty());
        assert!(state.values.is_empty());
    }

    #[test]
    fn load_pack_bytes_rejects_legacy_wowbc001_magic() {
        // Packs written before the version header must be discarded on
        // load so they don't get re-interpreted against the new layout.
        let legacy_magic = *b"WOWBC001";
        let bytes = synth_pack_bytes(&legacy_magic, WHITELIST_VERSION, &[(1, b"z")]);
        let mut state = CacheState::default();
        assert_eq!(load_pack_bytes(&mut state, bytes), None);
    }

    #[test]
    fn load_pack_bytes_rejects_truncated_header() {
        // File shorter than magic + version — can't even check the
        // version. Reject rather than crash on the slice try_into.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(PACK_MAGIC);
        bytes.extend_from_slice(&[0u8; 3]); // only 3 of 4 version bytes
        let mut state = CacheState::default();
        assert_eq!(load_pack_bytes(&mut state, bytes), None);
    }

    #[test]
    fn load_pack_bytes_keeps_valid_prefix_before_torn_entry() {
        let mut bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(SENTINEL_HASH, b"xyz")]);
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(b"partial");

        let mut state = CacheState::default();
        assert_eq!(
            load_pack_bytes(&mut state, bytes),
            Some(PACK_HEADER_LEN + PACK_ENTRY_HEADER_LEN + 3)
        );
        let payload_offset = PACK_HEADER_LEN + 8 + 4;
        assert_eq!(
            state.index.get(&SENTINEL_HASH).copied(),
            Some((payload_offset, 3))
        );
        assert_eq!(&state.values[payload_offset..payload_offset + 3], b"xyz");
    }

    #[test]
    fn bounded_store_rejects_oversized_pack_before_payload_read() {
        let (_temp_dir, path) = temp_pack_path("oversized-preflight");
        let mut file = std::fs::File::create(&path).expect("create sparse pack");
        file.write_all(&pack_header_bytes())
            .expect("write valid pack header");
        file.set_len(65)
            .expect("extend sparse pack beyond test limit");

        let mut state = CacheState::default();
        assert!(!load_pack_from_path_with_max(
            &mut state,
            &path,
            64,
            CacheMode::Writable
        ));
        assert!(state.index.is_empty());
        assert!(state.values.is_empty());
        assert!(!path.exists());
    }

    #[test]
    fn bounded_store_compacts_stale_entries_before_adding_new_entry() {
        let stale_hash = 1;
        let retained_hash = 2;
        let new_hash = 3;
        let bytes = synth_pack_bytes(
            PACK_MAGIC,
            WHITELIST_VERSION,
            &[
                (stale_hash, b"stale-value"),
                (stale_hash, b"fresh-value"),
                (retained_hash, b"retained"),
            ],
        );
        let (_temp_dir, path) = temp_pack_path("bounded-compact");
        std::fs::write(&path, &bytes).expect("write stale pack");
        let mut state = CacheState::default();
        load_pack_bytes(&mut state, bytes).expect("load stale pack state");
        state.pack_exists = true;

        let max = serialized_pack_len(&[
            b"fresh-value".as_slice(),
            b"retained".as_slice(),
            b"new".as_slice(),
        ]);
        assert_eq!(
            store_entry_at_path_with_max(
                &mut state,
                &path,
                max,
                CacheMode::Writable,
                new_hash,
                b"new",
            ),
            PutResult::Stored
        );

        assert_eq!(std::fs::metadata(&path).unwrap().len(), max);
        assert_cached_bytes(&state, stale_hash, b"fresh-value");
        assert_cached_bytes(&state, retained_hash, b"retained");
        assert_cached_bytes(&state, new_hash, b"new");
    }

    #[test]
    fn bounded_store_rebuilds_with_only_new_entry_when_unique_set_exceeds_limit() {
        let bytes = synth_pack_bytes(
            PACK_MAGIC,
            WHITELIST_VERSION,
            &[(1, b"first-value"), (2, b"second-value")],
        );
        let (_temp_dir, path) = temp_pack_path("bounded-rebuild");
        std::fs::write(&path, &bytes).expect("write existing pack");
        let mut state = CacheState::default();
        load_pack_bytes(&mut state, bytes).expect("load existing pack state");
        state.pack_exists = true;

        let max = serialized_pack_len(&[b"new-value".as_slice()]);
        assert_eq!(
            store_entry_at_path_with_max(
                &mut state,
                &path,
                max,
                CacheMode::Writable,
                3,
                b"new-value",
            ),
            PutResult::Stored
        );

        assert_eq!(std::fs::metadata(&path).unwrap().len(), max);
        assert_eq!(state.index.len(), 1);
        assert_cached_bytes(&state, 3, b"new-value");
    }

    #[test]
    fn bounded_store_rejects_entry_larger_than_pack_limit() {
        let bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(1, b"old")]);
        let (_temp_dir, path) = temp_pack_path("bounded-oversized-entry");
        std::fs::write(&path, &bytes).expect("write existing pack");
        let mut state = CacheState::default();
        load_pack_bytes(&mut state, bytes).expect("load existing pack state");
        state.pack_exists = true;

        let max = (PACK_HEADER_LEN + PACK_ENTRY_HEADER_LEN + b"oversized".len() - 1) as u64;
        assert_eq!(
            store_entry_at_path_with_max(
                &mut state,
                &path,
                max,
                CacheMode::Writable,
                2,
                b"oversized",
            ),
            PutResult::Failed
        );

        assert_eq!(std::fs::read(&path).unwrap(), pack_header_bytes());
        assert_eq!(state.values, pack_header_bytes());
        assert!(state.index.is_empty());
        assert!(state.pack_exists);
    }

    #[test]
    fn bounded_store_failed_append_leaves_in_memory_state_unchanged() {
        let bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(1, b"old")]);
        let dir = tempfile::tempdir().expect("temp directory");
        let path = dir.path();
        let mut state = CacheState::default();
        load_pack_bytes(&mut state, bytes).expect("load existing pack state");
        state.pack_exists = true;
        let original_values = state.values.clone();
        let original_index = state.index.clone();
        let original_pack_exists = state.pack_exists;

        assert_eq!(
            store_entry_at_path_with_max(&mut state, path, 1024, CacheMode::Writable, 2, b"new",),
            PutResult::Failed
        );

        assert_eq!(state.values, original_values);
        assert_eq!(state.index, original_index);
        assert_eq!(state.pack_exists, original_pack_exists);
    }

    fn temp_pack_path(test_name: &str) -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("wow-ui-sim-{test_name}-"))
            .tempdir()
            .expect("create temporary pack directory");
        let path = temp_dir.path().join(PACK_FILE);
        (temp_dir, path)
    }

    #[derive(Debug, PartialEq, Eq)]
    struct FileSnapshot {
        bytes: Vec<u8>,
        len: u64,
        modified: std::time::SystemTime,
        readonly: bool,
    }

    fn snapshot_file(path: &Path) -> FileSnapshot {
        let metadata = std::fs::metadata(path).expect("read cache file metadata");
        FileSnapshot {
            bytes: std::fs::read(path).expect("read cache file bytes"),
            len: metadata.len(),
            modified: metadata
                .modified()
                .expect("read cache file modification time"),
            readonly: metadata.permissions().readonly(),
        }
    }

    fn directory_entry_names(path: &Path) -> Vec<std::ffi::OsString> {
        let mut names = std::fs::read_dir(path)
            .expect("read cache directory")
            .map(|entry| entry.expect("read cache directory entry").file_name())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn read_only_invalid_pack_is_not_removed() {
        let (_temp_dir, path) = temp_pack_path("read-only-invalid");
        std::fs::write(&path, b"invalid-pack").expect("write invalid pack");
        let before = snapshot_file(&path);
        let entries_before = directory_entry_names(path.parent().unwrap());
        let mut state = CacheState::default();

        assert!(!load_pack_from_path_with_max(
            &mut state,
            &path,
            64,
            CacheMode::ReadOnly,
        ));

        assert_eq!(snapshot_file(&path), before);
        assert_eq!(
            directory_entry_names(path.parent().unwrap()),
            entries_before
        );
    }

    #[test]
    fn read_only_oversized_pack_is_not_removed() {
        let (_temp_dir, path) = temp_pack_path("read-only-oversized");
        let mut file = std::fs::File::create(&path).expect("create oversized pack");
        file.write_all(&pack_header_bytes())
            .expect("write pack header");
        file.set_len(65).expect("extend pack beyond test limit");
        drop(file);
        let before = snapshot_file(&path);
        let entries_before = directory_entry_names(path.parent().unwrap());
        let mut state = CacheState::default();

        assert!(!load_pack_from_path_with_max(
            &mut state,
            &path,
            64,
            CacheMode::ReadOnly,
        ));

        assert_eq!(snapshot_file(&path), before);
        assert_eq!(
            directory_entry_names(path.parent().unwrap()),
            entries_before
        );
    }

    #[test]
    fn read_only_torn_pack_is_not_truncated() {
        let mut bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(1, b"complete")]);
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        bytes.extend_from_slice(&16_u32.to_le_bytes());
        bytes.extend_from_slice(b"partial");
        let (_temp_dir, path) = temp_pack_path("read-only-torn");
        std::fs::write(&path, &bytes).expect("write torn pack");
        let before = snapshot_file(&path);
        let mut state = CacheState::default();

        assert!(load_pack_from_path_with_max(
            &mut state,
            &path,
            1024,
            CacheMode::ReadOnly,
        ));

        assert_cached_bytes(&state, 1, b"complete");
        assert_eq!(snapshot_file(&path), before);
    }

    #[test]
    fn compiler_identity_foreign_keys_in_current_pack_never_replay_or_promote() {
        let current_hash = content_hash(b"fixture", "@cache-fixture");
        let foreign_hash = content_hash_for_compiler(b"fixture", "@cache-fixture", &"0".repeat(40));
        assert_ne!(current_hash, foreign_hash);
        let stale = compile_cache_fixture("foreign compiler key executed");
        let bytes = single_entry_pack(foreign_hash, &stale);
        let (_dir, path) = temp_pack_path("compiler-foreign-key");
        std::fs::write(&path, bytes).unwrap();
        let before = snapshot_file(&path);
        for mode in [CacheMode::ReadOnly, CacheMode::Writable] {
            let mut state = CacheState::default();
            ensure_loaded(&mut state, mode, Some(&path));
            assert_eq!(replay_cache_fixture(&mut state, current_hash), None);
            assert!(!state.index.contains_key(&current_hash));
            assert_eq!(snapshot_file(&path), before);
        }
    }

    #[test]
    fn read_only_legacy_files_are_not_migrated() {
        let (temp_dir, pack) = temp_pack_path("read-only-legacy-migration");
        let legacy = temp_dir.path().join("0000000000000001.luac");
        std::fs::write(&legacy, b"legacy-bytecode").expect("write legacy cache entry");
        let legacy_before = snapshot_file(&legacy);
        let entries_before = directory_entry_names(temp_dir.path());
        let mut state = CacheState::default();

        ensure_loaded(&mut state, CacheMode::ReadOnly, Some(&pack));

        assert!(!pack.exists());
        assert_eq!(snapshot_file(&legacy), legacy_before);
        assert_eq!(directory_entry_names(temp_dir.path()), entries_before);
        assert!(state.index.is_empty());
        assert!(!state.pack_exists);
    }

    #[test]
    fn read_only_store_skips_append_replacement_and_temp_paths() {
        let (_missing_dir, missing_path) = temp_pack_path("read-only-missing-store");
        let missing_entries = directory_entry_names(missing_path.parent().unwrap());
        let mut missing_state = CacheState::default();
        assert_eq!(
            store_entry_at_path_with_max(
                &mut missing_state,
                &missing_path,
                1024,
                CacheMode::ReadOnly,
                1,
                b"new",
            ),
            PutResult::Skipped
        );
        assert!(!missing_path.exists());
        assert_eq!(
            directory_entry_names(missing_path.parent().unwrap()),
            missing_entries
        );

        let existing_bytes = synth_pack_bytes(PACK_MAGIC, WHITELIST_VERSION, &[(1, b"old")]);
        let (_existing_dir, existing_path) = temp_pack_path("read-only-existing-store");
        std::fs::write(&existing_path, &existing_bytes).expect("write existing pack");
        let existing_before = snapshot_file(&existing_path);
        let existing_entries = directory_entry_names(existing_path.parent().unwrap());
        let mut existing_state = CacheState::default();
        load_pack_bytes(&mut existing_state, existing_bytes).expect("load existing pack state");
        existing_state.pack_exists = true;

        assert_eq!(
            store_entry_at_path_with_max(
                &mut existing_state,
                &existing_path,
                serialized_pack_len(&[b"replacement".as_slice()]),
                CacheMode::ReadOnly,
                2,
                b"replacement",
            ),
            PutResult::Skipped
        );
        assert_eq!(snapshot_file(&existing_path), existing_before);
        assert_eq!(
            directory_entry_names(existing_path.parent().unwrap()),
            existing_entries
        );
        assert_cached_bytes(&existing_state, 1, b"old");
        assert!(!existing_state.index.contains_key(&2));
    }

    #[test]
    fn content_hash_changes_with_whitelist_version() {
        let base = content_hash(b"abc", "=@chunk");
        let mut hasher = DefaultHasher::new();
        b"abc".hash(&mut hasher);
        "=@chunk".hash(&mut hasher);
        WHITELIST_VERSION.wrapping_add(1).hash(&mut hasher);
        let stale = hasher.finish();
        assert_ne!(base, stale);
    }

    #[test]
    fn cache_namespace_distinguishes_parallel_worktrees() {
        let retail = cache_namespace_for_manifest("/syncthing/Sync/Projects/wow/wow-ui-sim");
        let classic =
            cache_namespace_for_manifest("/syncthing/Sync/Projects/wow/wow-ui-sim-classic");

        assert_ne!(retail, classic);
        assert!(retail.starts_with("worktree-"));
        assert!(classic.starts_with("worktree-"));
    }

    #[test]
    fn max_pack_size_allows_full_addon_warm_cache() {
        let full_addon_pack_budget = 512 * 1024 * 1024;

        assert!(
            MAX_PACK_SIZE >= full_addon_pack_budget,
            "full addon cache observed at 454 MiB; cap must keep it reusable"
        );
    }

    #[test]
    fn put_reports_stored_and_unchanged_entries() {
        let (_dir, path) = temp_pack_path("unchanged-entry");
        let hash = content_hash(b"source", "=@put-test");
        let mut state = CacheState::default();
        assert_eq!(
            store_entry_at_path_with_max(
                &mut state,
                &path,
                1024,
                CacheMode::Writable,
                hash,
                b"compiled"
            ),
            PutResult::Stored
        );
        let before = snapshot_file(&path);
        assert_eq!(
            store_entry_at_path_with_max(
                &mut state,
                &path,
                1024,
                CacheMode::Writable,
                hash,
                b"compiled"
            ),
            PutResult::Unchanged
        );
        assert_eq!(snapshot_file(&path), before);
    }

    #[test]
    fn with_cached_bytecode_borrows_current_hits() {
        let mut state = CacheState::default();
        state.values.extend_from_slice(b"current-bytecode");
        state.index.insert(SENTINEL_HASH, (0, state.values.len()));
        let pack_ptr = state.values.as_ptr();

        let borrowed = with_cached_bytecode_from_state(&state, SENTINEL_HASH, |bytes| {
            bytes.as_ptr() == pack_ptr
        })
        .expect("current cache hit should call callback");

        assert!(
            borrowed,
            "current cache hits should pass a borrowed slice from the in-memory pack"
        );
    }
}
