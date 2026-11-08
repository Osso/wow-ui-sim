use std::fs;
use std::path::{Path, PathBuf};

use super::saved_variables_parse::{SavedVariablesTableSize, SavedVariablesTableSizeCache};

const TABLE_SIZE_CACHE_DIR: &str = ".saved-variable-table-sizes";
const MAX_TABLE_SIZE_CACHE_BYTES: u64 = 64 * 1024 * 1024;
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub(super) fn load_table_size_cache(
    storage_dir: &Path,
    source_path: &Path,
) -> SavedVariablesTableSizeCache {
    let source_key = table_size_cache_source_key(source_path);
    let cache_path = table_size_cache_path(storage_dir, &source_key);
    if is_oversized_cache_file(&cache_path) {
        return SavedVariablesTableSizeCache::default();
    }
    let Ok(contents) = fs::read_to_string(cache_path) else {
        return SavedVariablesTableSizeCache::default();
    };

    let mut cache = SavedVariablesTableSizeCache::default();
    for line in contents.lines() {
        let Some((cached_source, table_path, size)) = parse_table_size_cache_line(line) else {
            continue;
        };
        if cached_source == source_key {
            cache.insert(table_path, size);
        }
    }
    cache
}

pub(super) fn save_table_size_cache(
    storage_dir: &Path,
    source_path: &Path,
    cache: &SavedVariablesTableSizeCache,
) -> crate::Result<()> {
    let source_key = table_size_cache_source_key(source_path);
    let cache_path = table_size_cache_path(storage_dir, &source_key);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|error| crate::Error::Other(error.to_string()))?;
    }

    let mut lines = Vec::new();
    for (table_path, size) in cache.iter() {
        lines.push(format!(
            "{}\t{}\t{}\t{}",
            escape_table_size_cache_field(&source_key),
            escape_table_size_cache_field(table_path),
            size.array_count,
            size.hash_count
        ));
    }

    let mut contents = lines.join("\n");
    if !contents.is_empty() {
        contents.push('\n');
    }
    crate::fs_atomic::write_atomic(&cache_path, contents.as_bytes())
        .map_err(|error| crate::Error::Other(error.to_string()))
}

fn table_size_cache_path(storage_dir: &Path, source_key: &str) -> PathBuf {
    storage_dir
        .join(TABLE_SIZE_CACHE_DIR)
        .join(format!("{:016x}.tsv", stable_hash(source_key.as_bytes())))
}

fn is_oversized_cache_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.len() > MAX_TABLE_SIZE_CACHE_BYTES)
        .unwrap_or(false)
}

fn table_size_cache_source_key(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

fn stable_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn parse_table_size_cache_line(line: &str) -> Option<(String, String, SavedVariablesTableSize)> {
    let mut fields = line.split('\t');
    let source = unescape_table_size_cache_field(fields.next()?)?;
    let table_path = unescape_table_size_cache_field(fields.next()?)?;
    let array_count = fields.next()?.parse().ok()?;
    let hash_count = fields.next()?.parse().ok()?;
    if fields.next().is_some() {
        return None;
    }
    Some((
        source,
        table_path,
        SavedVariablesTableSize {
            array_count,
            hash_count,
        },
    ))
}

fn escape_table_size_cache_field(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'%' => escaped.push_str("%25"),
            b'\t' => escaped.push_str("%09"),
            b'\n' => escaped.push_str("%0A"),
            b'\r' => escaped.push_str("%0D"),
            _ => escaped.push(byte as char),
        }
    }
    escaped
}

fn unescape_table_size_cache_field(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            output.push(bytes[index]);
            index += 1;
            continue;
        }

        let hex = bytes.get(index + 1..index + 3)?;
        let text = std::str::from_utf8(hex).ok()?;
        let byte = u8::from_str_radix(text, 16).ok()?;
        output.push(byte);
        index += 3;
    }
    String::from_utf8(output).ok()
}

#[cfg(test)]
mod tests {
    use super::super::saved_variables_parse::{
        SavedVariablesTableSize, SavedVariablesTableSizeCache,
    };
    use super::{
        MAX_TABLE_SIZE_CACHE_BYTES, load_table_size_cache, save_table_size_cache,
        table_size_cache_path, table_size_cache_source_key,
    };

    #[test]
    fn cache_file_is_scoped_to_one_saved_variables_source() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("Addon.lua");
        let source_key = table_size_cache_source_key(&source);
        let mut cache = SavedVariablesTableSizeCache::default();
        cache.insert(
            "AddonDB.child",
            SavedVariablesTableSize {
                array_count: 3,
                hash_count: 5,
            },
        );

        save_table_size_cache(dir.path(), &source, &cache).expect("save cache");

        let scoped_path = table_size_cache_path(dir.path(), &source_key);
        assert!(scoped_path.exists());
        assert!(!dir.path().join(".saved-variable-table-sizes.tsv").exists());

        let loaded = load_table_size_cache(dir.path(), &source);
        assert_eq!(
            loaded.get("AddonDB.child"),
            Some(SavedVariablesTableSize {
                array_count: 3,
                hash_count: 5,
            })
        );
    }

    #[test]
    fn legacy_monolithic_cache_is_not_scanned() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("Addon.lua");
        std::fs::write(
            dir.path().join(".saved-variable-table-sizes.tsv"),
            format!(
                "{}\tAddonDB.child\t99\t99\n",
                table_size_cache_source_key(&source)
            ),
        )
        .expect("write legacy monolithic cache");

        let loaded = load_table_size_cache(dir.path(), &source);

        assert!(loaded.get("AddonDB.child").is_none());
    }

    #[test]
    fn oversized_scoped_cache_file_is_ignored() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("Addon.lua");
        let source_key = table_size_cache_source_key(&source);
        let cache_path = table_size_cache_path(dir.path(), &source_key);
        std::fs::create_dir_all(cache_path.parent().expect("cache dir")).expect("cache dir");
        let oversized = std::fs::File::create(&cache_path).expect("cache file");
        oversized
            .set_len(MAX_TABLE_SIZE_CACHE_BYTES + 1)
            .expect("resize cache file");

        let loaded = load_table_size_cache(dir.path(), &source);

        assert!(loaded.get("AddonDB.child").is_none());
    }
}
