//! Immutable PTR source identity; Blizzard CDN bytes, never a Gethe content fallback.
use super::CacheProvenance;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::OnceLock;

const INDEX: &str = include_str!("../../data/blizzard-ui-builds/ptr.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClientIdentity {
    pub version: String,
    pub build: String,
}

#[derive(Deserialize)]
struct Build {
    schema: u32,
    product: String,
    version: String,
    build_key: String,
    cdn_key: String,
    install_key: String,
    gethe_revision: String,
    #[cfg(feature = "casc")]
    files: Vec<super::pinned_download::ContentEntry>,
}

fn build() -> crate::Result<&'static Build> {
    static BUILD: OnceLock<Result<Build, String>> = OnceLock::new();
    BUILD
        .get_or_init(|| {
            let build: Build = serde_json::from_str(INDEX).map_err(|cause| cause.to_string())?;
            if build.schema != 1 || build.product != "wowxptr" {
                return Err("unsupported PTR content-index schema or product".into());
            }
            #[cfg(feature = "casc")]
            validate_entries(&build)?;
            Ok(build)
        })
        .as_ref()
        .map_err(|cause| crate::Error::Other(cause.clone()))
}

pub(crate) fn client_identity() -> crate::Result<ClientIdentity> {
    let version_and_build = &build()?.version;
    let (version, build) = version_and_build.rsplit_once('.').ok_or_else(|| {
        crate::Error::Other(format!(
            "PTR version has no build suffix: {version_and_build}"
        ))
    })?;
    Ok(ClientIdentity {
        version: version.to_owned(),
        build: build.to_owned(),
    })
}

pub(super) fn provenance() -> crate::Result<CacheProvenance> {
    let build = build()?;
    let manifest_hash = format!("{:x}", Sha256::digest(super::PTR_BLIZZARD_UI_MANIFEST));
    let index_hash = format!("{:x}", Sha256::digest(INDEX));
    Ok(CacheProvenance {
        contents: format!(
            "schema=2\nprofile=ptr\nproduct={}\nversion={}\nbuild_key={}\ninstall_key={}\ncdn_key={}\ngethe_revision={}\nmanifest_sha256={manifest_hash}\ncontent_index_sha256={index_hash}\nsource=blizzard-cdn-pinned\nfallback=none\n",
            build.product,
            build.version,
            build.build_key,
            build.install_key,
            build.cdn_key,
            build.gethe_revision,
        ),
    })
}

pub(super) fn cache_matches(root: &Path) -> bool {
    let Ok(expected) = provenance() else {
        return false;
    };
    std::fs::read_to_string(root.join(super::PROVENANCE_FILE))
        .is_ok_and(|contents| contents == expected.contents())
}

#[cfg(feature = "casc")]
fn validate_entries(build: &Build) -> Result<(), String> {
    let expected: Vec<_> = super::PTR_BLIZZARD_UI_MANIFEST.lines().collect();
    let paths: Vec<_> = build
        .files
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    if paths != expected {
        return Err("PTR content index does not match its path manifest".into());
    }
    for entry in &build.files {
        let safe_path = !entry.path.starts_with('/')
            && entry
                .path
                .split('/')
                .all(|part| !matches!(part, "" | "." | "..") && !part.contains('\\'));
        let valid_keys = [&entry.archive, &entry.content_key, &entry.encoding_key]
            .into_iter()
            .all(|key| key.len() == 32 && key.bytes().all(|b| b.is_ascii_hexdigit()));
        if !safe_path
            || !valid_keys
            || entry.size < 9
            || entry.offset.checked_add(entry.size as u64).is_none()
        {
            return Err(format!("invalid pinned CDN entry: {}", entry.path));
        }
    }
    Ok(())
}

#[cfg(feature = "casc")]
pub(super) fn sync_to(root: &Path) -> crate::Result<super::SyncSummary> {
    let build = build()?;
    let expected = provenance()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|cause| crate::Error::Other(format!("create Blizzard CDN client: {cause}")))?;
    super::invalidate_cache_if_provenance_mismatched(root, &expected)?;
    let workers = 16.min(build.files.len()).max(1);
    let counts = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|worker| {
                let client = &client;
                scope.spawn(move || sync_worker(root, client, &build.files, worker, workers))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| crate::Error::Other("PTR source sync worker panicked".into()))?
            })
            .collect::<crate::Result<Vec<_>>>()
    })?;
    let present = counts.iter().map(|counts| counts.0).sum();
    let extracted = counts.iter().map(|counts| counts.1).sum();
    super::write_complete_marker(root, &expected)?;
    Ok(super::SyncSummary {
        root: root.to_path_buf(),
        total: build.files.len(),
        extracted,
        present,
        missing: 0,
    })
}

#[cfg(feature = "casc")]
fn sync_worker(
    root: &Path,
    client: &reqwest::blocking::Client,
    entries: &[super::pinned_download::ContentEntry],
    worker: usize,
    workers: usize,
) -> crate::Result<(usize, usize)> {
    let mut present = 0;
    let mut extracted = 0;
    for entry in entries.iter().skip(worker).step_by(workers) {
        let destination = root.join(&entry.path);
        match std::fs::read(&destination) {
            Ok(content) if super::pinned_download::content_matches(entry, &content) => {
                present += 1;
                continue;
            }
            Ok(_) => eprintln!("Replacing mismatched PTR source {}", destination.display()),
            Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => {}
            Err(cause) => {
                return Err(crate::Error::Other(format!(
                    "read {}: {cause}",
                    destination.display()
                )));
            }
        }
        let content = super::pinned_download::download_entry(client, entry)?;
        super::write_extracted_casc_path(&destination, &content)?;
        extracted += 1;
    }
    Ok((present, extracted))
}

#[cfg(not(feature = "casc"))]
pub(super) fn sync_to(_root: &Path) -> crate::Result<super::SyncSummary> {
    Err(super::casc_feature_error())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_ptr_rejects_legacy_provenance_and_accepts_exact_identity() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(super::super::PROVENANCE_FILE),
            "schema=1\nprofile=ptr\nproduct=wowt\nversion=12.1.0.69587\n",
        )
        .unwrap();
        assert!(!cache_matches(dir.path()));
        std::fs::write(
            dir.path().join(super::super::PROVENANCE_FILE),
            provenance().unwrap().contents(),
        )
        .unwrap();
        assert!(cache_matches(dir.path()));
    }
}
