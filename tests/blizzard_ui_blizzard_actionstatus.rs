//! Wrapper binary for Blizzard_ActionStatus tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_actionstatus/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_actionstatus/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_actionstatus/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_actionstatus/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_actionstatus/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_screenshot_started_hides_frame.rs"]
mod behavior_screenshot_started_hides_frame;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_screenshot_succeeded_shows_success_text.rs"]
mod behavior_screenshot_succeeded_shows_success_text;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_screenshot_failed_shows_failure_text.rs"]
mod behavior_screenshot_failed_shows_failure_text;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_display_message_direct_call.rs"]
mod behavior_display_message_direct_call;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_on_update_fades_alpha.rs"]
mod behavior_on_update_fades_alpha;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_on_update_after_fadetime_hides.rs"]
mod behavior_on_update_after_fadetime_hides;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_alternate_parent_frame.rs"]
mod behavior_alternate_parent_frame;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_update_parent_uses_world_frame_when_top_level_hidden.rs"]
mod behavior_update_parent_uses_world_frame_when_top_level_hidden;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_update_parent_resets_strata_to_tooltip.rs"]
mod behavior_update_parent_resets_strata_to_tooltip;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_alternate_top_level_parent_event.rs"]
mod behavior_alternate_top_level_parent_event;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_glue_branch_registers_glue_events.rs"]
mod behavior_glue_branch_registers_glue_events;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_get_best_parent_no_parent_returns_nil.rs"]
mod behavior_get_best_parent_no_parent_returns_nil;

#[path = "blizzard_ui/blizzard_actionstatus/behavior_clear_all_points_before_anchor.rs"]
mod behavior_clear_all_points_before_anchor;
