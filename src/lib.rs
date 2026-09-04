//! WoW UI Simulator
//!
//! A standalone environment for testing World of Warcraft addons outside the game.
//! Embeds Lua 5.1 and implements the WoW widget API.

#[cfg(feature = "fast-build")]
#[allow(unused_imports)]
use iced_dynamic;

pub mod addon_enable_state;
pub mod addon_tests;
pub mod app_icon_render;
pub mod asset_resolver_config;
pub mod atlas;
#[path = "../data/atlas.rs"]
mod atlas_data;
#[path = "../data/atlas_elements.rs"]
mod atlas_elements;
pub mod blizzard_ui_sync;
pub mod c_api;
pub mod casc_asset_fallback;
pub mod client_profile;
pub mod config;
#[path = "../data/currencies.rs"]
pub mod currencies;
pub mod cvars;
pub mod debug_helpers;
pub mod dump;
#[cfg(feature = "gui")]
pub mod dump_texture;
#[path = "../data/encounter_journal.rs"]
pub mod encounter_journal_data;
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub mod era;
pub mod error;
pub mod event;
pub mod extract_textures;
#[path = "render/font.rs"]
pub mod font;
pub mod global_slot_coverage;
#[path = "../data/global_strings.rs"]
pub mod global_strings;
pub mod iced_app;
#[cfg(not(target_os = "linux"))]
pub mod inspector_server_stub;
#[path = "../data/items.rs"]
pub mod items;
mod key_names;
pub mod keybinding_cache;
pub mod layout;
pub mod limited_listfile;
pub mod loader;
pub mod logging;
pub mod lua_api;
pub mod lua_bridge;
pub mod lua_errors;
#[cfg(unix)]
pub mod lua_server;
#[cfg(not(unix))]
#[path = "lua_server_windows.rs"]
pub mod lua_server;
#[path = "../data/manifest_interface_data.rs"]
pub mod manifest_interface_data;
#[path = "../data/map_art.rs"]
pub mod map_art;
pub mod map_exploration;
#[cfg(feature = "client-mists")]
pub mod mists;
pub mod paths;
pub mod profession_item_overrides;
#[cfg(feature = "retail-12-1-0")]
pub mod ptr;
#[path = "../data/quest_poi_blobs.rs"]
pub mod quest_poi_blobs;
#[path = "../data/quest_ui_map.rs"]
pub mod quest_ui_map;
pub mod render;
pub mod saved_variables;
pub mod screen;
pub mod self_test;
pub mod server_snapshot_import;
pub mod sound;
#[path = "../data/spec_display_spells.rs"]
pub mod spec_display_spells;
#[path = "../data/specializations.rs"]
pub mod specializations;
pub mod spell_description_resolver;
#[path = "../data/spell_descriptions.rs"]
pub mod spell_descriptions;
pub mod spell_lookup;
#[path = "../data/spell_power.rs"]
pub mod spell_power;
#[path = "../data/spells.rs"]
pub mod spells;
pub mod stack;
pub mod startup;
pub mod texture;
pub mod toc;
#[path = "../data/traits.rs"]
pub mod traits;
#[path = "../data/ui_maps.rs"]
pub mod ui_maps;
pub mod widget;
#[cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
pub mod wrath;
pub mod xml;
#[path = "../data/zones.rs"]
pub mod zones;

pub use error::{Error, Result};
#[cfg(feature = "gui")]
pub use iced_app::{DebugOptions, run_iced_ui};

/// Blend mode for quad rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// Standard alpha blending: src * alpha + dst * (1 - alpha)
    #[default]
    Alpha = 0,
    /// Additive blending: src + dst (for highlight textures)
    Additive = 1,
}

/// Computed layout position for a frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
