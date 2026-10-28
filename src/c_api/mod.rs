//! C_* namespace implementations.
//!
//! Real/state-backed surfaces live at the root of this module. Intentionally
//! unsupported compatibility gaps stay isolated under `permanent_shims`.

pub mod c_guild_info;
pub mod c_recruit_a_friend;
pub(crate) mod class_catalog;
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub(crate) mod classic_startup_data;
#[cfg(feature = "client-retail")]
pub(crate) mod currency_list;
pub(crate) mod ptr_i4_game_rules;
pub mod quest_objectives;
pub mod quest_readiness;
pub(crate) mod quest_reward_currencies;
pub mod ui_widget_manager;
#[cfg(feature = "client-wrath")]
pub mod who_routing;

#[cfg(feature = "retail-12-0-5")]
pub(crate) mod abbreviated_number_formatter;
#[cfg(feature = "retail-12-0-5")]
pub mod action_count_info;
#[cfg(feature = "retail-12-0-5")]
pub use action_count_info::ActionUseCountInfo;
pub mod action_macros;
pub mod action_repeat;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wrath"))]
pub mod action_spell_range;
pub mod action_usability;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
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
pub(crate) mod c_achievement_info_relations;
pub(crate) mod c_action_bar;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_counts;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_loss_of_control;
#[cfg(feature = "retail-12-0-0")]
pub mod c_action_bar_queries;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_action_bar_spell_slots;
pub mod c_addon_profiler;
pub mod c_addons;
pub(crate) mod c_addons_local_table;
pub mod c_allied_races;
pub mod c_ardenweald_gardening;
pub mod c_area_poi_info;
pub mod c_arrow_callout_manager;
pub mod c_artifact_relic_forge_ui;
pub mod c_artifact_ui;
pub(crate) mod c_assisted_combat_action;
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
pub mod c_calendar;
pub mod c_catalog_shop;
pub mod c_catalog_shop_products;
pub mod c_challenge_mode;
pub mod c_character_services;
pub mod c_chat_bubbles;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_chat_info;
pub mod c_chromie_time;
pub(crate) mod c_click_bindings;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_click_bindings_spell;
pub mod c_club;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_combat_audio_alert;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub(crate) mod c_combat_log;
mod c_combat_text;
pub(crate) mod c_console;
pub mod c_cooldown_viewer;
#[cfg(feature = "retail-12-0-0")]
mod c_crafting_orders;
pub(crate) mod c_creature_info;
pub mod c_cursor;
pub mod c_curve_util;
pub mod c_damage_meter;
pub(crate) mod c_date_and_time;
pub(crate) mod c_death_info;
pub mod c_death_recap;
#[cfg(not(feature = "retail-12-0-0"))]
pub(crate) mod c_debug;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_delves_entrance;
pub mod c_discord;
#[cfg(feature = "retail-12-0-0")]
pub mod c_dye_color;
#[cfg(feature = "client-wowforever")]
mod c_edit_mode;
#[cfg(feature = "retail-12-1-0")]
pub(crate) mod c_encounter_timeline;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_encounter_warnings;
mod c_engraving;
pub(crate) mod c_event_scheduler;
pub(crate) mod c_friend_list;
pub(crate) mod c_garrison_damage_classes;
#[cfg(feature = "client-retail")]
pub mod c_garrison_trees;
pub mod c_glue;
pub mod c_housing;
pub mod c_housing_bundles;
pub mod c_input_interface_style;
pub mod c_instance_encounter;
pub mod c_intl;
pub mod c_item_interaction;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_key_bindings;
pub mod c_lfg_info;
pub(crate) mod c_lfg_list_active_entry;
pub(crate) mod c_lfg_list_filter;
#[cfg(feature = "retail-12-0-5")]
pub mod c_lfg_list_policy;
pub mod c_lfg_list_search;
#[cfg(feature = "retail-12-0-0")]
mod c_limited_input;
pub(crate) mod c_log;
pub mod c_login;
pub mod c_loot_history;
pub mod c_major_factions;
pub mod c_map;
pub(crate) mod c_map_bounties;
pub mod c_map_exploration_info;
pub mod c_merchant_frame;
pub(crate) mod c_minimap_quest_blob;
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
mod c_mount_item;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_mount_spell_lookup;
#[cfg(feature = "retail-12-0-7")]
pub mod c_mythic_plus_calendar;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_nameplate_manager;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_navigation;
pub mod c_neighborhood_initiative;
pub mod c_paper_doll_info;
pub mod c_party_info;
pub mod c_pet_battles;
pub mod c_pet_journal_hover;
pub mod c_photo_sharing;
pub mod c_ping_secure;
#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
pub mod c_player_choice;
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
pub(crate) mod c_player_info_guid;
pub(crate) mod c_player_info_name_policy;
pub mod c_player_interaction_manager;
#[cfg(feature = "retail-12-0-0")]
pub mod c_prey;
pub mod c_pvp;
#[cfg(feature = "retail-12-0-5")]
pub mod c_pvp_crowd_control;
pub mod c_quest_hub;
#[cfg(feature = "retail-12-0-5")]
pub mod c_quest_info_system;
pub mod c_quest_log;
#[cfg(feature = "client-wowforever")]
mod c_quest_log_watches;
pub mod c_quest_offer;
pub mod c_super_track;
pub(crate) mod challenge_run;
pub mod container_quest;
pub mod engraving_inputs;
mod forever_outfits;
pub mod forever_startup;
mod lfr_availability;
pub(crate) use lfr_availability::stock_lfr_availability;
pub mod outfit_entry;
mod scenario_finder_availability;
pub mod store_public_eligibility;
pub mod transmog_availability;
#[cfg(feature = "client-wowforever")]
pub use c_quest_log_watches::QuestWatchType;
#[cfg(feature = "client-wowforever")]
pub(crate) use c_quest_log_watches::seed_quest_watch_types;
pub mod c_recent_allies;
pub mod c_report_system;
pub mod c_reputation;
pub(crate) mod c_roleset;
#[cfg(feature = "client-retail")]
pub(crate) mod c_scenario_bonus;
pub mod c_scenario_info;
pub mod c_secrets;
pub mod c_settings_util;
pub mod c_social;
mod c_social_restrictions;
pub mod c_sound;
pub mod c_spec;
mod c_spectating_ui;
pub mod c_spell;
pub(crate) mod c_spell_activation_overlay;
pub mod c_spell_book;
#[cfg(feature = "retail-12-0-0")]
pub mod c_spell_classification;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_spell_counts;
pub mod c_spell_diminish;
pub mod c_stable_info;
mod c_store_secure;
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
pub(crate) mod c_tooltip_info_outfit;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_tooltip_info_spell_mount;
pub(crate) mod c_trade_skill_filter;
pub mod c_trade_skill_quality;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod c_transmog_collection;
pub(crate) mod c_voice_chat_activity;
#[cfg(feature = "retail-12-0-5")]
pub mod c_voice_chat_channels;
mod c_voice_chat_settings;
#[cfg(feature = "retail-12-0-0")]
mod c_voice_chat_volume;
#[cfg(feature = "retail-12-0-5")]
pub mod chat_expressions;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod class_talent_commands;
mod club_inputs;
mod club_members;
pub mod club_model;
pub mod club_self_invitations;
mod club_unread;
pub(crate) mod crafting_execution;
pub(crate) mod crafting_input;
pub(crate) mod crafting_plan;
pub mod crafting_reagents;
pub(crate) mod crafting_tables;
pub(crate) mod cvar_combat_policy;
#[cfg(feature = "retail-12-0-5")]
pub mod equipment_set_command;
pub(crate) mod equipment_set_locks;
pub mod map_world_coordinates;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod patch_12_0_7_inputs;
mod spell_cooldown_output;
pub(crate) mod spell_tooltip_cost;
pub(crate) mod talent_spell;
pub(crate) mod unit_aura_access;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod unit_comparison;
pub mod url_texture_inputs;
#[cfg(feature = "retail-12-0-5")]
pub use c_transmog_collection::IllusionInfo;
#[cfg(feature = "retail-12-0-5")]
pub mod c_spell_maw_powers;
pub mod c_toy_box_info;
#[cfg(feature = "retail-12-0-0")]
pub mod c_transmog_outfit_info;
pub mod c_transmog_sets;
pub(crate) mod c_ui;
pub mod c_ui_file_asset;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod c_unit_aura_altered_form;
#[cfg(any(feature = "aura-spell-classifications", feature = "client-mists"))]
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
#[cfg(feature = "retail-12-0-0")]
pub mod c_weekly_rewards;
pub mod c_widget;
pub mod c_wow_token_public;
pub mod c_wowtoken_secure;
pub mod c_xml_util;
pub(crate) mod c_zone_ability_icon;
pub mod charge_state;
pub mod container_actions;
pub(crate) mod container_inventory;
pub(crate) mod cooldown_duration;
pub(crate) mod duration_clock;
pub(crate) mod duration_text_binding;
#[cfg(feature = "retail-12-0-5")]
pub mod intl_native;
pub mod item_spell;
#[cfg(feature = "retail-12-0-0")]
mod item_stats;
pub mod item_upgrade_eligibility;
#[cfg(feature = "retail-12-0-0")]
pub use item_stats::ItemStatsState;
#[cfg(feature = "retail-12-0-0")]
mod item_set_bonuses;
#[cfg(feature = "retail-12-0-0")]
pub use item_set_bonuses::ItemSetBonuses;
#[cfg(feature = "retail-12-0-0")]
mod c_ping_cooldown;
#[cfg(feature = "retail-12-0-0")]
pub use c_ping_cooldown::PingCooldownInterval;
#[cfg(feature = "retail-12-0-0")]
mod c_ping_texture_kits;
#[cfg(feature = "retail-12-0-0")]
pub use c_ping_texture_kits::PingTextureKits;
#[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
mod c_cvar_readiness;
pub mod c_loss_of_control;
#[cfg(not(feature = "client-wrath"))]
mod combat_audio_throttle;
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
pub(crate) mod private_aura_callbacks;
#[cfg(feature = "retail-12-0-5")]
pub mod private_aura_sounds;
pub(crate) mod ptr_edit_mode_enums;
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

#[cfg(any(feature = "client-wowforever", feature = "client-ptr"))]
pub mod c_game_pad;
#[cfg(feature = "client-wowforever")]
mod c_gamepad_ui;
#[cfg(feature = "client-wowforever")]
pub(crate) mod forever_edit_mode_enums;
#[cfg(feature = "client-wowforever")]
pub(crate) mod forever_finite_constants;
#[cfg(feature = "client-wowforever")]
mod gamepad_action_bar_constants;

pub(crate) mod c_tts_settings;
mod helpers;
#[cfg(feature = "retail-12-0-0")]
mod patch_retired_members;
#[cfg(feature = "retail-12-0-0")]
pub(crate) use patch_retired_members::RETIRED_4_0_1_GLOBALS;
mod registration;

pub(crate) use c_store_secure::register as register_store_secure_table;
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

fn supports_super_tracking() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

pub(crate) fn register_utility_bootstrap_tables(state: &mut LuaState) -> LuaResult<()> {
    c_engraving::register(state)?;
    c_social_restrictions::register(state)?;
    c_loot_history::register_c_loot_history(state)?;
    c_date_and_time::register(state)?;
    #[cfg(feature = "retail-12-0-5")]
    c_chat_info::register(state)?;
    #[cfg(feature = "retail-12-0-5")]
    c_navigation::register(state)?;
    c_weather::register(state)?;
    #[cfg(not(feature = "client-wrath"))]
    combat_audio_throttle::register(state)?;
    #[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
    c_cvar_readiness::register(state)?;
    c_achievement_info_relations::register(state)?;
    c_calendar::register(state)?;
    if supports_super_tracking() {
        c_quest_log::register(state)?;
        c_super_track::register(state)?;
    }
    c_garrison_damage_classes::register(state)?;
    #[cfg(feature = "client-retail")]
    c_garrison_trees::register(state)?;
    c_damage_meter::register(state)?;
    #[cfg(feature = "retail-12-1-0")]
    c_encounter_timeline::register(state)?;
    c_intl::register(state)?;
    #[cfg(feature = "aura-containers")]
    c_aura_container_util::register(state)?;
    #[cfg(any(feature = "aura-containers", feature = "retail-12-0-0"))]
    c_secrets::register(state)?;
    register_specialization_and_model_tables(state)?;
    register_glue_and_display_tables(state)?;
    #[cfg(feature = "retail-12-0-0")]
    patch_retired_members::mark_retired_members(state)?;
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
    c_ui::register(state)?;
    permanent_shims::c_ui::register_c_ui(state)
}

fn register_auxiliary_utility_tables(state: &mut LuaState) -> LuaResult<()> {
    register_token_texture_xml_tables(state)
}

fn register_token_texture_xml_tables(state: &mut LuaState) -> LuaResult<()> {
    c_string_util::register_c_string_util(state)?;
    c_string_util_decimal::register_escape_decimal_non_printables(state)?;
    c_pvp::register_c_pvp_surface(state)?;
    c_death_info::register(state)?;
    c_ping_secure::register_c_ping_secure_surface(state)?;
    #[cfg(feature = "retail-12-0-0")]
    c_ping_cooldown::register(state)?;
    #[cfg(feature = "retail-12-0-0")]
    c_ping_texture_kits::register(state)?;
    c_wowtoken_secure::register_c_wowtoken_secure(state)?;
    c_wow_token_public::register_c_wow_token_public(state)?;
    c_texture::register_c_texture(state)?;
    c_ui_file_asset::register_c_ui_file_asset(state)?;
    c_xml_util::register_c_xml_util(state)
}
