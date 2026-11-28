//! Real modeled Lua global surfaces.
//!
//! Modules here expose non-`C_*` Lua globals or mixins backed by simulator
//! state/behavior. Unmodeled compatibility defaults belong under
//! `lua_api::workarounds::{temporary,permanent}` instead.

pub(crate) mod achievement_eligibility;
pub(crate) mod achievement_ui_access;
pub mod action_bar_state;
pub mod action_highlights;
pub(crate) mod ambiguate;
pub(crate) mod bank_reagents;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod battle_net_account;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod battle_net_friends;
pub(crate) mod bit_library;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod bnet_blocked_list;
#[cfg(feature = "client-retail")]
pub(crate) mod bnet_friend_index;
#[cfg(any(feature = "retail-12-0-5", feature = "client-wowforever"))]
pub(crate) mod break_up_large_numbers;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod camera_fov_defaults;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod chat_type_index;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod client_info;
pub mod combat_probes;
pub mod combat_stats;
pub mod combo_points;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod configuration_warnings;
pub mod container_legacy;
pub mod cursor_model;
pub(crate) mod cursor_presentation;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod dangerous_scripts;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod debug_environment;
#[cfg(feature = "client-retail")]
pub(crate) mod difficulty_info;
pub(crate) mod display_scale;
#[cfg(any(
    feature = "client-retail",
    feature = "client-wowforever",
    feature = "retail-12-0-0"
))]
pub mod event_callbacks;
#[cfg(feature = "client-wowforever")]
pub mod forever_stat_contributions;
pub mod frame_level_helpers;
pub(crate) mod game_clock;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod global_text;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub(crate) mod glue_background;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub(crate) mod glue_character_season;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub(crate) mod glue_connection;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod glue_enter_world;
#[cfg(any(feature = "client-ptr", feature = "client-wowforever"))]
pub mod glue_feature_status;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod glue_save_character_order;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub(crate) mod glue_select_character;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod glue_selection;
pub mod glue_state;
pub mod glyph_state;
pub mod gossip_probes;
pub(crate) mod group_role_counts;
#[cfg(feature = "client-wowforever")]
pub mod guild_invites;
pub mod guild_logo;
pub(crate) mod heal_prediction;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub mod input_interface;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod instance_legacy;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod instanced_identity;
#[cfg(feature = "interaction-dialogs")]
pub mod interaction_dialogs;
pub mod item_legacy;
#[cfg(feature = "client-wrath")]
pub(crate) mod legacy_gamma;
pub(crate) mod legacy_guild_selection;
#[cfg(any(
    feature = "client-wrath",
    feature = "client-era",
    feature = "client-anniversary"
))]
pub mod legacy_skills;
#[cfg(feature = "client-wrath")]
pub(crate) mod legacy_sound;
#[cfg(feature = "client-retail")]
pub mod lfd_lock_requests;
pub mod lfg_role_check;
pub mod locale_info;
pub mod loot_method;
pub mod loot_specialization;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod lua_time;
pub(crate) mod macro_getters;
pub mod math_extensions;
pub(crate) mod merchant_affordability;
pub mod merchant_buyback;
pub(crate) mod merchant_filter;
pub mod merchant_item_link;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod merchant_repair;
mod merchant_requirements;
#[cfg(feature = "client-mists")]
pub(crate) mod mists_cinematic;
pub(crate) mod mixins;
pub mod modifier_keys;
pub mod mouse_probes;
#[cfg(feature = "client-wowforever")]
pub mod name_reservation;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod nameplate_display;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub mod neighborhood_invites;
pub mod net_stats;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod party_ai;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod performance_inputs;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod performance_metrics;
pub mod pet_bar;
pub mod pet_stats;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod player_facing;
pub mod player_identity;
pub(crate) mod player_languages;
pub mod player_probes;
pub mod preferred_interact;
pub(crate) mod protected_globals;
pub mod publication_12_0_0;
pub(crate) mod publication_unit_12_0_0;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod pvp_talent_info;
pub(crate) mod quest_dialog;
#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
pub(crate) mod quest_faction_group;
#[cfg(feature = "quest-giver-requests")]
pub(crate) mod quest_giver_requests;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod quest_objective_info;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod raid_targets;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub mod recent_allies_location;
#[cfg(feature = "client-wrath")]
pub mod recruit_a_friend;
pub(crate) mod region;
pub(crate) mod release_timer;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod run_binding;
pub(crate) mod secure_helpers;
pub mod session_exit;
pub mod shapeshift;
#[cfg(feature = "simulated-mouse-input")]
pub mod simulate_mouse;
pub(crate) mod source_location;
pub mod specialization_helpers;
pub mod specialization_legacy;
#[cfg(feature = "client-retail")]
pub(crate) mod specialization_system;
#[cfg(any(
    all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ),
    feature = "client-retail",
    feature = "client-wowforever"
))]
pub(crate) mod spell_confirmation_prompts;
pub mod spell_flyout_legacy;
#[cfg(not(feature = "retail-12-0-0"))]
pub mod spell_tabs;
#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
pub mod string_extensions;
pub mod table_extensions;
pub mod table_freeze;
#[cfg(feature = "client-retail")]
pub(crate) mod taxi_map;
pub mod template_queries;
pub mod timerunning;
pub mod totems;
pub(crate) mod trainer_services;
pub(crate) mod tutorial_requests;
pub mod ui_widget_container;
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub(crate) mod unit_health_knowledge;
pub mod unit_interaction;
#[cfg(feature = "client-retail")]
pub(crate) mod unit_power_display_mod;
#[cfg(feature = "aura-containers")]
pub mod unit_relationships;
pub(crate) mod unit_roles;
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
pub(crate) mod unit_roles_assignment;
pub(crate) mod unit_secret_predicates;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod unit_speed;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod unit_spell_target_name;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod unit_stagger;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod unit_threat;
pub(crate) mod utf8_case;
pub mod vehicle_possession;
pub mod voice_chat_probes;
#[cfg(feature = "client-wrath")]
pub mod wrath_auction_catalog;
#[cfg(feature = "client-wrath")]
pub(crate) mod wrath_autocomplete;
#[cfg(feature = "client-wrath")]
pub mod wrath_calendar;
#[cfg(feature = "client-wrath")]
pub(crate) mod wrath_friends;
pub(crate) mod wrath_guild_bank;
pub(crate) mod wrath_guild_info;
pub mod wrath_inventory_candidates;
#[cfg(feature = "client-wrath")]
pub mod wrath_legacy;
#[cfg(feature = "client-wrath")]
pub mod wrath_legacy_lfg;
#[cfg(any(
    feature = "client-wrath",
    feature = "client-era",
    feature = "client-anniversary"
))]
pub mod wrath_legacy_state;
#[cfg(feature = "client-wrath")]
pub mod wrath_minimap_arrows;
#[cfg(feature = "client-wrath")]
pub(crate) mod wrath_quest_dialog;
pub mod wrath_relic_slot;
pub mod wrath_spell_slots;
#[cfg(feature = "client-wrath")]
pub(crate) mod wrath_spellbook;
#[cfg(feature = "client-wrath")]
pub(crate) mod wrath_startup_queries;
#[cfg(feature = "client-wrath")]
pub mod wrath_talent_groups;
#[cfg(feature = "client-wrath")]
pub mod wrath_voice;
pub mod xp_honor_rest;

pub mod wrath_currency;

pub mod inventory_candidates;

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
pub(crate) mod cinematic_summary;

#[cfg(any(
    all(feature = "profile-retail", feature = "retail-12-1-5"),
    feature = "client-wowforever"
))]
pub(crate) mod set_role_button;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod world_presence;
