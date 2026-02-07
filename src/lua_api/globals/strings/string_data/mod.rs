//! Static string constant data for WoW UI.
//!
//! This module contains all UI string constants as static arrays,
//! separated from registration logic for maintainability.
//!
//! Split into submodules:
//! - `core_strings` - Error, game, UI, item, combat, and economy strings
//! - `more_strings` - HUD, unit frame, font, LFG, stat, and misc strings
//! - `game_enums` - Structured data, game error enums, and tutorial constants

pub mod core_strings;
pub mod game_enums;
pub mod more_strings;

/// String constant definition: (name, value)
pub type StringDef = (&'static str, &'static str);

/// Integer constant definition: (name, value)
pub type IntDef = (&'static str, i32);

/// Float constant definition: (name, value)
pub type FloatDef = (&'static str, f64);

pub use core_strings::*;
pub use game_enums::*;
pub use more_strings::*;
