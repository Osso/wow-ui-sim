//! CASC-backed Blizzard UI source synchronization.

mod pinned;
#[cfg(feature = "casc")]
mod pinned_download;
mod profile_cache;

use self::profile_cache::{cache_entry_is_usable, required_profile_cache_entries};
#[cfg(feature = "casc")]
use cascette_client_storage::BuildInfoFile;
#[cfg(feature = "casc")]
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
#[cfg(feature = "casc")]
use std::sync::OnceLock;

const RETAIL_BLIZZARD_UI_MANIFEST: &str = include_str!("../data/blizzard-ui-files/retail.txt");
const PTR_BLIZZARD_UI_MANIFEST: &str = include_str!("../data/blizzard-ui-files/ptr.txt");
const WRATH_BLIZZARD_UI_MANIFEST: &str = include_str!("../data/blizzard-ui-files/wrath.txt");
const MISTS_BLIZZARD_UI_MANIFEST: &str = include_str!("../data/blizzard-ui-files/mists.txt");
const ERA_BLIZZARD_UI_MANIFEST: &str = include_str!("../data/blizzard-ui-files/era.txt");
const ANNIVERSARY_BLIZZARD_UI_MANIFEST: &str =
    include_str!("../data/blizzard-ui-files/anniversary.txt");
const WOWFOREVER_BLIZZARD_UI_MANIFEST: &str =
    include_str!("../data/blizzard-ui-files/wowforever.txt");
const COMPLETE_MARKER: &str = ".wow-ui-sim-blizzard-ui-complete";
const PROVENANCE_FILE: &str = ".wow-ui-sim-blizzard-ui-provenance";
const PROVENANCE_SCHEMA: &str = "1";
#[cfg(feature = "casc")]
static CASC_CONFIGURED: OnceLock<bool> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncSummary {
    pub root: PathBuf,
    pub total: usize,
    pub extracted: usize,
    pub present: usize,
    pub missing: usize,
}

pub fn default_cache_addons_path() -> crate::Result<PathBuf> {
    dirs::cache_dir()
        .map(|dir| {
            dir.join("wow-ui-sim/blizzard-ui")
                .join(crate::client_profile::ACTIVE.cache_subdir())
                .join("AddOns")
        })
        .ok_or_else(|| crate::Error::Other("could not determine user cache directory".to_string()))
}

#[cfg(feature = "client-ptr")]
pub(crate) fn ptr_client_identity() -> crate::Result<pinned::ClientIdentity> {
    pinned::client_identity()
}

pub fn cached_blizzard_ui_addons_path() -> Option<PathBuf> {
    let path = default_cache_addons_path().ok()?;
    let is_complete = path.join(COMPLETE_MARKER).is_file();
    let identity_matches = crate::client_profile::ACTIVE
        != crate::client_profile::ClientProfile::Ptr
        || pinned::cache_matches(&path);
    (is_complete && identity_matches && cache_has_required_profile_files(&path)).then_some(path)
}

fn cache_has_required_profile_files(root: &Path) -> bool {
    required_profile_cache_entries().iter().all(|entry| {
        let path = root.join(entry);
        path.is_file() && cache_entry_is_usable(entry, &path)
    })
}

pub fn sync_blizzard_ui() -> crate::Result<SyncSummary> {
    let root = default_cache_addons_path()?;
    sync_blizzard_ui_to(&root)
}

pub fn sync_blizzard_ui_to(root: &Path) -> crate::Result<SyncSummary> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        return pinned::sync_to(root);
    }
    let expected_provenance = expected_cache_provenance()?;
    invalidate_cache_if_provenance_mismatched(root, &expected_provenance)?;
    sync_blizzard_ui_entries(root, sync_manifest_entries(), &expected_provenance)
}

pub fn manifest_entries() -> impl Iterator<Item = &'static str> {
    active_blizzard_ui_manifest()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
}

fn active_blizzard_ui_manifest() -> &'static str {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => RETAIL_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::Ptr => PTR_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::Wrath => WRATH_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::Mists => MISTS_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::Era => ERA_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::Anniversary => ANNIVERSARY_BLIZZARD_UI_MANIFEST,
        crate::client_profile::ClientProfile::WowForever => WOWFOREVER_BLIZZARD_UI_MANIFEST,
    }
}

fn sync_manifest_entries() -> impl Iterator<Item = &'static str> {
    manifest_entries().filter(|entry| profile_cache::sync_entry_belongs_to_active_profile(entry))
}

fn sync_blizzard_ui_entries<'a>(
    root: &Path,
    entries: impl Iterator<Item = &'a str>,
    expected_provenance: &CacheProvenance,
) -> crate::Result<SyncSummary> {
    #[cfg(feature = "casc")]
    if !casc_available() {
        return Err(crate::Error::WowInstallNotFound);
    }

    let mut summary = SyncSummary {
        root: root.to_path_buf(),
        total: 0,
        extracted: 0,
        present: 0,
        missing: 0,
    };
    let mut last_missing_entry: Option<String> = None;

    for entry in entries {
        summary.total += 1;
        match sync_manifest_entry(root, entry)? {
            EntrySyncResult::Present => summary.present += 1,
            EntrySyncResult::Extracted => summary.extracted += 1,
            EntrySyncResult::Missing => {
                summary.missing += 1;
                last_missing_entry = Some(entry.to_string());
            }
        }
    }

    if summary.missing > 0 {
        return Err(crate::Error::BlizzardUiPartial {
            missing: summary.missing,
            total: summary.total,
            last_error: last_missing_entry
                .unwrap_or_else(|| "unknown extraction failure".to_string()),
        });
    }

    write_complete_marker(root, expected_provenance)?;
    Ok(summary)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheProvenance {
    contents: String,
}

impl CacheProvenance {
    fn new(
        profile: &str,
        product: &str,
        version: &str,
        build_key: &str,
        install_key: &str,
        manifest_sha256: &str,
    ) -> Self {
        let contents = format!(
            "schema={PROVENANCE_SCHEMA}\nprofile={profile}\nproduct={product}\nversion={version}\nbuild_key={build_key}\ninstall_key={install_key}\nmanifest_sha256={manifest_sha256}\nsource=casc-local-or-cdn\nfallback=none\n"
        );
        Self { contents }
    }

    fn contents(&self) -> &str {
        &self.contents
    }
}

#[cfg(feature = "casc")]
struct BuildIdentity {
    version: String,
    build_key: String,
    install_key: String,
}

fn expected_cache_provenance() -> crate::Result<CacheProvenance> {
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr {
        return pinned::provenance();
    }
    #[cfg(feature = "casc")]
    {
        let install_root =
            asset_resolver::wow_install_path().ok_or(crate::Error::WowInstallNotFound)?;
        let product = crate::asset_resolver_config::active_profile_casc_product();
        let build_identity = read_active_build_identity(install_root, product)?;
        let manifest_sha256 = format!("{:x}", Sha256::digest(active_blizzard_ui_manifest()));
        return Ok(CacheProvenance::new(
            crate::client_profile::ACTIVE.cache_subdir(),
            product,
            &build_identity.version,
            &build_identity.build_key,
            &build_identity.install_key,
            &manifest_sha256,
        ));
    }

    #[cfg(not(feature = "casc"))]
    {
        Err(casc_feature_error())
    }
}

#[cfg(feature = "casc")]
fn read_active_build_identity(
    install_root: &Path,
    active_product: &str,
) -> crate::Result<BuildIdentity> {
    let build_info_path = install_root.join(".build.info");
    let contents = std::fs::read_to_string(&build_info_path).map_err(|error| {
        crate::Error::Other(format!("read {}: {error}", build_info_path.display()))
    })?;
    parse_active_build_identity(&contents, active_product).map_err(crate::Error::Other)
}

#[cfg(feature = "casc")]
fn parse_active_build_identity(
    contents: &str,
    active_product: &str,
) -> Result<BuildIdentity, String> {
    let build_info = BuildInfoFile::parse_str(contents)
        .map_err(|error| format!("parse .build.info: {error}"))?;
    let active_entry = build_info
        .entries()
        .into_iter()
        .find(|entry| entry.is_active() && entry.product() == Some(active_product))
        .ok_or_else(|| {
            format!(".build.info has no active entry for CASC product {active_product}")
        })?;

    Ok(BuildIdentity {
        version: required_build_info_value(active_entry.version(), "Version")?,
        build_key: required_build_info_value(active_entry.build_key(), "Build Key")?,
        install_key: active_entry.install_key().unwrap_or_default().to_string(),
    })
}

#[cfg(feature = "casc")]
fn required_build_info_value(value: Option<&str>, column: &str) -> Result<String, String> {
    value
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("active .build.info entry is missing {column}"))
}

fn invalidate_cache_if_provenance_mismatched(
    root: &Path,
    expected_provenance: &CacheProvenance,
) -> crate::Result<bool> {
    let provenance_path = root.join(PROVENANCE_FILE);
    let provenance_matches = std::fs::read_to_string(provenance_path)
        .is_ok_and(|actual| actual == expected_provenance.contents());
    if provenance_matches || !root.exists() {
        return Ok(false);
    }

    std::fs::remove_dir_all(root).map_err(|error| {
        crate::Error::Other(format!(
            "remove stale Blizzard UI cache directory {}: {error}",
            root.display()
        ))
    })?;
    Ok(true)
}

enum EntrySyncResult {
    Present,
    Extracted,
    Missing,
}
fn sync_manifest_entry(root: &Path, entry: &str) -> crate::Result<EntrySyncResult> {
    let out_path = root.join(entry);
    if entry_is_present_and_usable(entry, &out_path) {
        return Ok(EntrySyncResult::Present);
    }

    let fdid = manifest_entry_fdid(entry);
    if extract_manifest_entry(fdid, &out_path)? && entry_is_present_and_usable(entry, &out_path) {
        return Ok(EntrySyncResult::Extracted);
    }

    if extract_manifest_entry_by_path(entry, &out_path)?
        && entry_is_present_and_usable(entry, &out_path)
    {
        return Ok(EntrySyncResult::Extracted);
    }

    Ok(EntrySyncResult::Missing)
}

fn entry_is_present_and_usable(entry: &str, path: &Path) -> bool {
    path.is_file() && cache_entry_is_usable(entry, path)
}

fn extract_manifest_entry(fdid: Option<u32>, out_path: &Path) -> crate::Result<bool> {
    match fdid {
        Some(fdid) => extract_fdid(fdid, out_path),
        None => Ok(false),
    }
}

fn manifest_entry_fdid(entry: &str) -> Option<u32> {
    let asset_path = format!("interface/addons/{}", entry.replace('\\', "/"));
    crate::limited_listfile::lookup_path(&asset_path)
}

#[cfg(test)]
fn manifest_entry_is_allowed_unmapped(entry: &str) -> bool {
    if profile_cache::MISTS_REQUIRED_PROFILE_CACHE_ENTRIES.contains(&entry) {
        return true;
    }

    matches!(
        entry,
        "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster.lua"
            | "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster.toc"
            | "Blizzard_CooldownBroadcaster/Blizzard_CooldownBroadcaster_Bootstrap.lua"
            | "Blizzard_CooldownBroadcaster/MessageQueue.lua"
            | "Blizzard_CooldownBroadcaster/TrackedCooldowns.lua"
            | "Blizzard_CombatLog/Wrath/Blizzard_CombatLog.lua"
            | "Blizzard_CombatLog/Wrath/Blizzard_CombatLog.xml"
            | "Blizzard_PrivateAurasUI/Mainline/PrivateAurasTooltip.lua"
            | "Blizzard_PrivateAurasUI/Mainline/PrivateAurasTooltip.xml"
            | "Blizzard_PrivateAurasUI/PrivateAuraInit.lua"
            | "Blizzard_PrivateAurasUI/Shared/PrivateAurasTooltip.lua"
    )
}

#[cfg(feature = "casc")]
fn extract_fdid(fdid: u32, out_path: &Path) -> crate::Result<bool> {
    if !casc_available() {
        return Err(casc_unavailable_error());
    }
    remove_missing_marker(out_path);
    extract_fdid_with_cdn_fallback(
        fdid,
        out_path,
        |fdid, out_path| {
            let resolver = crate::asset_resolver_config::resolver();
            Ok(resolver.ensure_cached(fdid, out_path).is_some())
        },
        extract_fdid_from_cdn,
    )
}

#[cfg(feature = "casc")]
fn extract_fdid_with_cdn_fallback(
    fdid: u32,
    out_path: &Path,
    extract_local: impl FnOnce(u32, &Path) -> crate::Result<bool>,
    extract_cdn: impl FnOnce(u32, &Path) -> crate::Result<bool>,
) -> crate::Result<bool> {
    if extract_local(fdid, out_path)? {
        return Ok(true);
    }

    extract_cdn(fdid, out_path)
}

#[cfg(feature = "casc")]
fn extract_fdid_from_cdn(fdid: u32, out_path: &Path) -> crate::Result<bool> {
    let Some(encoding_key) = casc_path_reader()?.resolve_fdid_to_encoding(fdid) else {
        return Ok(false);
    };
    let data = fetch_cdn_encoding_key(&encoding_key).map_err(|error| {
        crate::Error::Other(format!(
            "download fdid {fdid} from Blizzard CDN by encoding key {}: {error}",
            encoding_key.to_hex()
        ))
    })?;
    write_extracted_casc_path(out_path, &data)?;
    eprintln!(
        "CASC CDN: extracted fdid {} -> {}",
        fdid,
        out_path.display()
    );
    Ok(true)
}

#[cfg(feature = "casc")]
fn fetch_cdn_encoding_key(
    encoding_key: &cascette_crypto::EncodingKey,
) -> std::result::Result<Vec<u8>, String> {
    let product = crate::asset_resolver_config::active_profile_casc_product();
    casc_extract::fetch_encoding_key_blocking(product, encoding_key)
        .map_err(|error| format!("{error:#}"))
}

#[cfg(not(feature = "casc"))]
fn extract_fdid(_fdid: u32, _out_path: &Path) -> crate::Result<bool> {
    Err(casc_feature_error())
}

#[cfg(feature = "casc")]
fn extract_manifest_entry_by_path(entry: &str, out_path: &Path) -> crate::Result<bool> {
    if !casc_available() {
        return Err(casc_unavailable_error());
    }
    remove_missing_marker(out_path);
    let asset_paths = casc_manifest_path_candidates(entry);
    let data = match casc_path_reader()?.read_first_path(&asset_paths) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("CASC path extraction failed for {entry}: {error}");
            return Ok(false);
        }
    };
    write_extracted_casc_path(out_path, &data)?;
    eprintln!(
        "CASC: extracted path {} -> {}",
        asset_paths[0],
        out_path.display()
    );
    Ok(true)
}

#[cfg(not(feature = "casc"))]
fn extract_manifest_entry_by_path(_entry: &str, _out_path: &Path) -> crate::Result<bool> {
    Err(casc_feature_error())
}

#[cfg(not(feature = "casc"))]
fn casc_feature_error() -> crate::Error {
    crate::Error::Other("Blizzard UI CASC sync requires the `casc` feature".to_string())
}

#[cfg(feature = "casc")]
fn casc_unavailable_error() -> crate::Error {
    crate::Error::Other(
        "local WoW CASC data is not available; set WOW_INSTALL_PATH or WOW_DATA_PATH, and make sure WOW_SIM_CASC is not 0".to_string(),
    )
}

#[cfg(feature = "casc")]
fn casc_manifest_path_candidates(entry: &str) -> Vec<String> {
    let slash_entry = entry.replace('\\', "/");
    let lower = format!("interface/addons/{slash_entry}").to_ascii_lowercase();
    let original = format!("Interface/AddOns/{slash_entry}");
    let backslash = original.replace('/', "\\");
    vec![lower, original, backslash]
}

#[cfg(feature = "casc")]
fn write_extracted_casc_path(out_path: &Path, data: &[u8]) -> crate::Result<()> {
    let parent = out_path
        .parent()
        .ok_or_else(|| crate::Error::Other(format!("missing parent for {}", out_path.display())))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| crate::Error::Other(format!("create {}: {e}", parent.display())))?;
    std::fs::write(out_path, data)
        .map_err(|e| crate::Error::Other(format!("write {}: {e}", out_path.display())))
}

#[cfg(feature = "casc")]
struct CascPathReader {
    install: cascette_client_storage::Installation,
}

#[cfg(feature = "casc")]
impl CascPathReader {
    fn new() -> crate::Result<Self> {
        crate::asset_resolver_config::configure_casc_product_env();
        let install_root =
            asset_resolver::wow_install_path().ok_or(crate::Error::WowInstallNotFound)?;
        let casc_dir = asset_resolver::casc_resolver::casc_cache_dir_for_install(&install_root)
            .map_err(|e| crate::Error::Other(format!("resolve CASC cache dir: {e}")))?;
        asset_resolver::casc_resolver::open_resolution_cache_for_install(&install_root)
            .map_err(|e| crate::Error::Other(format!("open CASC resolution cache: {e}")))?;

        let install = cascette_client_storage::Installation::open(install_root.join("Data"))
            .map_err(|e| crate::Error::Other(format!("open CASC install: {e}")))?;
        load_casc_resolver_files(&install, &casc_dir)?;
        run_casc_async(install.initialize())
            .map_err(|e| crate::Error::Other(format!("initialize CASC install: {e}")))?;
        Ok(Self { install })
    }

    fn read_first_path(&self, paths: &[String]) -> crate::Result<Vec<u8>> {
        for path in paths {
            if let Some(data) = self.try_read_path(path)? {
                return Ok(data);
            }
        }

        Err(crate::Error::Other(format!(
            "CASC path not found: {}",
            paths.join(", ")
        )))
    }

    fn try_read_path(&self, path: &str) -> crate::Result<Option<Vec<u8>>> {
        let Some(encoding_key) = self.install.resolver().resolve_path_to_encoding(path) else {
            return Ok(None);
        };
        let data = run_casc_async(self.install.read_file_by_encoding_key(&encoding_key))
            .map_err(|e| crate::Error::Other(format!("read CASC path {path}: {e}")))?;
        Ok(Some(data))
    }

    fn resolve_fdid_to_encoding(&self, fdid: u32) -> Option<cascette_crypto::EncodingKey> {
        self.install.resolver().resolve_fdid_to_encoding(fdid)
    }
}

#[cfg(feature = "casc")]
fn load_casc_resolver_files(
    install: &cascette_client_storage::Installation,
    casc_dir: &Path,
) -> crate::Result<()> {
    let root_data = std::fs::read(casc_dir.join("root.bin"))
        .map_err(|e| crate::Error::Other(format!("read CASC root.bin: {e}")))?;
    let encoding_data = std::fs::read(casc_dir.join("encoding.bin"))
        .map_err(|e| crate::Error::Other(format!("read CASC encoding.bin: {e}")))?;
    install
        .load_root_file(&root_data)
        .map_err(|e| crate::Error::Other(format!("load CASC root.bin: {e}")))?;
    install
        .load_encoding_file(&encoding_data)
        .map_err(|e| crate::Error::Other(format!("load CASC encoding.bin: {e}")))
}

#[cfg(feature = "casc")]
fn casc_path_reader() -> crate::Result<&'static CascPathReader> {
    static CASC_PATH_READER: OnceLock<Result<CascPathReader, String>> = OnceLock::new();
    CASC_PATH_READER
        .get_or_init(|| CascPathReader::new().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| crate::Error::Other(e.clone()))
}

#[cfg(feature = "casc")]
fn run_casc_async<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(future)
}

fn write_complete_marker(root: &Path, expected_provenance: &CacheProvenance) -> crate::Result<()> {
    std::fs::create_dir_all(root).map_err(|error| {
        crate::Error::Other(format!(
            "could not create Blizzard UI cache directory {}: {error}",
            root.display()
        ))
    })?;
    std::fs::write(root.join(PROVENANCE_FILE), expected_provenance.contents()).map_err(
        |error| {
            crate::Error::Other(format!(
                "could not write Blizzard UI cache provenance in {}: {error}",
                root.display()
            ))
        },
    )?;
    std::fs::write(root.join(COMPLETE_MARKER), b"ok\n").map_err(|error| {
        crate::Error::Other(format!(
            "could not write Blizzard UI cache marker in {}: {error}",
            root.display()
        ))
    })
}

#[cfg(feature = "casc")]
fn casc_available() -> bool {
    *CASC_CONFIGURED.get_or_init(|| {
        if std::env::var("WOW_SIM_CASC").ok().as_deref() == Some("0") {
            return false;
        }
        asset_resolver::wow_install_path().is_some()
    })
}

fn remove_missing_marker(path: &Path) {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let missing_marker = path.with_extension(format!("{extension}.missing"));
    if missing_marker.is_file() {
        let _ = std::fs::remove_file(missing_marker);
    }
}

#[cfg(test)]
mod tests;
