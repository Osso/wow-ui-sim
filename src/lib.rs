//! WoW UI Simulator
//!
//! A standalone environment for testing World of Warcraft addons outside the game.
//! Embeds Lua 5.1 and implements the WoW widget API.

#[path = "../data/atlas.rs"]
mod atlas_data;
pub mod atlas;
#[path = "../data/global_strings.rs"]
pub mod global_strings;
#[path = "../data/items.rs"]
pub mod items;
#[path = "../data/manifest_interface_data.rs"]
pub mod manifest_interface_data;
#[path = "../data/spells.rs"]
pub mod spells;
pub mod cvars;
pub mod error;
pub mod event;
pub mod extract_textures;
pub mod iced_app;
pub mod loader;
pub mod lua_api;
pub mod lua_server;
pub mod render;
pub mod saved_variables;
pub mod texture;
pub mod toc;
pub mod widget;
pub mod xml;

pub use error::{Error, Result};
pub use iced_app::{run_iced_ui, run_iced_ui_with_textures, DebugOptions};

/// Computed layout position for a frame.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
