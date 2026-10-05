//! Real modeled Lua global surfaces.
//!
//! Modules here expose non-`C_*` Lua globals or mixins backed by simulator
//! state/behavior. Unmodeled compatibility defaults belong under
//! `lua_api::workarounds::{temporary,permanent}` instead.

pub mod action_bar_state;
pub mod action_highlights;
pub(crate) mod ambiguate;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod break_up_large_numbers;
pub mod combat_probes;
pub mod combat_stats;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod combo_points;
pub mod container_legacy;
#[cfg(feature = "retail-12-0-0")]
pub mod event_callbacks;
#[cfg(feature = "client-wowforever")]
pub mod forever_stat_contributions;
pub mod frame_level_helpers;
pub mod glyph_state;
pub mod gossip_probes;
#[cfg(feature = "client-wowforever")]
pub mod guild_invites;
pub mod guild_logo;
#[cfg(feature = "client-wowforever")]
pub mod input_interface;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod instanced_identity;
pub mod item_legacy;
pub mod locale_info;
pub mod loot_method;
pub mod math_extensions;
#[cfg(feature = "client-wowforever")]
pub mod merchant_buyback;
#[cfg(feature = "client-wowforever")]
pub mod merchant_repair;
pub mod modifier_keys;
pub mod mouse_probes;
#[cfg(feature = "retail-12-0-5")]
pub(crate) mod nameplate_display;
#[cfg(feature = "client-wowforever")]
pub mod neighborhood_invites;
pub mod net_stats;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod performance_inputs;
pub mod pet_bar;
pub mod pet_stats;
#[cfg(feature = "client-wowforever")]
pub mod player_facing;
pub mod player_identity;
pub mod player_probes;
pub mod preferred_interact;
#[cfg(feature = "client-wowforever")]
pub mod recent_allies_location;
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
pub mod spell_tabs;
#[cfg(feature = "retail-12-1-5")]
pub mod string_extensions;
pub mod table_extensions;
pub mod table_freeze;
pub mod template_queries;
pub mod timerunning;
pub mod ui_widget_container;
pub mod unit_interaction;
#[cfg(feature = "aura-containers")]
pub mod unit_relationships;
pub(crate) mod unit_secret_predicates;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub mod unit_speed;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
pub mod unit_spell_target_name;
pub mod vehicle_possession;
pub mod voice_chat_probes;
pub mod xp_honor_rest;
