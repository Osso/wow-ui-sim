use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct GeneratedDataRefreshManifest {
    generated_files: Vec<GeneratedFileEntry>,
}

#[derive(Deserialize)]
struct GeneratedFileEntry {
    path: String,
    source: String,
    sha256: String,
    bytes: u64,
    lines: usize,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn manifest_path() -> PathBuf {
    manifest_dir()
        .join("docs")
        .join("wow-client-diff")
        .join("generated_data_refresh_manifest.json")
}

fn read_manifest() -> GeneratedDataRefreshManifest {
    let contents = fs::read_to_string(manifest_path())
        .expect("failed to read generated data refresh manifest");
    serde_json::from_str(&contents).expect("failed to parse generated data refresh manifest")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[test]
fn generated_lua_refresh_manifest_matches_current_file_contents() {
    let manifest = read_manifest();

    assert_eq!(
        manifest.generated_files.len(),
        3,
        "refresh manifest should track every generated artifact explicitly"
    );

    for entry in manifest.generated_files {
        assert!(
            !entry.source.trim().is_empty(),
            "manifest entry for {} must record its source",
            entry.path
        );

        let path = manifest_dir().join(&entry.path);
        let contents =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("failed to read {}", entry.path));
        let bytes = contents.as_bytes();
        let line_count = contents.lines().count();
        let actual_hash = sha256_hex(bytes);

        assert_eq!(
            bytes.len() as u64,
            entry.bytes,
            "byte count changed for {}; update the refresh manifest in the same change",
            entry.path
        );
        assert_eq!(
            line_count, entry.lines,
            "line count changed for {}; update the refresh manifest in the same change",
            entry.path
        );
        assert_eq!(
            actual_hash, entry.sha256,
            "content hash changed for {}; update the refresh manifest in the same change",
            entry.path
        );
    }
}
