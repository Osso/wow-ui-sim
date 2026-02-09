//! Simulator configuration persistence.
//!
//! Stores user preferences (player class, race, rot damage settings) as JSON
//! at `~/.local/share/wow-sim/config.json`. Loaded once on startup; saved on
//! every change so the file is always current.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default config file path.
fn default_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("wow-sim")
        .join("config.json")
}

/// Persisted simulator settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimConfig {
    #[serde(default = "default_class")]
    pub player_class: String,
    #[serde(default = "default_race")]
    pub player_race: String,
    #[serde(default = "default_rot_level")]
    pub rot_damage_level: String,
    #[serde(default)]
    pub rot_damage_enabled: bool,
    #[serde(default = "default_true")]
    pub xp_bar_visible: bool,
    /// Path the config was loaded from (not serialized).
    #[serde(skip)]
    path: PathBuf,
}

fn default_class() -> String { "Warrior".into() }
fn default_race() -> String { "Human".into() }
fn default_rot_level() -> String { "Light (1%)".into() }
fn default_true() -> bool { true }

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            player_class: default_class(),
            player_race: default_race(),
            rot_damage_level: default_rot_level(),
            rot_damage_enabled: false,
            xp_bar_visible: true,
            path: default_path(),
        }
    }
}

impl SimConfig {
    /// Load from disk, falling back to defaults on any error.
    pub fn load() -> Self {
        let path = default_path();
        let mut config = match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
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
