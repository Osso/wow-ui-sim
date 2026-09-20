//! Real modeled Lua global surfaces.
//!
//! Modules here expose non-`C_*` Lua globals or mixins backed by simulator
//! state/behavior. Unmodeled compatibility defaults belong under
//! `lua_api::workarounds::{temporary,permanent}` instead.

pub mod action_bar_state;
pub mod action_highlights;
pub mod combat_probes;
pub mod combat_stats;
pub mod container_legacy;
#[cfg(feature = "retail-12-0-0")]
pub mod event_callbacks;
pub mod frame_level_helpers;
pub mod glyph_state;
pub mod gossip_probes;
pub mod guild_logo;
pub mod item_legacy;
pub mod locale_info;
pub mod loot_method;
pub mod math_extensions;
pub mod modifier_keys;
pub mod mouse_probes;
pub mod net_stats;
pub mod pet_bar;
pub mod pet_stats;
pub mod player_identity;
pub mod player_probes;
pub mod shapeshift;
pub mod specialization_helpers;
pub mod specialization_legacy;
pub mod spell_flyout_legacy;
pub mod spell_tabs;
#[cfg(feature = "retail-12-1-5")]
pub mod string_extensions;
pub mod table_extensions;
pub mod table_freeze;
pub mod timerunning;
pub mod ui_widget_container;
pub mod unit_interaction;
#[cfg(feature = "retail-12-1-0")]
pub mod unit_relationships;
pub mod vehicle_possession;
pub mod voice_chat_probes;
pub mod xp_honor_rest;
