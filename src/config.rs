//! Simulator configuration persistence.
//!
//! Stores user preferences (player class, race, rot damage settings) as JSON
//! at `config.json` inside the writable session WTF. Loaded once on startup; saved on
//! every change so the file is always current.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Persisted simulator settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    #[serde(default = "default_class")]
    pub player_class: String,
    #[serde(default = "default_race")]
    pub player_race: String,
    #[serde(default = "default_rot_level")]
    pub rot_damage_level: String,
    #[serde(default = "default_xp_level")]
    pub xp_level: String,
    #[serde(default)]
    pub party_size: u8,
    /// Player movement state toggles.
    #[serde(default)]
    pub movement: MovementConfig,
    /// Path the config was loaded from (not serialized).
    #[serde(skip)]
    path: PathBuf,
}

/// Persisted movement toggle states (all default to false).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MovementConfig {
    #[serde(default)]
    pub moving: bool,
    #[serde(default)]
    pub mounted: bool,
    #[serde(default)]
    pub flying: bool,
    #[serde(default)]
    pub falling: bool,
    #[serde(default)]
    pub swimming: bool,
}

fn default_class() -> String {
    "Paladin".into()
}
fn default_race() -> String {
    "Human".into()
}
fn default_rot_level() -> String {
    "Off".into()
}
fn default_xp_level() -> String {
    "33%".into()
}

impl Default for SimConfig {
    fn default() -> Self {
        Self::default_for_session(&crate::paths::default_session_wtf_path())
    }
}

impl SimConfig {
    /// Default preferences using this session's save destination.
    pub fn default_for_session(root: &Path) -> Self {
        Self {
            player_class: default_class(),
            player_race: default_race(),
            rot_damage_level: default_rot_level(),
            xp_level: default_xp_level(),
            party_size: 0,
            movement: MovementConfig::default(),
            path: root.join("config.json"),
        }
    }

    /// Load from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        Self::load_from_session_wtf(&crate::paths::default_session_wtf_path())
    }

    /// Load preferences from this session's directory and retain its save destination.
    pub fn load_from_session_wtf(root: &Path) -> Self {
        let path = root.join("config.json");
        let mut config = match std::fs::read_to_string(&path) {
            Ok(contents) => {
                serde_json::from_str(&contents).unwrap_or_else(|_| Self::default_for_session(root))
            }
            Err(_) => Self::default_for_session(root),
        };
        config.path = path;
        config
    }

    /// Persist current config to disk.
    pub fn save(&self) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&self.path, json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SimConfig;

    #[test]
    fn missing_party_size_deserializes_to_solo_default() {
        let config: SimConfig =
            serde_json::from_str(r#"{"player_class":"Paladin","xp_level":"33%"}"#).unwrap();

        assert_eq!(config.party_size, 0);
    }
}
