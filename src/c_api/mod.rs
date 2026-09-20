//! C_* namespace implementations.
//!
//! Real/state-backed surfaces live at the root of this module. Intentionally
//! unsupported compatibility gaps stay isolated under `permanent_shims`.

pub mod action_macros;
pub mod c_account_services;
pub(crate) mod c_action_bar;
pub mod c_addon_profiler;
pub mod c_addons;
pub mod c_allied_races;
pub mod c_ardenweald_gardening;
pub mod c_arrow_callout_manager;
pub mod c_artifact_relic_forge_ui;
pub mod c_artifact_ui;
#[cfg(feature = "retail-12-1-0")]
pub mod c_aura_container_util;
pub mod c_auto_complete;
pub mod c_azerite_empowered_item;
pub mod c_azerite_essence;
pub mod c_azerite_item;
pub mod c_barber_shop;
pub mod c_battle_net;
pub mod c_catalog_shop;
pub mod c_character_services;
pub mod c_chat_bubbles;
pub mod c_chromie_time;
#[cfg(feature = "retail-12-0-0")]
mod c_combat_audio_alert;
#[cfg(feature = "retail-12-0-0")]
mod c_combat_text;
pub(crate) mod c_creature_info;
pub mod c_cursor;
pub mod c_curve_util;
pub mod c_death_recap;
pub mod c_discord;
#[cfg(feature = "retail-12-1-5")]
pub(crate) mod c_encounter_timeline;
#[cfg(feature = "retail-12-1-5")]
pub(crate) mod c_encounter_warnings;
pub mod c_glue;
pub mod c_housing;
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
#[cfg(feature = "retail-12-0-0")]
mod c_neighborhood_initiative;
pub mod c_paper_doll_info;
pub mod c_party_info;
pub mod c_pet_battles;
pub mod c_ping_secure;
pub mod c_player_choice;
pub mod c_player_interaction_manager;
pub mod c_pvp;
pub mod c_quest_hub;
pub mod c_report_system;
pub mod c_reputation;
#[cfg(feature = "retail-12-1-0")]
pub mod c_secrets;
pub mod c_settings_util;
pub mod c_social;
pub mod c_spec;
pub mod c_spell;
pub mod c_spell_book;
pub mod c_spell_diminish;
pub mod c_stable_info;
pub mod c_string_util;
mod c_string_util_decimal;
pub mod c_summon_info;
pub mod c_texture;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_transmog_collection;
#[cfg(feature = "retail-12-0-0")]
mod c_transmog_outfit_info;
#[cfg(feature = "retail-12-0-0")]
mod c_transmog_sets;
pub mod c_ui_file_asset;
#[cfg(feature = "retail-12-0-0")]
pub mod c_unit_auras;
pub(crate) mod c_weather;
pub mod c_widget;
pub mod c_wow_token_public;
pub mod c_wowtoken_secure;
pub mod c_xml_util;
pub(crate) mod duration_text_binding;
#[cfg(feature = "retail-12-1-5")]
pub mod intl_native;
pub mod item_spell;
#[cfg(feature = "client-mists")]
pub mod legacy_spell_book;
#[cfg(feature = "client-mists")]
mod mists_talents;
#[cfg(feature = "retail-12-1-0")]
mod numeric_rule_formatter;
pub mod permanent_shims;
pub(crate) mod seconds_formatter;
#[cfg(feature = "timed-signal-maps")]
pub mod timed_signal_map;

#[cfg(feature = "client-wowforever")]
pub(crate) mod forever_edit_mode_enums;
#[cfg(feature = "client-wowforever")]
mod gamepad_action_bar_constants;

mod helpers;
mod registration;

pub(crate) use helpers::{ensure_global_table, ensure_namespace, global_val, set_global_val};
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
    c_weather::register(state)?;
    #[cfg(feature = "retail-12-1-5")]
    c_encounter_timeline::register(state)?;
    c_intl::register(state)?;
    #[cfg(feature = "retail-12-1-0")]
    c_aura_container_util::register(state)?;
    #[cfg(feature = "retail-12-1-0")]
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
