//! WoW UI Simulator
//!
//! A standalone environment for testing World of Warcraft addons outside the game.
//! Embeds Lua 5.1 and implements the WoW widget API.

pub mod error;
pub mod event;
pub mod gtk_app;
pub mod loader;
pub mod lua_api;
pub mod render;
pub mod saved_variables;
pub mod texture;
pub mod toc;
pub mod widget;
pub mod xml;

pub use error::{Error, Result};
pub use gtk_app::{run_gtk_ui, run_gtk_ui_with_textures};
