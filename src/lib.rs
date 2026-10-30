//! WoW UI Simulator
//!
//! A standalone environment for testing World of Warcraft addons outside the game.
//! Embeds Lua 5.1 and implements the WoW widget API.

#[cfg(test)]
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/common/profile_fixtures.rs"
));

#[cfg(feature = "fast-build")]
#[allow(unused_imports)]
use iced_dynamic;

pub mod addon_enable_state;
pub mod addon_tests;
pub mod app_icon_render;
pub mod asset_resolver_config;
pub mod atlas;
pub(crate) use sim_data_profile::atlas_data;
pub(crate) use sim_data_profile::atlas_elements;
pub mod blizzard_ui_sync;
pub mod c_api;
pub mod casc_asset_fallback;
pub mod client_profile;
pub mod config;
pub mod forever_db2;
pub mod forever_spell_data;
mod forever_spell_runtime;
pub use sim_data::currencies;
pub mod cvars;
pub mod debug_helpers;
pub mod dump;
#[cfg(feature = "gui")]
pub mod dump_texture;
pub use sim_data::encounter_journal_data;
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub mod era;
pub mod error;
pub mod event;
pub mod extract_textures;
#[path = "render/font.rs"]
pub mod font;
pub mod global_slot_coverage;
pub use sim_data_profile::global_strings;
pub mod iced_app;
#[cfg(not(target_os = "linux"))]
pub mod inspector_server_stub;
pub use sim_data::items;
mod key_names;
pub mod keybinding_cache;
pub mod layout;
pub mod limited_listfile;
pub mod loader;
pub use sim_texture::logging;
pub mod lua_api;
pub use sim_lua_bridge as lua_bridge;
pub use sim_lua_bridge::{
    __lua_bridge_define_function_entries, __lua_bridge_extract_args,
    __lua_bridge_extract_method_args, define_functions, define_methods,
};
pub mod lua_errors;
#[cfg(unix)]
pub mod lua_server;
#[cfg(not(unix))]
#[path = "lua_server_windows.rs"]
pub mod lua_server;
pub use sim_data::manifest_interface_data;
pub use sim_data::map_art;
pub mod map_exploration;
#[cfg(feature = "client-mists")]
pub mod mists;
pub mod paths;
pub use sim_data::profession_item_overrides;
#[cfg(feature = "retail-12-1-0")]
pub mod ptr;
pub use sim_data::quest_poi_blobs;
pub use sim_data::quest_ui_map;
pub mod render;
pub mod saved_variables;
pub mod screen;
#[cfg(feature = "gui")]
pub mod screenshot_output;
pub mod self_test;
pub mod server_snapshot_import;
pub mod sound;
pub use sim_data::spec_display_spells;
pub use sim_data::specializations;
pub mod spell_description_resolver;
pub mod spell_descriptions;
pub mod spell_lookup;
pub mod spell_power;
pub mod spells;
pub mod stack;
pub mod startup;
pub mod texture;
pub mod toc;
pub use sim_data::traits;
pub use sim_data::ui_maps;
pub mod widget;
#[cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
pub mod wrath;
pub mod xml;
pub use sim_data::zones;

pub use error::{Error, Result};
#[cfg(feature = "gui")]
pub use iced_app::{DebugOptions, run_iced_ui};

pub use sim_widget::{BlendMode, LayoutRect};
