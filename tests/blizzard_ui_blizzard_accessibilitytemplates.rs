//! Wrapper binary that pulls every per-aspect Blizzard_AccessibilityTemplates
//! test file under `tests/blizzard_ui/blizzard_accessibilitytemplates/` into a
//! single `cargo test --test ...` target. Cargo only auto-discovers
//! `tests/*.rs`, so the nested `load.rs` / `surface_*.rs` / `behavior_*.rs`
//! files declared by the per-addon plan template need a flat re-export here
//! to be reachable.

use crate::common;

// Wrath has no native accessibility mixin publishers; the simulator intrinsic
// supplies only the template declaration, not the vendor mixin implementations.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_accessibilitytemplates/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_accessibilitytemplates/surface_globals.rs"]
mod surface_globals;
}

#[path = "blizzard_ui/blizzard_accessibilitytemplates/surface_frames.rs"]
mod surface_frames;

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_accessibilitytemplates/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_accessibilitytemplates/behavior_text_size_manager.rs"]
mod behavior_text_size_manager;

#[path = "blizzard_ui/blizzard_accessibilitytemplates/behavior_theme_update.rs"]
mod behavior_theme_update;

#[path = "blizzard_ui/blizzard_accessibilitytemplates/behavior_user_scaled_element.rs"]
mod behavior_user_scaled_element;
}
