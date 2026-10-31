//! Real modeled Lua global surfaces.
//!
//! Modules here expose non-`C_*` Lua globals or mixins backed by simulator
//! state/behavior. Unmodeled compatibility defaults belong under
//! `lua_api::workarounds::{temporary,permanent}` instead.

pub mod action_bar_state;
pub mod action_highlights;
pub(crate) mod ambiguate;
#[cfg(feature = "client-retail")]
pub(crate) mod bnet_friend_index;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod break_up_large_numbers;
pub mod combat_probes;
pub mod combat_stats;
pub mod combo_points;
pub mod container_legacy;
#[cfg(feature = "retail-12-0-0")]
pub mod event_callbacks;
#[cfg(feature = "client-wowforever")]
pub mod forever_stat_contributions;
pub mod frame_level_helpers;
pub mod glyph_state;
pub mod gossip_probes;
pub(crate) mod group_role_counts;
#[cfg(feature = "client-wowforever")]
pub mod guild_invites;
pub mod guild_logo;
pub(crate) mod heal_prediction;
#[cfg(feature = "client-wowforever")]
pub mod input_interface;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod instanced_identity;
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
pub mod locale_info;
pub mod loot_method;
pub mod loot_specialization;
pub mod math_extensions;
#[cfg(feature = "client-wowforever")]
pub mod merchant_buyback;
#[cfg(feature = "client-wowforever")]
pub mod merchant_repair;
pub mod modifier_keys;
pub mod mouse_probes;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod nameplate_display;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub mod neighborhood_invites;
pub mod net_stats;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod party_ai;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod performance_inputs;
pub mod pet_bar;
pub mod pet_stats;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod player_facing;
pub mod player_identity;
pub mod player_probes;
pub mod preferred_interact;
pub mod publication_12_0_0;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod publication_unit_12_0_0;
pub(crate) mod quest_dialog;
#[cfg(feature = "retail-12-0-0")]
pub(crate) mod raid_targets;
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub mod recent_allies_location;
#[cfg(feature = "client-wrath")]
pub mod recruit_a_friend;
pub mod session_exit;
pub mod shapeshift;
#[cfg(feature = "retail-12-0-7")]
pub mod simulate_mouse;
pub mod specialization_helpers;
pub mod specialization_legacy;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub(crate) mod spell_confirmation_prompts;
pub mod spell_flyout_legacy;
#[cfg(not(feature = "retail-12-0-0"))]
pub mod spell_tabs;
#[cfg(feature = "retail-12-1-5")]
pub mod string_extensions;
pub mod table_extensions;
pub mod table_freeze;
pub mod template_queries;
pub mod timerunning;
pub mod totems;
pub(crate) mod tutorial_requests;
pub mod ui_widget_container;
#[cfg(any(feature = "client-era", feature = "client-anniversary"))]
pub(crate) mod unit_health_knowledge;
pub mod unit_interaction;
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
