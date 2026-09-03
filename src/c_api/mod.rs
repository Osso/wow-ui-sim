//! C_* namespace implementations.
//!
//! Real/state-backed surfaces live at the root of this module. Intentionally
//! unsupported compatibility gaps stay isolated under `permanent_shims`.

pub mod c_account_services;
pub mod c_addon_profiler;
pub mod c_addons;
pub mod c_allied_races;
pub mod c_ardenweald_gardening;
pub mod c_arrow_callout_manager;
pub mod c_artifact_relic_forge_ui;
pub mod c_artifact_ui;
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
pub mod c_cursor;
pub mod c_death_recap;
pub mod c_discord;
pub mod c_glue;
pub mod c_housing;
pub mod c_instance_encounter;
pub mod c_lfg_info;
pub mod c_login;
pub mod c_loot_history;
pub mod c_major_factions;
pub mod c_map;
pub mod c_map_exploration_info;
pub mod c_merchant_frame;
pub mod c_paper_doll_info;
pub mod c_party_info;
pub mod c_pet_battles;
pub mod c_ping_secure;
pub mod c_player_choice;
pub mod c_player_interaction_manager;
pub mod c_pvp;
pub mod c_quest_hub;
pub mod c_report_system;
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
pub mod c_ui_file_asset;
pub mod c_widget;
pub mod c_wow_token_public;
pub mod c_wowtoken_secure;
pub mod c_xml_util;
pub mod item_spell;
#[cfg(feature = "client-mists")]
pub mod legacy_spell_book;
#[cfg(feature = "client-mists")]
mod mists_talents;
pub mod permanent_shims;

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
