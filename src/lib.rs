//! WoW UI Simulator
//!
//! A standalone environment for testing World of Warcraft addons outside the game.
//! Embeds Lua 5.1 and implements the WoW widget API.

pub mod error;
pub mod event;
pub mod lua_api;
pub mod render;
pub mod widget;
pub mod xml;

pub use error::{Error, Result};
