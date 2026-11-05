//! Wrapper binary for Blizzard_AddOnList tests.
//!
//! Cargo only auto-discovers `tests/*.rs`, so nested files under
//! `tests/blizzard_ui/blizzard_addonlist/` are re-exported here.

use crate::common;

#[path = "blizzard_ui/blizzard_addonlist/load.rs"]
mod load;

#[path = "blizzard_ui/blizzard_addonlist/surface_globals.rs"]
pub(crate) mod surface_globals;

#[path = "blizzard_ui/blizzard_addonlist/surface_frames.rs"]
pub(crate) mod surface_frames;

#[path = "blizzard_ui/blizzard_addonlist/surface_mixins.rs"]
pub(crate) mod surface_mixins;

#[path = "blizzard_ui/blizzard_addonlist/surface_events.rs"]
mod surface_events;

#[path = "blizzard_ui/blizzard_addonlist/behavior_initial_show_populates_scroll.rs"]
mod behavior_initial_show_populates_scroll;

#[path = "blizzard_ui/blizzard_addonlist/behavior_search_box_filters_rows.rs"]
mod behavior_search_box_filters_rows;

#[path = "blizzard_ui/blizzard_addonlist/behavior_force_load_toggles_version_check.rs"]
mod behavior_force_load_toggles_version_check;

#[path = "blizzard_ui/blizzard_addonlist/behavior_enable_all_disables_disable_all.rs"]
mod behavior_enable_all_disables_disable_all;

#[path = "blizzard_ui/blizzard_addonlist/behavior_okay_button_saves_when_pending_changes.rs"]
mod behavior_okay_button_saves_when_pending_changes;

#[path = "blizzard_ui/blizzard_addonlist/behavior_cancel_button_resets_changes.rs"]
mod behavior_cancel_button_resets_changes;

#[path = "blizzard_ui/blizzard_addonlist/behavior_load_lod_addon_calls_load.rs"]
mod behavior_load_lod_addon_calls_load;

#[path = "blizzard_ui/blizzard_addonlist/behavior_is_addon_load_on_demand_walks_deps.rs"]
mod behavior_is_addon_load_on_demand_walks_deps;

#[path = "blizzard_ui/blizzard_addonlist/behavior_addon_tooltip_update_builds_deps_line.rs"]
mod behavior_addon_tooltip_update_builds_deps_line;

#[path = "blizzard_ui/blizzard_addonlist/behavior_addon_tooltip_banned_short_circuit.rs"]
mod behavior_addon_tooltip_banned_short_circuit;

#[path = "blizzard_ui/blizzard_addonlist/behavior_init_addon_uses_question_mark_icon.rs"]
mod behavior_init_addon_uses_question_mark_icon;

#[path = "blizzard_ui/blizzard_addonlist/behavior_init_addon_grays_out_disabled.rs"]
mod behavior_init_addon_grays_out_disabled;

#[path = "blizzard_ui/blizzard_addonlist/behavior_addon_actions_blocked_appends_alert_icon.rs"]
mod behavior_addon_actions_blocked_appends_alert_icon;

#[path = "blizzard_ui/blizzard_addonlist/behavior_tristate_checkbox_visual_states.rs"]
mod behavior_tristate_checkbox_visual_states;

#[path = "blizzard_ui/blizzard_addonlist/behavior_category_collapse_persists_to_saved_var.rs"]
mod behavior_category_collapse_persists_to_saved_var;

#[path = "blizzard_ui/blizzard_addonlist/behavior_node_right_click_opens_context_menu.rs"]
mod behavior_node_right_click_opens_context_menu;

#[path = "blizzard_ui/blizzard_addonlist/behavior_dropdown_setup_radio_options.rs"]
mod behavior_dropdown_setup_radio_options;

#[path = "blizzard_ui/blizzard_addonlist/behavior_update_performance_skips_in_glue.rs"]
mod behavior_update_performance_skips_in_glue;

#[path = "blizzard_ui/blizzard_addonlist/behavior_update_performance_writes_metric_text.rs"]
mod behavior_update_performance_writes_metric_text;

#[path = "blizzard_ui/blizzard_addonlist/behavior_get_addon_metric_percent_warning_color.rs"]
mod behavior_get_addon_metric_percent_warning_color;

#[path = "blizzard_ui/blizzard_addonlist/behavior_glue_dialog_addons_out_of_date.rs"]
mod behavior_glue_dialog_addons_out_of_date;

#[path = "blizzard_ui/blizzard_addonlist/behavior_glue_dialog_confirm_disable_disables_outdated.rs"]
mod behavior_glue_dialog_confirm_disable_disables_outdated;

#[path = "blizzard_ui/blizzard_addonlist/behavior_pending_children_attach_when_parent_loaded.rs"]
mod behavior_pending_children_attach_when_parent_loaded;

#[path = "blizzard_ui/blizzard_addonlist/behavior_search_with_invalid_group_falls_back_to_root.rs"]
mod behavior_search_with_invalid_group_falls_back_to_root;

#[path = "blizzard_ui/blizzard_addonlist/behavior_set_enabled_dependencies_walks_chain.rs"]
mod behavior_set_enabled_dependencies_walks_chain;

#[path = "blizzard_ui/blizzard_addonlist/behavior_reset_to_default_uses_default_enabled.rs"]
mod behavior_reset_to_default_uses_default_enabled;

#[path = "blizzard_ui/blizzard_addonlist/behavior_addon_list_clear_character_dropdown.rs"]
mod behavior_addon_list_clear_character_dropdown;

#[path = "blizzard_ui/blizzard_addonlist/behavior_security_icon_uses_subregion_texcoords.rs"]
mod behavior_security_icon_uses_subregion_texcoords;

#[path = "blizzard_ui/blizzard_addonlist/behavior_memory_usage_throttled_to_15s.rs"]
mod behavior_memory_usage_throttled_to_15s;
