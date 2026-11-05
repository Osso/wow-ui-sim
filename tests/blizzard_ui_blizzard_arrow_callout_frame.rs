//! Wrapper binary for Blizzard_ArrowCalloutFrame tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_arrow_callout_frame/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/surface_templates.rs"]
mod surface_templates;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_show_callout_anchors_to_global_frame.rs"]
mod behavior_show_callout_anchors_to_global_frame;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_show_callout_unknown_anchor_returns_silently.rs"]
mod behavior_show_callout_unknown_anchor_returns_silently;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_show_callout_duplicate_id_is_idempotent.rs"]
mod behavior_show_callout_duplicate_id_is_idempotent;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_show_callout_picks_pool_by_type.rs"]
mod behavior_show_callout_picks_pool_by_type;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_hide_callout_releases_to_pool.rs"]
mod behavior_hide_callout_releases_to_pool;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_hide_callout_nil_id_is_noop.rs"]
mod behavior_hide_callout_nil_id_is_noop;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_close_button_acknowledges_and_hides.rs"]
mod behavior_close_button_acknowledges_and_hides;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_anchor_callout_clears_previous_points.rs"]
mod behavior_anchor_callout_clears_previous_points;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_anchor_directions_use_documented_offsets.rs"]
mod behavior_anchor_directions_use_documented_offsets;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_widget_container_registers_widget_set.rs"]
mod behavior_widget_container_registers_widget_set;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_acknowledged_callouts_persist_across_load.rs"]
mod behavior_acknowledged_callouts_persist_across_load;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_text_width_clamps_to_226.rs"]
mod behavior_text_width_clamps_to_226;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_arrow_anim_reschedules_on_finished.rs"]
mod behavior_arrow_anim_reschedules_on_finished;

#[path = "blizzard_ui/blizzard_arrow_callout_frame/behavior_player_soft_interact_changed_is_registered_unused.rs"]
mod behavior_player_soft_interact_changed_is_registered_unused;
