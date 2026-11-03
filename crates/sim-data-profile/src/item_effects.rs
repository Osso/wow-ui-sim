//! No item-effect catalog is selected when this crate is built without a profile.
//!
//! Availability flags distinguish an absent source column from captured zeroes.

pub const HAS_SPECIALIZATION_IDS: bool = false;
pub const HAS_PLAYER_CONDITION_IDS: bool = false;
pub static ITEM_EFFECTS: &[(u32, u32, i32, i32, i32)] = &[];
pub static ITEM_EFFECT_METADATA: &[(u32, u32, u32)] = &[];
pub static ITEM_SPELL_NAMES: &[(u32, &str)] = &[];
