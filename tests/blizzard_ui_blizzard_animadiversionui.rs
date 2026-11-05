//! Wrapper binary for Blizzard_AnimaDiversionUI tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_animadiversionui/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_animadiversionui/surface_globals.rs"]
mod surface_globals;

// Wrath's manifest-backed FrameXML/AddOns have no AnimaDiversion UI publishers.
// Keep the independent enum probe above on every profile.
post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_animadiversionui/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_animadiversionui/surface_frames.rs"]
mod surface_frames;

#[path = "blizzard_ui/blizzard_animadiversionui/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_animadiversionui/surface_mixins.rs"]
mod surface_mixins;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onload_seeds_data_providers_and_pin_levels.rs"]
mod behavior_onload_seeds_data_providers_and_pin_levels;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onload_styles_close_button.rs"]
mod behavior_onload_styles_close_button;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_can_reinforce_gates_on_progress.rs"]
mod behavior_can_reinforce_gates_on_progress;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_has_available_node_walks_node_list.rs"]
mod behavior_has_available_node_walks_node_list;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onhide_unregisters_events_and_plays_close_sound.rs"]
mod behavior_onhide_unregisters_events_and_plays_close_sound;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onevent_anima_close_hides_panel.rs"]
mod behavior_onevent_anima_close_hides_panel;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onevent_currency_update_refreshes.rs"]
mod behavior_onevent_currency_update_refreshes;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_onshow_registers_events_and_plays_open_sound.rs"]
mod behavior_onshow_registers_events_and_plays_open_sound;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_setup_bolster_progress_caps_at_max_and_records_new_gems.rs"]
mod behavior_setup_bolster_progress_caps_at_max_and_records_new_gems;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_setup_bolster_progress_releases_pool_and_inits_info_frame.rs"]
mod behavior_setup_bolster_progress_releases_pool_and_inits_info_frame;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_setup_currency_frame_formats_quantity_with_icon.rs"]
mod behavior_setup_currency_frame_formats_quantity_with_icon;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_set_exclusive_selection_node_dims_other_pins.rs"]
mod behavior_set_exclusive_selection_node_dims_other_pins;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_pin_setup_node_marks_reinforce_state.rs"]
mod behavior_pin_setup_node_marks_reinforce_state;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_pin_set_visual_state_dims_unavailable_only.rs"]
mod behavior_pin_set_visual_state_dims_unavailable_only;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_pin_click_routes_by_reinforce_state.rs"]
mod behavior_pin_click_routes_by_reinforce_state;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_pin_have_enough_anima_compares_currency.rs"]
mod behavior_pin_have_enough_anima_compares_currency;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_pin_refresh_tooltip_branches_on_state_and_currency.rs"]
mod behavior_pin_refresh_tooltip_branches_on_state_and_currency;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_confirm_channel_popup_plays_covenant_sound.rs"]
mod behavior_confirm_channel_popup_plays_covenant_sound;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_confirm_reinforce_popup_calls_select_anima_node_permanent.rs"]
mod behavior_confirm_reinforce_popup_calls_select_anima_node_permanent;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_connection_setup_anchors_origin_and_rotates.rs"]
mod behavior_connection_setup_anchors_origin_and_rotates;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_data_provider_refresh_skips_when_origin_or_nodes_missing.rs"]
mod behavior_data_provider_refresh_skips_when_origin_or_nodes_missing;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_data_provider_connects_active_pins_to_origin.rs"]
mod behavior_data_provider_connects_active_pins_to_origin;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_reinforce_info_select_node_updates_title_and_button.rs"]
mod behavior_reinforce_info_select_node_updates_title_and_button;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_reinforce_button_click_shows_static_popup.rs"]
mod behavior_reinforce_button_click_shows_static_popup;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_update_tutorial_tips_branches_on_reinforce_state.rs"]
mod behavior_update_tutorial_tips_branches_on_reinforce_state;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_world_quest_data_provider_returns_subclassed_pin_template.rs"]
mod behavior_world_quest_data_provider_returns_subclassed_pin_template;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_data_provider_clear_effect_on_all_pins_skips_exempt.rs"]
mod behavior_data_provider_clear_effect_on_all_pins_skips_exempt;

#[path = "blizzard_ui/blizzard_animadiversionui/behavior_try_show_seeds_state_and_opens_panel.rs"]
mod behavior_try_show_seeds_state_and_opens_panel;
}
