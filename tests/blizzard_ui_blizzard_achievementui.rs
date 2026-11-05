//! Wrapper binary that pulls every per-aspect Blizzard_AchievementUI
//! test file under `tests/blizzard_ui/blizzard_achievementui/` into a
//! single `cargo test --test ...` target. Cargo only auto-discovers
//! `tests/*.rs`, so the nested `load.rs` / `surface_*.rs` / `behavior_*.rs`
//! files declared by the per-addon plan template need a flat re-export here
//! to be reachable.

use crate::common;

#[path = "blizzard_ui/blizzard_achievementui/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_achievementui/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_achievementui/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_achievementui/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_achievementui/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_achievementui/behavior_toggle.rs"]
mod behavior_toggle;

#[path = "blizzard_ui/blizzard_achievementui/behavior_progress_bar.rs"]
mod behavior_progress_bar;

#[path = "blizzard_ui/blizzard_achievementui/behavior_summary_update.rs"]
mod behavior_summary_update;

#[path = "blizzard_ui/blizzard_achievementui/behavior_button_click_link.rs"]
mod behavior_button_click_link;

#[path = "blizzard_ui/blizzard_achievementui/behavior_button_double_click.rs"]
mod behavior_button_double_click;

#[path = "blizzard_ui/blizzard_achievementui/behavior_objective_rep.rs"]
mod behavior_objective_rep;

#[path = "blizzard_ui/blizzard_achievementui/behavior_comparison_set.rs"]
mod behavior_comparison_set;

#[path = "blizzard_ui/blizzard_achievementui/behavior_comparison_clear.rs"]
mod behavior_comparison_clear;

#[path = "blizzard_ui/blizzard_achievementui/behavior_comparison_portrait.rs"]
mod behavior_comparison_portrait;

#[path = "blizzard_ui/blizzard_achievementui/behavior_achievement_earned_event.rs"]
mod behavior_achievement_earned_event;

#[path = "blizzard_ui/blizzard_achievementui/behavior_criteria_update_event.rs"]
mod behavior_criteria_update_event;

#[path = "blizzard_ui/blizzard_achievementui/behavior_category_select.rs"]
mod behavior_category_select;

#[path = "blizzard_ui/blizzard_achievementui/behavior_category_default.rs"]
mod behavior_category_default;

#[path = "blizzard_ui/blizzard_achievementui/behavior_search_progress.rs"]
mod behavior_search_progress;

#[path = "blizzard_ui/blizzard_achievementui/behavior_search_filter.rs"]
mod behavior_search_filter;

#[path = "blizzard_ui/blizzard_achievementui/behavior_guild_view_toggle.rs"]
mod behavior_guild_view_toggle;

#[path = "blizzard_ui/blizzard_achievementui/behavior_guild_member_tooltip.rs"]
mod behavior_guild_member_tooltip;

#[path = "blizzard_ui/blizzard_achievementui/behavior_comparison_friend_completed.rs"]
mod behavior_comparison_friend_completed;

#[path = "blizzard_ui/blizzard_achievementui/behavior_statistic_row.rs"]
mod behavior_statistic_row;

#[path = "blizzard_ui/blizzard_achievementui/behavior_expand_collapse.rs"]
mod behavior_expand_collapse;

#[path = "blizzard_ui/blizzard_achievementui/behavior_saturate.rs"]
mod behavior_saturate;

#[path = "blizzard_ui/blizzard_achievementui/behavior_track_toggle.rs"]
mod behavior_track_toggle;

#[path = "blizzard_ui/blizzard_achievementui/behavior_filter_dropdown.rs"]
mod behavior_filter_dropdown;

#[path = "blizzard_ui/blizzard_achievementui/behavior_restricted_mode.rs"]
mod behavior_restricted_mode;
