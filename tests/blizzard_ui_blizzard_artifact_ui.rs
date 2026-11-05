//! Wrapper binary for Blizzard_ArtifactUI tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_artifact_ui/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_artifact_ui/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_artifact_ui/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_artifact_ui/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_artifact_ui/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_show_panel_with_no_artifact_redirects.rs"]
mod behavior_show_panel_with_no_artifact_redirects;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_show_panel_with_purchased_ranks.rs"]
mod behavior_show_panel_with_purchased_ranks;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_set_tab_resizes_panel.rs"]
mod behavior_set_tab_resizes_panel;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_artifact_close_event_hides_panel.rs"]
mod behavior_artifact_close_event_hides_panel;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_artifact_update_with_new_item_swaps_data.rs"]
mod behavior_artifact_update_with_new_item_swaps_data;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_artifact_update_when_hidden_shows_panel.rs"]
mod behavior_artifact_update_when_hidden_shows_panel;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_confirm_respec_popup_watchdog.rs"]
mod behavior_confirm_respec_popup_watchdog;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_inventory_hover_highlights_relic_slot.rs"]
mod behavior_inventory_hover_highlights_relic_slot;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_underlay_rotation_drag.rs"]
mod behavior_underlay_rotation_drag;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_appearance_tutorial_helptip.rs"]
mod behavior_appearance_tutorial_helptip;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_knowledge_tooltip_meta_powers.rs"]
mod behavior_knowledge_tooltip_meta_powers;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_knowledge_tooltip_clears_on_leave.rs"]
mod behavior_knowledge_tooltip_clears_on_leave;

#[path = "blizzard_ui/blizzard_artifact_ui/behavior_can_view_predicate.rs"]
mod behavior_can_view_predicate;
