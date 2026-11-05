//! Wrapper binary for Blizzard_ArdenwealdGardening tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_ardenweald_gardening/` are re-exported here.

use crate::common;

// Wrath manifest1-382/cache has no ArdenwealdGardening UI implementation.
// These consume its button/tooltip handlers, not the independent C_* model.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_ardenweald_gardening/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_create_attaches_panel.rs"]
mod behavior_create_attaches_panel;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_onenter_active_branch.rs"]
mod behavior_onenter_active_branch;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_onenter_ready_branch.rs"]
mod behavior_onenter_ready_branch;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_onenter_active_and_ready_branch.rs"]
mod behavior_onenter_active_and_ready_branch;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_onenter_dormant_branch.rs"]
mod behavior_onenter_dormant_branch;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_onleave_hides_tooltip.rs"]
mod behavior_onleave_hides_tooltip;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_is_garden_accessible_gates_landing_page.rs"]
mod behavior_is_garden_accessible_gates_landing_page;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_seconds_formatter_format.rs"]
mod behavior_seconds_formatter_format;

#[path = "blizzard_ui/blizzard_ardenweald_gardening/behavior_remaining_seconds_zero_when_no_active.rs"]
mod behavior_remaining_seconds_zero_when_no_active;
}
