//! Default filesystem discovery for local WoW resources.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::saved_variables::WtfConfig;

const DEFAULT_ACCOUNT: &str = "50868465#2";
const DEFAULT_REALM: &str = "Burning Blade";
const DEFAULT_CHARACTER: &str = "Haky";
const ENV_WTF_ACCOUNT: &str = "WOW_SIM_WTF_ACCOUNT";
const ENV_WTF_REALM: &str = "WOW_SIM_WTF_REALM";
const ENV_WTF_CHARACTER: &str = "WOW_SIM_WTF_CHARACTER";
const ENV_INCLUDE_BETA_ADDONS: &str = "WOW_SIM_INCLUDE_BETA_ADDONS";

/// Default writable session state, separate from the read-only WoW WTF import.
pub fn default_session_wtf_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wow-sim")
}

/// Preserve automatic isolation for legacy default constructors in test binaries.
/// Explicit runtime constructors always use the directory supplied by their caller.
pub(crate) fn default_constructor_session_wtf_path() -> PathBuf {
    static NEXT_TEST_SESSION_ID: AtomicU64 = AtomicU64::new(1);
    let in_test_binary = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.ends_with("deps")))
        .unwrap_or(false);

    if in_test_binary {
        let session_id = NEXT_TEST_SESSION_ID.fetch_add(1, Ordering::Relaxed);
        return std::env::temp_dir().join(format!(
            "wow-sim-session-{}-{}",
            std::process::id(),
            session_id
        ));
    }

    default_session_wtf_path()
}

#[derive(Debug, Clone)]
pub struct WowResourcePaths {
    pub install_root: Option<PathBuf>,
    pub casc_data_path: Option<PathBuf>,
    pub interface_path: PathBuf,
    pub addons_path: PathBuf,
    pub wtf_path: Option<PathBuf>,
}

pub fn discover_wow_resources() -> WowResourcePaths {
    let install_root = first_existing_path(wow_install_roots());
    WowResourcePaths {
        casc_data_path: default_casc_data_path_for_root(install_root.as_ref()),
        interface_path: default_interface_path_for_root(install_root.as_ref()),
        addons_path: default_addons_path_for_root(install_root.as_ref()),
        wtf_path: default_wtf_path_for_root(install_root.as_ref()),
        install_root,
    }
}

pub fn default_textures_path() -> PathBuf {
    if let Some(path) = env_path("WOW_SIM_TEXTURES_PATH") {
        return path;
    }
    PathBuf::from("./textures")
}

/// Find a directory entry case-insensitively.
pub fn find_case_insensitive(dir: &Path, name: &str) -> Option<PathBuf> {
    let name_lower = name.to_lowercase();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().to_lowercase() == name_lower {
                return Some(entry.path());
            }
        }
    }
    None
}

pub fn default_casc_data_path() -> Option<PathBuf> {
    discover_wow_resources().casc_data_path
}

pub fn default_interface_path() -> PathBuf {
    discover_wow_resources().interface_path
}

pub fn default_addons_path() -> PathBuf {
    discover_wow_resources().addons_path
}

/// Find third-party roots, replacing default discovery when CLI roots are supplied.
pub fn find_addons_paths(exclusive_paths: &[PathBuf]) -> Vec<PathBuf> {
    if exclusive_paths.is_empty() {
        return default_addons_paths();
    }
    existing_unique_paths(exclusive_paths.to_vec())
}

pub fn exclusive_addons_path_enabled() -> bool {
    std::env::var("WOW_SIM_ADDONS_PATH_EXCLUSIVE")
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

pub fn default_addons_paths() -> Vec<PathBuf> {
    if exclusive_addons_path_enabled() {
        return existing_unique_paths(env_path("WOW_SIM_ADDONS_PATH").into_iter().collect());
    }
    let install_root = first_existing_path(wow_install_roots());
    existing_unique_paths(addons_path_candidates(install_root.as_ref()))
}

/// Blizzard UI addon root used by local benchmarks and dev binaries.
pub fn default_blizzard_ui_addons_path() -> crate::Result<PathBuf> {
    if let Some(cache_path) = crate::blizzard_ui_sync::cached_blizzard_ui_addons_path() {
        return Ok(cache_path);
    }
    Err(crate::Error::Other(format!(
        "missing Blizzard UI cache at {}; run `wow-cli casc sync-blizzard-ui`",
        blizzard_ui_cache_path_label()
    )))
}

pub fn default_wtf_config() -> Option<WtfConfig> {
    let wtf_path = discover_wow_resources().wtf_path?;
    let (account, realm, character) =
        discover_wtf_identity(&wtf_path).unwrap_or_else(|| default_wtf_identity(&wtf_path));
    Some(WtfConfig::new(wtf_path, &account, &realm, &character))
}

/// Keep optional discovery, but require an explicitly selected character to exist.
pub fn try_default_wtf_config() -> crate::Result<Option<WtfConfig>> {
    let Some(character) = env_string(ENV_WTF_CHARACTER) else {
        return Ok(default_wtf_config());
    };
    let wtf_path = env_path("WOW_SIM_WTF_PATH")
        .or_else(default_wtf_path)
        .ok_or_else(|| {
            crate::Error::Other(format!(
                "cannot select WTF character {character:?}: no WTF directory found"
            ))
        })?;
    explicit_wtf_config(
        wtf_path,
        env_string(ENV_WTF_ACCOUNT).as_deref(),
        env_string(ENV_WTF_REALM).as_deref(),
        &character,
    )
    .map(Some)
}

fn explicit_wtf_config(
    wtf_path: PathBuf,
    account: Option<&str>,
    realm: Option<&str>,
    character: &str,
) -> crate::Result<WtfConfig> {
    require_wtf_directory(&wtf_path, "root")?;
    let account_root = wtf_path.join("Account");
    let account = account.map(str::to_owned).unwrap_or_else(|| {
        preferred_child_dir(&account_root, DEFAULT_ACCOUNT)
            .unwrap_or_else(|| DEFAULT_ACCOUNT.to_owned())
    });
    let account_path = account_root.join(&account);
    require_wtf_directory(&account_path, "account")?;
    let realm = realm.map(str::to_owned).unwrap_or_else(|| {
        preferred_child_dir(&account_path, DEFAULT_REALM)
            .unwrap_or_else(|| DEFAULT_REALM.to_owned())
    });
    let realm_path = account_path.join(&realm);
    require_wtf_directory(&realm_path, "realm")?;
    require_wtf_directory(&realm_path.join(character), "character")?;
    Ok(WtfConfig::new(wtf_path, &account, &realm, character))
}

fn require_wtf_directory(path: &Path, component: &str) -> crate::Result<()> {
    match std::fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => Ok(()),
        Ok(_) => Err(crate::Error::Other(format!(
            "selected WTF {component} is not a directory: {}",
            path.display()
        ))),
        Err(error) => Err(crate::Error::Other(format!(
            "cannot open selected WTF {component} directory {}: {error}",
            path.display()
        ))),
    }
}

pub fn default_wtf_path() -> Option<PathBuf> {
    discover_wow_resources().wtf_path
}

fn default_casc_data_path_for_root(install_root: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(path) = env_path("WOW_SIM_CASC_PATH") {
        return Some(path);
    }

    install_root
        .map(|root| root.join("Data"))
        .filter(|path| path.exists())
}

fn default_interface_path_for_root(install_root: Option<&PathBuf>) -> PathBuf {
    first_existing_path(interface_path_candidates(install_root)).unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_default()
            .join("Projects/wow/Interface")
    })
}

fn default_addons_path_for_root(install_root: Option<&PathBuf>) -> PathBuf {
    first_existing_path(addons_path_candidates(install_root))
        .unwrap_or_else(|| PathBuf::from("./Interface/AddOns"))
}

fn default_wtf_path_for_root(install_root: Option<&PathBuf>) -> Option<PathBuf> {
    first_existing_path(wtf_path_candidates(install_root))
}

fn blizzard_ui_cache_path_label() -> String {
    crate::blizzard_ui_sync::default_cache_addons_path()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| {
            format!(
                "~/.cache/wow-ui-sim/blizzard-ui/{}/AddOns",
                crate::client_profile::ACTIVE.cache_subdir()
            )
        })
}

fn interface_path_candidates(install_root: Option<&PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_env_path(&mut paths, "WOW_SIM_INTERFACE_PATH");

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join("Projects/wow/Interface"));
    }

    for root in install_roots_for_candidates(install_root) {
        paths.extend(interface_paths_for_install_root(&root));
    }

    paths
}

pub fn blizzard_interface_art_root_for_install_root(root: &Path) -> Option<PathBuf> {
    let path = match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => root.join("_retail_/BlizzardInterfaceArt"),
        crate::client_profile::ClientProfile::Ptr => root.join("_ptr_/BlizzardInterfaceArt"),
        crate::client_profile::ClientProfile::WowForever => {
            root.join("_classic_beta_/BlizzardInterfaceArt")
        }
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => {
            root.join("_classic_/BlizzardInterfaceArt")
        }
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => {
            root.join("_classic_era_/BlizzardInterfaceArt")
        }
    };
    Some(path)
}

fn interface_paths_for_install_root(root: &Path) -> Vec<PathBuf> {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => vec![
            root.join("_retail_/BlizzardInterfaceArt/Interface"),
            root.join("_retail_/Interface"),
            root.join("_beta_/BlizzardInterfaceArt/Interface"),
            root.join("_beta_/Interface"),
        ],
        crate::client_profile::ClientProfile::Ptr => vec![
            root.join("_ptr_/BlizzardInterfaceArt/Interface"),
            root.join("_ptr_/Interface"),
        ],
        crate::client_profile::ClientProfile::WowForever => vec![
            root.join("_classic_beta_/BlizzardInterfaceArt/Interface"),
            root.join("_classic_beta_/Interface"),
        ],
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => {
            vec![root.join("_classic_/Interface")]
        }
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => {
            vec![root.join("_classic_era_/Interface")]
        }
    }
}

fn addons_path_candidates(install_root: Option<&PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_env_path(&mut paths, "WOW_SIM_ADDONS_PATH");
    paths.push(PathBuf::from("./Interface/AddOns"));
    paths.extend(bundled_addons_path_candidates());

    for root in install_roots_for_candidates(install_root) {
        paths.extend(addon_paths_for_install_root(&root));
    }

    paths
}

/// Find only the bundled test framework, without exposing other bundled addons.
pub fn find_bundled_test_framework_toc() -> Option<PathBuf> {
    bundled_addons_path_candidates()
        .into_iter()
        .find_map(|root| crate::loader::find_toc_file(&root.join("TestFramework")))
}

pub(crate) fn bundled_addons_path_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(exe_path) = std::env::current_exe() {
        paths.extend(bundled_addons_path_candidates_for_exe(&exe_path));
    }
    paths.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/AddOns"));
    paths
}

fn bundled_addons_path_candidates_for_exe(exe_path: &Path) -> Vec<PathBuf> {
    let Some(exe_dir) = exe_path.parent() else {
        return Vec::new();
    };

    let mut paths = vec![exe_dir.join("Interface/AddOns")];
    if let Some(resources_dir) = macos_app_resources_dir(exe_dir) {
        paths.push(resources_dir.join("Interface/AddOns"));
    }

    paths
}

fn macos_app_resources_dir(exe_dir: &Path) -> Option<PathBuf> {
    let contents_dir = exe_dir.parent()?;
    if exe_dir.file_name()? != "MacOS" {
        return None;
    }
    if contents_dir.file_name()? != "Contents" {
        return None;
    }

    Some(contents_dir.join("Resources"))
}

fn addon_paths_for_install_root(root: &Path) -> Vec<PathBuf> {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => {
            let mut paths = vec![root.join("_retail_/Interface/AddOns")];
            if include_beta_addons() {
                paths.push(root.join("_beta_/Interface/AddOns"));
            }
            paths
        }
        crate::client_profile::ClientProfile::Ptr => {
            vec![root.join("_ptr_/Interface/AddOns")]
        }
        crate::client_profile::ClientProfile::WowForever => {
            vec![root.join("_classic_beta_/Interface/AddOns")]
        }
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => {
            vec![root.join("_classic_/Interface/AddOns")]
        }
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => {
            vec![root.join("_classic_era_/Interface/AddOns")]
        }
    }
}

fn include_beta_addons() -> bool {
    std::env::var_os(ENV_INCLUDE_BETA_ADDONS).is_some_and(|value| {
        let value = value.to_string_lossy();
        value != "0" && !value.eq_ignore_ascii_case("false")
    })
}

#[cfg(test)]
fn addon_paths_for_install_root_with_beta(root: &Path, include_beta: bool) -> Vec<PathBuf> {
    let mut paths = vec![root.join("_retail_/Interface/AddOns")];
    if include_beta {
        paths.push(root.join("_beta_/Interface/AddOns"));
    }
    paths
}

fn wtf_path_candidates(install_root: Option<&PathBuf>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_env_path(&mut paths, "WOW_SIM_WTF_PATH");

    for root in install_roots_for_candidates(install_root) {
        paths.extend(wtf_paths_for_install_root(&root));
    }

    if !cfg!(windows) && project_wtf_fallback_enabled() {
        paths.push(PathBuf::from("/syncthing/Sync/Projects/wow/WTF"));
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join("Projects/wow/WTF"));
        }
    }

    paths
}

fn project_wtf_fallback_enabled() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
    )
}

fn wtf_paths_for_install_root(root: &Path) -> Vec<PathBuf> {
    match crate::client_profile::ACTIVE {
        crate::client_profile::ClientProfile::Retail => {
            vec![root.join("_retail_/WTF"), root.join("_beta_/WTF")]
        }
        crate::client_profile::ClientProfile::Ptr => {
            vec![root.join("_ptr_/WTF")]
        }
        crate::client_profile::ClientProfile::WowForever => {
            vec![root.join("_classic_beta_/WTF")]
        }
        crate::client_profile::ClientProfile::Wrath
        | crate::client_profile::ClientProfile::Mists => {
            vec![root.join("_classic_/WTF")]
        }
        crate::client_profile::ClientProfile::Era
        | crate::client_profile::ClientProfile::Anniversary => {
            vec![root.join("_classic_era_/WTF")]
        }
    }
}

fn install_roots_for_candidates(install_root: Option<&PathBuf>) -> Vec<PathBuf> {
    install_root
        .cloned()
        .into_iter()
        .chain(wow_install_roots())
        .collect()
}

fn wow_install_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    push_env_path(&mut roots, "WOW_SIM_WOW_PATH");

    if cfg!(windows) {
        roots.push(PathBuf::from(r"C:\World of Warcraft"));
        roots.push(PathBuf::from(r"C:\Program Files (x86)\World of Warcraft"));
        roots.push(PathBuf::from(r"C:\Program Files\World of Warcraft"));
    } else {
        roots.push(PathBuf::from("/syncthing/World of Warcraft"));
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join("Games/World of Warcraft"));
            roots.push(home.join("World of Warcraft"));
        }
    }

    roots
}

fn discover_wtf_identity(wtf_path: &std::path::Path) -> Option<(String, String, String)> {
    let account_root = wtf_path.join("Account");
    let account = preferred_child_dir(&account_root, &preferred_wtf_account())?;
    let account_path = account_root.join(&account);
    let realm = preferred_child_dir(&account_path, &preferred_wtf_realm())?;
    let character = preferred_child_dir(&account_path.join(&realm), &preferred_wtf_character())?;
    Some((account, realm, character))
}

fn default_wtf_identity(wtf_path: &std::path::Path) -> (String, String, String) {
    let account_root = wtf_path.join("Account");
    let account = preferred_wtf_account();
    let account = preferred_child_dir(&account_root, &account).unwrap_or(account);
    let account_path = account_root.join(&account);
    let realm = preferred_wtf_realm();
    let realm = preferred_child_dir(&account_path, &realm).unwrap_or(realm);
    let character = preferred_wtf_character();
    let character =
        preferred_child_dir(&account_path.join(&realm), &character).unwrap_or(character);
    (account, realm, character)
}

fn preferred_wtf_account() -> String {
    env_string(ENV_WTF_ACCOUNT).unwrap_or_else(|| DEFAULT_ACCOUNT.to_string())
}

fn preferred_wtf_realm() -> String {
    env_string(ENV_WTF_REALM).unwrap_or_else(|| DEFAULT_REALM.to_string())
}

fn preferred_wtf_character() -> String {
    env_string(ENV_WTF_CHARACTER).unwrap_or_else(|| DEFAULT_CHARACTER.to_string())
}

fn preferred_child_dir(parent: &std::path::Path, preferred: &str) -> Option<String> {
    let preferred_path = parent.join(preferred);
    if preferred_path.is_dir() {
        return Some(preferred.to_string());
    }

    std::fs::read_dir(parent)
        .ok()?
        .flatten()
        .find(|entry| entry.path().is_dir() && entry.file_name() != "SavedVariables")
        .map(|entry| entry.file_name().to_string_lossy().to_string())
}

fn first_existing_path(paths: Vec<PathBuf>) -> Option<PathBuf> {
    paths
        .into_iter()
        .find_map(|path| path.exists().then(|| path.canonicalize().unwrap_or(path)))
}

fn existing_unique_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for path in paths.into_iter().filter(|path| path.exists()) {
        let key = path.canonicalize().unwrap_or_else(|_| path.clone());
        if seen.insert(key) {
            unique.push(path);
        }
    }

    unique
}

fn push_env_path(paths: &mut Vec<PathBuf>, var: &str) {
    if let Some(path) = env_path(var) {
        paths.push(path);
    }
}

fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn env_string(var: &str) -> Option<String> {
    std::env::var_os(var)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn explicit_wtf_character_resolves_exact_unicode_identity() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(
            root.path()
                .join("Account/Chosen/Test Realm/Wörkspacé-Family"),
        )
        .unwrap();
        let config = explicit_wtf_config(
            root.path().to_path_buf(),
            Some("Chosen"),
            Some("Test Realm"),
            "Wörkspacé-Family",
        )
        .unwrap();
        assert_eq!(config.account, "Chosen");
        assert_eq!(config.realm, "Test Realm");
        assert_eq!(config.character, "Wörkspacé-Family");
    }

    #[test]
    fn explicit_wtf_character_does_not_fall_back_to_another_character() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("Account/Chosen/Realm/OtherCharacter")).unwrap();
        let error = explicit_wtf_config(
            root.path().to_path_buf(),
            Some("Chosen"),
            Some("Realm"),
            "MissingCharacter",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("character"), "{error}");
        assert!(error.contains("MissingCharacter"), "{error}");
    }

    #[test]
    fn explicit_wtf_account_and_realm_do_not_fall_back_to_other_directories() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("Account/OtherAccount/OtherRealm/Hero")).unwrap();
        for (account, realm, component, selected) in [
            ("MissingAccount", "OtherRealm", "account", "MissingAccount"),
            ("OtherAccount", "MissingRealm", "realm", "MissingRealm"),
        ] {
            let error = explicit_wtf_config(
                root.path().to_path_buf(),
                Some(account),
                Some(realm),
                "Hero",
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains(component), "{error}");
            assert!(error.contains(selected), "{error}");
        }
    }

    #[test]
    fn explicit_wtf_character_rejects_missing_roots_and_non_directory_components() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("MissingWTF");
        let error = explicit_wtf_config(missing, Some("Chosen"), Some("Realm"), "Hero")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("root") && error.contains("MissingWTF"),
            "{error}"
        );

        let realm = root.path().join("Account/Chosen/Realm");
        std::fs::create_dir_all(&realm).unwrap();
        std::fs::write(realm.join("Hero"), b"not a directory").unwrap();
        let error = explicit_wtf_config(
            root.path().to_path_buf(),
            Some("Chosen"),
            Some("Realm"),
            "Hero",
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("character") && error.contains("not a directory"),
            "{error}"
        );
    }

    #[test]
    fn explicit_wtf_character_keeps_discovery_for_unspecified_account_and_realm() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("Account/OnlyAccount/OnlyRealm/Hero")).unwrap();
        let config = explicit_wtf_config(root.path().to_path_buf(), None, None, "Hero").unwrap();
        assert_eq!(config.account, "OnlyAccount");
        assert_eq!(config.realm, "OnlyRealm");
        assert_eq!(config.character, "Hero");
    }

    #[test]
    fn explicit_addon_roots_keep_order_and_deduplicate_existing_paths() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        let duplicate = first.join(".");
        assert_eq!(
            find_addons_paths(&[second.clone(), first.clone(), duplicate]),
            vec![second, first]
        );
    }

    #[test]
    fn missing_explicit_addon_roots_do_not_restore_default_discovery() {
        let temp = tempfile::tempdir().unwrap();
        assert!(find_addons_paths(&[temp.path().join("missing")]).is_empty());
    }

    #[test]
    #[cfg(feature = "client-wowforever")]
    fn wowforever_profile_uses_classic_beta_install_paths_only() {
        let root = Path::new("/wow");
        assert_eq!(
            addon_paths_for_install_root(root),
            vec![root.join("_classic_beta_/Interface/AddOns")]
        );
        assert_eq!(
            wtf_paths_for_install_root(root),
            vec![root.join("_classic_beta_/WTF")]
        );
        assert_eq!(
            interface_paths_for_install_root(root),
            vec![
                root.join("_classic_beta_/BlizzardInterfaceArt/Interface"),
                root.join("_classic_beta_/Interface"),
            ]
        );
        assert_eq!(
            blizzard_interface_art_root_for_install_root(root),
            Some(root.join("_classic_beta_/BlizzardInterfaceArt"))
        );
        assert!(!project_wtf_fallback_enabled());
    }

    #[test]
    fn bundled_addons_candidates_include_macos_app_resources() {
        let exe_path = Path::new("/Applications/WoW Sim.app/Contents/MacOS/wow-sim");

        let candidates = bundled_addons_path_candidates_for_exe(exe_path);

        assert!(
            candidates.contains(&PathBuf::from(
                "/Applications/WoW Sim.app/Contents/Resources/Interface/AddOns"
            )),
            "{candidates:#?}"
        );
    }

    #[test]
    #[cfg(feature = "profile-retail")]
    fn wtf_candidates_prefer_install_root_before_project_mirror_on_linux() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = wtf_path_candidates(Some(&install_root));

        let profile_wtf = wtf_paths_for_install_root(&install_root)
            .into_iter()
            .next()
            .expect("active profile should have a WTF candidate");
        let project_mirror = PathBuf::from("/syncthing/Sync/Projects/wow/WTF");
        let profile_index = candidates
            .iter()
            .position(|path| path == &profile_wtf)
            .expect("profile WTF should be a candidate");
        let project_index = candidates
            .iter()
            .position(|path| path == &project_mirror)
            .expect("project mirror should remain a fallback");
        assert!(
            profile_index < project_index,
            "live install WTF should win over stale project mirrors"
        );
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_wtf_candidates_use_classic_install_flavor_only() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = wtf_paths_for_install_root(&install_root);

        assert_eq!(candidates.as_slice(), &[install_root.join("_classic_/WTF")]);
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_wtf_candidates_exclude_retail_project_mirror() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = wtf_path_candidates(Some(&install_root));

        assert!(
            !candidates
                .iter()
                .any(|path| path.ends_with("Projects/wow/WTF")),
            "PTR must not fall back to retail WTF mirrors: {candidates:#?}"
        );
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_wtf_candidates_use_ptr_install_flavor_only() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = wtf_paths_for_install_root(&install_root);

        assert_eq!(candidates.as_slice(), &[install_root.join("_ptr_/WTF")]);
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_interface_candidates_use_ptr_install_flavor() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = interface_paths_for_install_root(&install_root);

        assert_eq!(
            candidates.as_slice(),
            &[
                install_root.join("_ptr_/BlizzardInterfaceArt/Interface"),
                install_root.join("_ptr_/Interface"),
            ]
        );
    }

    #[test]
    fn addon_candidates_do_not_mix_beta_addons_into_retail_loads() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = addon_paths_for_install_root_with_beta(&install_root, false);

        assert_eq!(
            candidates.as_slice(),
            &[install_root.join("_retail_/Interface/AddOns")]
        );
    }

    #[test]
    fn addon_candidates_can_include_beta_addons_when_requested() {
        let install_root = PathBuf::from("/tmp/wow-install");

        let candidates = addon_paths_for_install_root_with_beta(&install_root, true);

        assert_eq!(
            candidates.as_slice(),
            &[
                install_root.join("_retail_/Interface/AddOns"),
                install_root.join("_beta_/Interface/AddOns"),
            ]
        );
    }

    #[test]
    fn wtf_identity_can_target_palaky_from_environment() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let palaky_path = temp.path().join("Account/50868465#2/Burning Blade/Palaky");
        std::fs::create_dir_all(&palaky_path).expect("create Palaky WTF path");

        let _guard = EnvVarGuard::set("WOW_SIM_WTF_CHARACTER", "Palaky");
        let (_account, _realm, character) = default_wtf_identity(temp.path());

        assert_eq!(character, "Palaky");
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            unsafe {
                std::env::set_var(key, value);
            }
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var(self.key, value),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    #[test]
    #[cfg(feature = "client-mists")]
    fn mists_addon_candidates_use_classic_install_flavor_only() {
        let root = Path::new("/tmp/wow-install");
        assert_eq!(
            addon_paths_for_install_root(root),
            vec![root.join("_classic_/Interface/AddOns")]
        );
    }

    #[test]
    #[cfg(feature = "client-ptr")]
    fn ptr_addon_candidates_use_ptr_install_flavor_only() {
        let root = Path::new("/tmp/wow-install");
        assert_eq!(
            addon_paths_for_install_root(root),
            vec![root.join("_ptr_/Interface/AddOns")]
        );
    }
}
