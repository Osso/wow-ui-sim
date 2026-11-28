//! Encoding-key fallbacks for UI assets no longer reachable by path.
//!
//! Some historical UI sheets still appear in the listfile, but the current
//! CASC root does not expose their paths. The FDID resolution cache can also
//! miss those rows on fresh caches, so path-based extraction fails. Keep the
//! same kind of explicit encoding-key fallback used for core fonts.

#[cfg(feature = "casc")]
use std::path::{Path, PathBuf};

#[cfg(feature = "casc")]
use std::sync::OnceLock;

struct KnownCascAsset {
    path: &'static str,
    encoding_key_hex: &'static str,
}

const KNOWN_CASC_ASSETS: &[KnownCascAsset] = &[
    KnownCascAsset {
        path: "interface/common/currencywindow.blp",
        encoding_key_hex: "6DB0A357702C10D71C4F945DA8DC28E5",
    },
    KnownCascAsset {
        path: "interface/framegeneral/uiframemetal2x.blp",
        encoding_key_hex: "729D039CA266E29BD582D7B2244687D1",
    },
    KnownCascAsset {
        path: "interface/framegeneral/uiframemetalhorizontal2x.blp",
        encoding_key_hex: "9AC6043E78C8A6AE72B94E46ED2A7142",
    },
    KnownCascAsset {
        path: "interface/framegeneral/uiframemetalvertical2x.blp",
        encoding_key_hex: "A8F52A3D17E2FD4C08D5C36FB1FF76AC",
    },
    KnownCascAsset {
        path: "interface/framegeneral/uiframetabs.blp",
        encoding_key_hex: "4F12CFB0612E91FDB48E60EA21241B64",
    },
    KnownCascAsset {
        path: "interface/options/optionsexpandlistbutton.blp",
        encoding_key_hex: "9A09A727F4A6AFAA39F29E6AE538FC7B",
    },
    KnownCascAsset {
        path: "interface/paperdollinfoframe/paperdollinfopart1.blp",
        encoding_key_hex: "B8D8BDE4505B8AEFC0D513008C91DDD7",
    },
];

pub fn lookup_encoding_key_hex(path: &str) -> Option<&'static str> {
    let normalized = normalize_path(path);
    KNOWN_CASC_ASSETS
        .iter()
        .find(|asset| asset.path == normalized)
        .map(|asset| asset.encoding_key_hex)
}

#[cfg(feature = "casc")]
struct CascAssetReader {
    install: cascette_client_storage::Installation,
}

#[cfg(feature = "casc")]
static CASC_ASSET_READER: OnceLock<Option<CascAssetReader>> = OnceLock::new();

#[cfg(feature = "casc")]
pub(crate) fn ensure_known_asset_cached(path: &str, out_path: &Path) -> Option<PathBuf> {
    if out_path.exists() {
        return Some(out_path.to_path_buf());
    }

    let encoding_key_hex = lookup_encoding_key_hex(path)?;
    let reader = CASC_ASSET_READER
        .get_or_init(init_casc_asset_reader)
        .as_ref()?;
    let encoding_key = encoding_key_from_hex(encoding_key_hex)?;
    let bytes = run_casc_asset_async(reader.install.read_file_by_encoding_key(&encoding_key))
        .map_err(|err| {
            eprintln!(
                "asset-cache encoding-key fallback failed: {path} ({encoding_key_hex}): {err}"
            );
        })
        .ok()?;

    write_asset(out_path, &bytes).ok()?;
    eprintln!(
        "asset-cache encoding-key fallback: {path} ({encoding_key_hex}) -> {}",
        out_path.display()
    );
    Some(out_path.to_path_buf())
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

#[cfg(feature = "casc")]
fn init_casc_asset_reader() -> Option<CascAssetReader> {
    let data_path = asset_resolver::wow_data_path()?;
    let install = cascette_client_storage::Installation::open(data_path).ok()?;
    run_casc_asset_async(install.initialize()).ok()?;
    Some(CascAssetReader { install })
}

#[cfg(feature = "casc")]
fn run_casc_asset_async<F: std::future::Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("asset CASC runtime")
        .block_on(future)
}

#[cfg(feature = "casc")]
fn encoding_key_from_hex(hex: &str) -> Option<cascette_crypto::EncodingKey> {
    let bytes = parse_hex_16(hex)?;
    Some(cascette_crypto::EncodingKey::from_bytes(bytes))
}

#[cfg(feature = "casc")]
fn parse_hex_16(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }

    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&hex[start..start + 2], 16).ok()?;
    }
    Some(bytes)
}

#[cfg(feature = "casc")]
fn write_asset(out_path: &Path, bytes: &[u8]) -> Result<(), String> {
    sim_texture::resolve::write_cache_file_atomically(out_path, |temporary| {
        std::fs::write(temporary, bytes)
    })
    .map_err(|err| format!("write {}: {err}", out_path.display()))
}

#[cfg(all(test, feature = "casc"))]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn extracted_asset_readers_keep_their_complete_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("asset.blp");
        write_asset(&path, b"first complete asset").unwrap();
        let mut reader = std::fs::File::open(&path).unwrap();
        std::thread::scope(|scope| {
            scope.spawn(|| write_asset(&path, b"second complete asset").unwrap());
        });
        let mut opened = Vec::new();
        reader.read_to_end(&mut opened).unwrap();
        assert_eq!(opened, b"first complete asset");
        assert_eq!(std::fs::read(&path).unwrap(), b"second complete asset");
    }
}
