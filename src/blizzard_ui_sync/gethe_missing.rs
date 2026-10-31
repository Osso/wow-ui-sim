//! Forever's mapped-but-nonresident UI sources, verified against local CASC CKeys.
use super::{CacheProvenance, DISABLED_VERSION_MISMATCH, EntrySyncResult};
use cascette_crypto::ContentKey;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PIN: &str = include_str!("../../data/blizzard-ui-builds/wowforever.json");

#[derive(Deserialize)]
struct SourcePin {
    version: String,
    gethe_revision: String,
}

struct MappedFile {
    path: String,
    fdid: u32,
    content_key: String,
    resident: bool,
}

struct GetheTree {
    root: PathBuf,
    revision: String,
    version: String,
}

#[derive(Deserialize, Serialize)]
struct Substitution {
    path: String,
    fdid: u32,
    ckey: String,
    gethe_revision: String,
    source: String,
}

pub(super) fn append_source_identity(
    provenance: &mut CacheProvenance,
    installed_version: &str,
    allow_ui_version_mismatch: bool,
) -> crate::Result<()> {
    let pin = read_source_pin()?;
    if installed_version != pin.version && !allow_ui_version_mismatch {
        return Err(crate::Error::Other(format!(
            "UI version mismatch: installed {installed_version}, configured {}. \
             Pass --allow-ui-version-mismatch to use this installation without Gethe fallback.",
            pin.version
        )));
    }
    if installed_version == pin.version {
        provenance.contents = provenance.contents.replace(
            "source=casc-local-or-cdn\nfallback=none\n",
            "source=casc-local-or-gethe-md5\nfallback=none\n",
        );
    } else {
        // The selected installation remains authoritative. An older Gethe pin
        // must not block exact-build CASC reads or supply replacement files.
        provenance.contents.push_str(DISABLED_VERSION_MISMATCH);
        provenance.contents.push('\n');
    }
    provenance.contents.push_str(&format!(
        "gethe_version={}\ngethe_revision={}\n",
        pin.version, pin.gethe_revision
    ));
    Ok(())
}

fn read_source_pin() -> crate::Result<SourcePin> {
    let pin: SourcePin = serde_json::from_str(PIN)
        .map_err(|cause| crate::Error::Other(format!("read Forever source pin: {cause}")))?;
    let revision = &pin.gethe_revision;
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(crate::Error::Other("invalid Forever Gethe revision".into()));
    }
    Ok(pin)
}

fn gethe_fallback_enabled(provenance: &str) -> bool {
    !provenance
        .lines()
        .any(|line| line == DISABLED_VERSION_MISMATCH)
}

pub(super) fn sync_manifest_entry(
    root: &Path,
    entry: &str,
    provenance: &mut CacheProvenance,
) -> crate::Result<EntrySyncResult> {
    let destination = root.join(entry);
    let (result, record) = sync_mapped_file(
        gethe_fallback_enabled(provenance.contents()),
        || read_mapped_file(entry),
        &destination,
        || super::sync_manifest_entry(root, entry),
        load_pinned_tree,
    )?;
    if let Some(record) = record {
        append_substitution(provenance, &record)?;
    }
    Ok(result)
}

fn read_mapped_file(entry: &str) -> crate::Result<Option<MappedFile>> {
    let Some(fdid) = super::manifest_entry_fdid(entry) else {
        return Ok(None);
    };
    let reader = super::casc_path_reader()?;
    let Some(content_key) = reader.root.resolve_fdid(fdid) else {
        return Ok(None);
    };
    let Some(encoding_key) = reader.install.resolver().resolve_content_key(&content_key) else {
        return Ok(None);
    };
    let resident = super::run_casc_async(reader.install.has_encoding_key(&encoding_key));
    Ok(Some(MappedFile {
        path: entry.to_owned(),
        fdid,
        content_key: content_key.to_hex(),
        resident,
    }))
}

fn load_pinned_tree() -> crate::Result<GetheTree> {
    let pin = read_source_pin()?;
    let cache = dirs::cache_dir()
        .ok_or_else(|| crate::Error::Other("cannot determine Gethe cache directory".into()))?;
    Ok(GetheTree {
        root: cache.join("wow-ui-sim/wow-ui-source-git/forever"),
        revision: pin.gethe_revision,
        version: pin.version,
    })
}

fn sync_mapped_file(
    allow_gethe: bool,
    lookup: impl FnOnce() -> crate::Result<Option<MappedFile>>,
    destination: &Path,
    read_local: impl FnOnce() -> crate::Result<EntrySyncResult>,
    load_gethe: impl FnOnce() -> crate::Result<GetheTree>,
) -> crate::Result<(EntrySyncResult, Option<Substitution>)> {
    if !allow_gethe {
        return Ok((read_local()?, None));
    }
    let Some(file) = lookup()? else {
        // No root/encoding mapping retains the existing failure path, not Gethe.
        return Ok((read_local()?, None));
    };
    if file.resident {
        return Ok((read_local()?, None));
    }
    let tree = load_gethe()?;
    let bytes = read_verified_gethe(&tree, &file)?;
    let result = write_verified_file(destination, &bytes)?;
    let record = Substitution {
        path: file.path.clone(),
        fdid: file.fdid,
        ckey: file.content_key.clone(),
        gethe_revision: tree.revision,
        source: "gethe".to_owned(),
    };
    Ok((result, Some(record)))
}

fn read_verified_gethe(tree: &GetheTree, file: &MappedFile) -> crate::Result<Vec<u8>> {
    let version = read_pinned_blob(tree, "version.txt")?;
    let version = std::str::from_utf8(&version)
        .map_err(|cause| crate::Error::Other(format!("read Gethe version.txt: {cause}")))?;
    require_version(&tree.version, version.trim_end_matches(['\r', '\n']))?;
    let bytes = read_pinned_blob(tree, &format!("Interface/AddOns/{}", file.path))?;
    let actual = ContentKey::from_data(&bytes).to_hex();
    if actual != file.content_key {
        return Err(crate::Error::Other(format!(
            "Gethe MD5 mismatch for {} (FDID {}): CKey={}, Gethe={actual}",
            file.path, file.fdid, file.content_key
        )));
    }
    Ok(bytes)
}

fn require_version(expected: &str, actual: &str) -> crate::Result<()> {
    if actual != expected {
        return Err(crate::Error::Other(format!(
            "Gethe version mismatch: expected {expected}, found {actual}"
        )));
    }
    Ok(())
}

fn read_pinned_blob(tree: &GetheTree, path: &str) -> crate::Result<Vec<u8>> {
    let object = format!("{}:{path}", tree.revision);
    let output = std::process::Command::new("git")
        .args(["--no-optional-locks", "-C"])
        .arg(&tree.root)
        .args(["show", &object])
        .output()
        .map_err(|cause| crate::Error::Other(format!("read Gethe {object}: {cause}")))?;
    if !output.status.success() {
        return Err(crate::Error::Other(format!(
            "read Gethe {object} in {}: {}",
            tree.root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}

fn write_verified_file(destination: &Path, bytes: &[u8]) -> crate::Result<EntrySyncResult> {
    match std::fs::read(destination) {
        Ok(existing) if existing == bytes => return Ok(EntrySyncResult::Present),
        Ok(_) => {}
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => {}
        Err(cause) => {
            return Err(crate::Error::Other(format!(
                "read {}: {cause}",
                destination.display()
            )));
        }
    }
    super::write_extracted_casc_path(destination, bytes)?;
    Ok(EntrySyncResult::Extracted)
}

fn append_substitution(
    provenance: &mut CacheProvenance,
    record: &Substitution,
) -> crate::Result<()> {
    let json =
        serde_json::to_string(record).map_err(|error| crate::Error::Other(error.to_string()))?;
    provenance
        .contents
        .push_str(&format!("gethe_file={json}\n"));
    Ok(())
}

pub(super) fn provenance_matches(actual: &str, expected: &str) -> bool {
    if !gethe_fallback_enabled(expected) {
        // A CASC-only cache must not acquire historical Gethe substitutions.
        return actual == expected;
    }
    let Some(records) = actual.strip_prefix(expected) else {
        return false;
    };
    records.lines().all(|line| {
        line.strip_prefix("gethe_file=")
            .is_some_and(|json| serde_json::from_str::<Substitution>(json).is_ok())
    })
}

#[cfg(test)]
#[path = "gethe_missing_tests.rs"]
mod tests;
