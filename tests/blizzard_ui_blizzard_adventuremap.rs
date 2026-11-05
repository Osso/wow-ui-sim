//! Wrapper binary for Blizzard_AdventureMap tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_adventuremap/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_adventuremap/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_adventuremap/surface_globals.rs"]
mod surface_globals;

#[path = "blizzard_ui/blizzard_adventuremap/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_adventuremap/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_adventuremap/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_uipanel_window_entry_registered_at_load.rs"]
mod behavior_uipanel_window_entry_registered_at_load;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_onload_registers_inset_update_event.rs"]
mod behavior_onload_registers_inset_update_event;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_onload_adds_three_data_providers.rs"]
mod behavior_onload_adds_three_data_providers;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_onshow_clears_area_ids_and_sets_map_id.rs"]
mod behavior_onshow_clears_area_ids_and_sets_map_id;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_onhide_calls_dialog_parent_hide_and_close.rs"]
mod behavior_onhide_calls_dialog_parent_hide_and_close;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_refresh_insets_skips_when_no_count.rs"]
mod behavior_refresh_insets_skips_when_no_count;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_refresh_insets_filters_by_area_table_id.rs"]
mod behavior_refresh_insets_filters_by_area_table_id;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_inset_initialize_sizes_and_positions.rs"]
mod behavior_inset_initialize_sizes_and_positions;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_inset_collapse_releases_area_trigger.rs"]
mod behavior_inset_collapse_releases_area_trigger;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_inset_expand_acquires_trigger_and_pans.rs"]
mod behavior_inset_expand_acquires_trigger_and_pans;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_inset_canvas_scale_change_collapses_when_zooming_out.rs"]
mod behavior_inset_canvas_scale_change_collapses_when_zooming_out;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_choice_provider_event_registration.rs"]
mod behavior_quest_choice_provider_event_registration;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_choice_provider_quest_accepted_event_removes_pin.rs"]
mod behavior_quest_choice_provider_quest_accepted_event_removes_pin;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_choice_refresh_acquires_pin_per_valid_choice.rs"]
mod behavior_quest_choice_refresh_acquires_pin_per_valid_choice;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_choice_select_quest_id_shows_dialog.rs"]
mod behavior_quest_choice_select_quest_id_shows_dialog;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_choice_deselect_zooms_out.rs"]
mod behavior_quest_choice_deselect_zooms_out;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_offer_pin_click_anchors_dialog_and_trigger.rs"]
mod behavior_quest_offer_pin_click_anchors_dialog_and_trigger;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_dialog_accept_records_result_and_starts_quest.rs"]
mod behavior_quest_dialog_accept_records_result_and_starts_quest;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_dialog_decline_path_distinguishes_abstain_vs_decline.rs"]
mod behavior_quest_dialog_decline_path_distinguishes_abstain_vs_decline;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_dialog_show_with_quest_swap_declines_previous.rs"]
mod behavior_dialog_show_with_quest_swap_declines_previous;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_quest_dialog_refresh_details_branches_on_widgets_and_rewards.rs"]
mod behavior_quest_dialog_refresh_details_branches_on_widgets_and_rewards;

#[path = "blizzard_ui/blizzard_adventuremap/behavior_zone_summary_provider_groups_quests_by_zone_and_inset.rs"]
mod behavior_zone_summary_provider_groups_quests_by_zone_and_inset;
