//! C_* namespace implementations.
//!
//! Real/state-backed surfaces live at the root of this module. Intentionally
//! unsupported compatibility gaps stay isolated under `permanent_shims`.

#[cfg(feature = "retail-12-0-5")]
pub(crate) mod abbreviated_number_formatter;
#[cfg(feature = "retail-12-0-5")]
pub mod action_count_info;
#[cfg(feature = "retail-12-0-5")]
pub use action_count_info::ActionUseCountInfo;
pub mod action_macros;
#[cfg(feature = "client-wowforever")]
mod addon_messages;
#[cfg(feature = "retail-12-0-5")]
pub mod aura_duration;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod aura_entry;
#[cfg(feature = "retail-12-0-5")]
pub mod aura_entry_ids;
#[cfg(feature = "aura-instance-enumeration")]
pub(crate) mod aura_filter;
pub mod bag_info;
pub mod c_account_services;
pub(crate) mod c_action_bar;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_counts;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_loss_of_control;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_spell_slots;
pub mod c_addon_profiler;
pub mod c_addons;
pub mod c_allied_races;
pub mod c_ardenweald_gardening;
pub mod c_area_poi_info;
pub mod c_arrow_callout_manager;
pub mod c_artifact_relic_forge_ui;
pub mod c_artifact_ui;
#[cfg(feature = "aura-containers")]
pub mod c_aura_container_util;
pub mod c_auto_complete;
pub mod c_azerite_empowered_item;
pub mod c_azerite_essence;
pub mod c_azerite_item;
pub mod c_barber_shop;
pub mod c_battle_net;
#[cfg(feature = "retail-12-1-0")]
pub(crate) mod c_battle_net_friend_search;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod c_battle_net_invite;
pub mod c_catalog_shop;
pub mod c_catalog_shop_products;
pub mod c_character_services;
pub mod c_chat_bubbles;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_chat_info;
pub mod c_chromie_time;
pub(crate) mod c_click_bindings;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_click_bindings_spell;
#[cfg(feature = "retail-12-0-0")]
mod c_combat_audio_alert;
#[cfg(feature = "client-wowforever")]
mod c_combat_log;
#[cfg(feature = "retail-12-0-0")]
mod c_combat_text;
pub(crate) mod c_console;
pub(crate) mod c_creature_info;
pub mod c_cursor;
pub mod c_curve_util;
pub mod c_damage_meter;
pub mod c_death_recap;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod c_delves_entrance;
pub mod c_discord;
#[cfg(feature = "client-wowforever")]
mod c_edit_mode;
#[cfg(feature = "retail-12-1-5")]
pub(crate) mod c_encounter_timeline;
#[cfg(feature = "retail-12-1-5")]
pub(crate) mod c_encounter_warnings;
pub mod c_glue;
pub mod c_housing;
pub mod c_housing_bundles;
#[cfg(feature = "client-wowforever")]
pub mod c_input_interface_style;
pub mod c_instance_encounter;
pub mod c_intl;
pub mod c_lfg_info;
pub mod c_login;
pub mod c_loot_history;
pub mod c_major_factions;
pub mod c_map;
pub mod c_map_exploration_info;
pub mod c_merchant_frame;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_mount_spell_lookup;
#[cfg(feature = "retail-12-0-7")]
pub mod c_mythic_plus_calendar;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_nameplate_manager;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_navigation;
mod c_neighborhood_initiative;
pub mod c_paper_doll_info;
pub mod c_party_info;
pub mod c_pet_battles;
pub mod c_ping_secure;
pub mod c_player_choice;
pub mod c_player_interaction_manager;
pub mod c_pvp;
pub mod c_quest_hub;
#[cfg(feature = "retail-12-0-5")]
pub mod c_quest_info_system;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod c_recent_allies;
pub mod c_report_system;
pub mod c_reputation;
pub(crate) mod c_roleset;
pub mod c_scenario_info;
pub mod c_secrets;
pub mod c_settings_util;
pub mod c_social;
#[cfg(feature = "retail-12-1-0")]
pub mod c_sound;
pub mod c_spec;
pub mod c_spell;
pub mod c_spell_book;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_spell_counts;
pub mod c_spell_diminish;
pub mod c_stable_info;
pub mod c_string_util;
mod c_string_util_decimal;
pub mod c_summon_info;
pub mod c_texture;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_tooltip_info_aura_instance;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_tooltip_info_indexed_aura;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_tooltip_info_item_context;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_tooltip_info_spell_mount;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_transmog_collection;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod class_talent_commands;
#[cfg(feature = "retail-12-0-5")]
pub mod equipment_set_command;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod patch_12_0_7_inputs;
pub(crate) mod unit_aura_access;
pub mod url_texture_inputs;
#[cfg(feature = "retail-12-0-5")]
pub use c_transmog_collection::IllusionInfo;
#[cfg(feature = "retail-12-0-5")]
pub mod c_spell_maw_powers;
#[cfg(feature = "retail-12-0-0")]
pub mod c_transmog_outfit_info;
#[cfg(feature = "retail-12-0-0")]
mod c_transmog_sets;
pub mod c_ui_file_asset;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_altered_form;
#[cfg(feature = "retail-12-0-5")]
pub mod c_unit_aura_classification;
#[cfg(feature = "retail-12-0-5")]
pub mod c_unit_aura_cooldown_spells;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_dispel_color;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_display_count;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_filter_query;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_index_queries;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_slot_enumeration;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_slot_query;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_spell_queries;
#[cfg(feature = "aura-instance-enumeration")]
pub mod c_unit_auras;
#[cfg(feature = "retail-12-0-5")]
pub mod c_voice_chat_speak;
pub(crate) mod c_weather;
pub mod c_widget;
pub mod c_wow_token_public;
pub mod c_wowtoken_secure;
pub mod c_xml_util;
pub mod charge_state;
pub(crate) mod container_inventory;
pub(crate) mod cooldown_duration;
pub(crate) mod duration_clock;
pub(crate) mod duration_text_binding;
#[cfg(feature = "retail-12-0-5")]
pub mod intl_native;
pub mod item_spell;
#[cfg(feature = "client-mists")]
pub mod legacy_spell_book;
pub(crate) mod loss_of_control;
#[cfg(feature = "client-mists")]
mod mists_talents;
#[cfg(feature = "native-duration-formatting")]
pub(crate) mod native_icu;
#[cfg(feature = "numeric-rule-formatters")]
pub(crate) mod numeric_rule_formatter;
pub(crate) mod on_update_modes;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod patch_12_0_5_enums;
#[cfg(feature = "retail-12-1-0")]
pub(crate) mod patch_12_1_0_enums;
pub mod permanent_shims;
pub mod private_aura_anchors;
#[cfg(feature = "retail-12-0-5")]
pub mod private_aura_sounds;
pub(crate) mod seconds_formatter;
#[cfg(feature = "base-spell-relationships")]
pub mod spell_base;
#[cfg(feature = "timed-signal-maps")]
pub mod timed_signal_map;
#[cfg(feature = "retail-12-0-5")]
pub mod tooltip_item_context;
#[cfg(feature = "retail-12-0-5")]
pub use tooltip_item_context::ItemTooltipContext;
pub mod weapon_enchants;

#[cfg(feature = "client-wowforever")]
pub mod c_game_pad;
#[cfg(feature = "client-wowforever")]
mod c_gamepad_ui;
#[cfg(feature = "client-wowforever")]
pub(crate) mod forever_edit_mode_enums;
#[cfg(feature = "client-wowforever")]
pub(crate) mod forever_finite_constants;
#[cfg(feature = "client-wowforever")]
mod gamepad_action_bar_constants;

mod helpers;
mod registration;

pub(crate) use helpers::{
    ensure_global_table, ensure_namespace, global_val, mark_namespace_keys_removed, set_global_val,
};
pub use permanent_shims::c_map_api;
pub(crate) use registration::{
    register_character_progression_tables, register_interaction_tables, register_item_power_tables,
    register_map_environment_tables, register_map_prefix_tables, register_nameplate_tables,
    register_spell_and_widget_tables,
};

use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_utility_bootstrap_tables(state: &mut LuaState) -> LuaResult<()> {
    c_loot_history::register_c_loot_history(state)?;
    #[cfg(feature = "retail-12-0-5")]
    c_chat_info::register(state)?;
    #[cfg(feature = "retail-12-0-5")]
    c_navigation::register(state)?;
    c_weather::register(state)?;
    c_damage_meter::register(state)?;
    #[cfg(feature = "retail-12-1-5")]
    c_encounter_timeline::register(state)?;
    c_intl::register(state)?;
    #[cfg(feature = "aura-containers")]
    c_aura_container_util::register(state)?;
    #[cfg(any(feature = "aura-containers", feature = "retail-12-0-5"))]
    c_secrets::register(state)?;
    register_specialization_and_model_tables(state)?;
    register_glue_and_display_tables(state)?;
    register_auxiliary_utility_tables(state)
}

fn register_specialization_and_model_tables(state: &mut LuaState) -> LuaResult<()> {
    c_spec::register_c_specialization_info(state)?;
    permanent_shims::c_model_info::register_c_model_info(state)?;
    Ok(())
}

fn register_glue_and_display_tables(state: &mut LuaState) -> LuaResult<()> {
    c_glue::register_c_glue(state)?;
    c_login::register_c_login(state)?;
    permanent_shims::c_ui::register_c_ui(state)
}

fn register_auxiliary_utility_tables(state: &mut LuaState) -> LuaResult<()> {
    register_token_texture_xml_tables(state)
}

fn register_token_texture_xml_tables(state: &mut LuaState) -> LuaResult<()> {
    c_string_util::register_c_string_util(state)?;
    c_string_util_decimal::register_escape_decimal_non_printables(state)?;
    c_pvp::register_c_pvp_surface(state)?;
    c_ping_secure::register_c_ping_secure_surface(state)?;
    c_wowtoken_secure::register_c_wowtoken_secure(state)?;
    c_wow_token_public::register_c_wow_token_public(state)?;
    c_texture::register_c_texture(state)?;
    c_ui_file_asset::register_c_ui_file_asset(state)?;
    c_xml_util::register_c_xml_util(state)
}
