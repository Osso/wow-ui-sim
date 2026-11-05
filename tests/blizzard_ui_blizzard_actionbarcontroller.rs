//! Wrapper binary for Blizzard_ActionBarController tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_actionbarcontroller/` are re-exported here.

use crate::common;

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[path = "blizzard_ui/blizzard_actionbarcontroller/load.rs"]
mod load;
}

#[cfg(feature = "optional-tests")]
#[path = "blizzard_ui/blizzard_actionbarcontroller/surface_globals.rs"]
mod surface_globals;

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[path = "blizzard_ui/blizzard_actionbarcontroller/surface_frames.rs"]
mod surface_frames;
}

post_wrath_ui_fixture! {
#[cfg(feature = "optional-tests")]
#[path = "blizzard_ui/blizzard_actionbarcontroller/surface_events.rs"]
mod surface_events;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_initial_state.rs"]
mod behavior_initial_state;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_player_entering_world_runs_update_all.rs"]
mod behavior_player_entering_world_runs_update_all;
}

// Wrath ActionButton.lua handles ACTIONBAR_PAGE_CHANGED; adapt the fixture, not applicability.
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_page_changed_routes_to_update_all.rs"]
mod behavior_page_changed_routes_to_update_all;

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_override_status_bar_flags.rs"]
mod behavior_override_status_bar_flags;
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_override_skinned.rs"]
mod behavior_update_override_skinned;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_override_unskinned_uses_main.rs"]
mod behavior_update_override_unskinned_uses_main;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_vehicle_skinned.rs"]
mod behavior_update_vehicle_skinned;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_vehicle_unskinned_uses_vehicle_index.rs"]
mod behavior_update_vehicle_unskinned_uses_vehicle_index;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_temp_shapeshift_uses_temp_index.rs"]
mod behavior_update_temp_shapeshift_uses_temp_index;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_bonus_uses_bonus_index_when_page_one.rs"]
mod behavior_update_bonus_uses_bonus_index_when_page_one;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_pet_battle_uses_action_bar_page.rs"]
mod behavior_update_pet_battle_uses_action_bar_page;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_reset_to_default.rs"]
mod behavior_reset_to_default;
}

// Wrath VehicleMenuBar.lua refreshes vehicle mana on UNIT_DISPLAYPOWER.
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_unit_displaypower_updates_vehicle_mana.rs"]
mod behavior_unit_displaypower_updates_vehicle_mana;

// Wrath BonusActionBarFrame.lua handles all three shapeshift refresh events.
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_shapeshift_events_update_stance_bar.rs"]
mod behavior_shapeshift_events_update_stance_bar;

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_possess_event_updates_possess_and_stance.rs"]
mod behavior_possess_event_updates_possess_and_stance;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_extra_actionbar_event.rs"]
mod behavior_extra_actionbar_event;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_show_bottomleft_after_settings_loaded.rs"]
mod behavior_show_bottomleft_after_settings_loaded;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_settings_loaded_registers_callbacks.rs"]
mod behavior_settings_loaded_registers_callbacks;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_pet_battle_close_validates_transition.rs"]
mod behavior_pet_battle_close_validates_transition;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_pet_battle_open_hides_override.rs"]
mod behavior_pet_battle_open_hides_override;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_bonus_resets_icon_intro_tracker.rs"]
mod behavior_update_bonus_resets_icon_intro_tracker;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_all_iterates_button_events_frame.rs"]
mod behavior_update_all_iterates_button_events_frame;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_update_all_spell_highlights.rs"]
mod behavior_update_all_spell_highlights;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_action_bar_busy_during_slide.rs"]
mod behavior_action_bar_busy_during_slide;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_validate_transition_state_main.rs"]
mod behavior_validate_transition_state_main;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_validate_transition_state_override.rs"]
mod behavior_validate_transition_state_override;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_status_tracking_bar_animation_hook.rs"]
mod behavior_status_tracking_bar_animation_hook;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_main_menu_micro_button_init.rs"]
mod behavior_main_menu_micro_button_init;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_validate_transition_resets_micro_menu.rs"]
mod behavior_validate_transition_resets_micro_menu;
}

post_wrath_ui_fixture! {
#[path = "blizzard_ui/blizzard_actionbarcontroller/behavior_validate_transition_relayouts_uiparent.rs"]
mod behavior_validate_transition_relayouts_uiparent;
}
