//! Persistent per-product WoW install path settings.
//!
//! Resolution order is CLI, environment, stored settings, then automatic
//! discovery. An explicitly supplied or discovered path is stored in
//! `<platform config dir>/wow-ui-sim/install-paths.json`, keyed by CASC product
//! (`wow`, `wowxptr`, `wowf`, `wow_classic`, `wow_classic_era`). Two clients in
//! different folders therefore keep separate entries.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SETTINGS_DIR_NAME: &str = "wow-ui-sim";
pub const SETTINGS_FILE_NAME: &str = "install-paths.json";

/// One product's stored install locations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallPathEntry {
    /// Install root: the directory that contains `Data`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_path: Option<PathBuf>,
    /// Optional `Data` directory supplied through `WOW_DATA_PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_path: Option<PathBuf>,
}

impl InstallPathEntry {
    pub fn is_empty(&self) -> bool {
        self.install_path.is_none() && self.data_path.is_none()
    }

    /// Stored install root, or the parent of a stored `Data` path.
    pub fn root(&self) -> Option<&Path> {
        self.install_path
            .as_deref()
            .or_else(|| self.data_path.as_deref().and_then(Path::parent))
    }
}

/// Settings file contents. Install locations are keyed by CASC product code.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallPathSettings {
    #[serde(default)]
    pub install_paths: BTreeMap<String, InstallPathEntry>,
}

impl InstallPathSettings {
    pub fn entry(&self, product: &str) -> Option<&InstallPathEntry> {
        self.install_paths.get(product)
    }
}

/// The JSON settings file at a resolved location.
#[derive(Debug, Clone)]
pub struct SettingsFile {
    path: PathBuf,
}

impl SettingsFile {
    /// `<platform config dir>/wow-ui-sim/install-paths.json`.
    pub fn platform_default() -> Option<Self> {
        dirs::config_dir().map(|dir| Self::at(dir.join(SETTINGS_DIR_NAME).join(SETTINGS_FILE_NAME)))
    }

    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the settings. A missing file is empty settings; unreadable content
    /// is reported and treated as empty so startup can continue.
    pub fn load(&self) -> InstallPathSettings {
        unimplemented!("install path settings load")
    }

    pub fn save(&self, settings: &InstallPathSettings) -> std::io::Result<()> {
        let _ = settings;
        unimplemented!("install path settings save")
    }

    /// Insert or replace one product entry and write the file back.
    pub fn store(&self, product: &str, entry: &InstallPathEntry) -> std::io::Result<()> {
        let _ = (product, entry);
        unimplemented!("install path settings store")
    }
}

/// Paths supplied outside the settings file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstallPathInputs {
    pub cli: InstallPathEntry,
    pub env: InstallPathEntry,
}

impl InstallPathInputs {
    /// No CLI overrides; environment values are read from the `WOW_*` names
    /// derived from the entry field names.
    pub fn from_process_env() -> Self {
        Self {
            cli: InstallPathEntry::default(),
            env: InstallPathEntry::from_env(),
        }
    }
}

impl InstallPathEntry {
    fn from_env() -> Self {
        Self {
            install_path: env_path("install_path"),
            data_path: env_path("data_path"),
        }
    }
}

fn env_name(field: &str) -> String {
    format!("WOW_{}", field.to_ascii_uppercase())
}

fn env_path(field: &str) -> Option<PathBuf> {
    std::env::var_os(env_name(field)).map(PathBuf::from)
}

/// Where the resolved install root came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPathSource {
    Cli,
    Env,
    Stored,
    Detected,
}

impl InstallPathSource {
    fn label(self) -> &'static str {
        match self {
            Self::Cli => "CLI",
            Self::Env => "environment",
            Self::Stored => "stored",
            Self::Detected => "auto-detected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInstallPath {
    pub root: PathBuf,
    pub source: InstallPathSource,
}

/// Resolve the install root for one CASC product and persist what was learned.
///
/// The first valid source wins: CLI, environment, stored settings, then
/// `detect`. Invalid supplied or stored paths are reported and skipped. A
/// detected path is stored under `product`.
pub fn resolve_install_path(
    file: &SettingsFile,
    product: &str,
    inputs: &InstallPathInputs,
    detect: impl FnOnce() -> Option<PathBuf>,
) -> Option<ResolvedInstallPath> {
    let _ = (file, product, inputs, detect);
    unimplemented!("install path resolution")
}

/// Settings file and key that store one product's install root, for error
/// messages. The key matches the settings file layout.
pub fn settings_hint(product: &str) -> String {
    let key = format!("install_paths.{product}.install_path");
    match SettingsFile::platform_default() {
        Some(file) => format!("{} (key {key})", file.path().display()),
        None => key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed(dir: &Path) -> PathBuf {
        std::fs::create_dir_all(dir.join("Data/data")).unwrap();
        dir.to_path_buf()
    }

    fn settings_file(dir: &Path) -> SettingsFile {
        SettingsFile::at(dir.join(SETTINGS_DIR_NAME).join(SETTINGS_FILE_NAME))
    }

    fn entry(path: &Path) -> InstallPathEntry {
        InstallPathEntry {
            install_path: Some(path.to_path_buf()),
            data_path: None,
        }
    }

    fn inputs(cli: Option<&Path>, env: Option<&Path>) -> InstallPathInputs {
        InstallPathInputs {
            cli: cli.map(entry).unwrap_or_default(),
            env: env.map(entry).unwrap_or_default(),
        }
    }

    #[test]
    fn env_names_are_derived_from_entry_field_names() {
        assert_eq!(env_name("install_path"), "WOW_INSTALL_PATH");
        assert_eq!(env_name("data_path"), "WOW_DATA_PATH");
    }

    #[test]
    fn resolution_order_prefers_cli_then_env_then_stored_then_detection() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let cli = installed(&temp.path().join("cli"));
        let env = installed(&temp.path().join("env"));
        let stored = installed(&temp.path().join("stored"));
        let detected = installed(&temp.path().join("detected"));
        file.store("wow", &entry(&stored)).unwrap();

        let resolved = resolve_install_path(&file, "wow", &inputs(Some(&cli), Some(&env)), || {
            Some(detected.clone())
        });
        assert_eq!(
            resolved,
            Some(ResolvedInstallPath {
                root: cli,
                source: InstallPathSource::Cli,
            })
        );

        let resolved = resolve_install_path(&file, "wow", &inputs(None, Some(&env)), || {
            Some(detected.clone())
        });
        assert_eq!(
            resolved,
            Some(ResolvedInstallPath {
                root: env,
                source: InstallPathSource::Env,
            })
        );

        let resolved =
            resolve_install_path(&file, "wow", &inputs(None, None), || Some(detected.clone()));
        assert_eq!(
            resolved,
            Some(ResolvedInstallPath {
                root: stored,
                source: InstallPathSource::Stored,
            })
        );
    }

    #[test]
    fn detection_is_stored_when_no_override_or_stored_path_exists() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let detected = installed(&temp.path().join("detected"));

        let resolved = resolve_install_path(&file, "wowxptr", &inputs(None, None), || {
            Some(detected.clone())
        });
        assert_eq!(
            resolved,
            Some(ResolvedInstallPath {
                root: detected.clone(),
                source: InstallPathSource::Detected,
            })
        );
        assert_eq!(file.load().entry("wowxptr"), Some(&entry(&detected)));
    }

    #[test]
    fn invalid_stored_path_is_ignored_and_replaced() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let stale = temp.path().join("moved-away");
        let detected = installed(&temp.path().join("detected"));
        file.store("wow", &entry(&stale)).unwrap();

        let resolved =
            resolve_install_path(&file, "wow", &inputs(None, None), || Some(detected.clone()));
        assert_eq!(
            resolved,
            Some(ResolvedInstallPath {
                root: detected.clone(),
                source: InstallPathSource::Detected,
            })
        );
        assert_eq!(file.load().entry("wow"), Some(&entry(&detected)));
    }

    #[test]
    fn explicit_path_is_persisted_for_the_next_launch() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let supplied = installed(&temp.path().join("supplied"));

        resolve_install_path(&file, "wow_classic", &inputs(None, Some(&supplied)), || {
            None
        })
        .expect("supplied path resolves");

        assert_eq!(file.load().entry("wow_classic"), Some(&entry(&supplied)));
    }

    #[test]
    fn persistence_round_trip_preserves_both_paths() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let install = installed(&temp.path().join("install"));
        let stored = InstallPathEntry {
            install_path: Some(install.clone()),
            data_path: Some(install.join("Data")),
        };

        file.store("wow", &stored).unwrap();

        assert_eq!(file.load().entry("wow"), Some(&stored));
        let raw = std::fs::read_to_string(file.path()).unwrap();
        assert!(raw.contains("\"install_paths\""));
    }

    #[test]
    fn products_keep_separate_install_paths() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        let retail = installed(&temp.path().join("retail"));
        let classic = installed(&temp.path().join("classic"));

        file.store("wow", &entry(&retail)).unwrap();
        file.store("wow_classic", &entry(&classic)).unwrap();

        let settings = file.load();
        assert_eq!(settings.entry("wow"), Some(&entry(&retail)));
        assert_eq!(settings.entry("wow_classic"), Some(&entry(&classic)));
        assert_eq!(settings.entry("wowxptr"), None);
    }

    #[test]
    fn data_path_entry_resolves_to_parent_root() {
        let temp = tempfile::tempdir().unwrap();
        let install = installed(&temp.path().join("install"));
        let entry = InstallPathEntry {
            install_path: None,
            data_path: Some(install.join("Data")),
        };
        assert_eq!(entry.root(), Some(install.as_path()));
    }

    #[test]
    fn missing_settings_file_is_empty_settings() {
        let temp = tempfile::tempdir().unwrap();
        let file = settings_file(temp.path());
        assert!(file.load().install_paths.is_empty());
    }
}
