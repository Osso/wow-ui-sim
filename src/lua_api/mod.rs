//! Lua API bindings implementing WoW's addon API.

mod addon_scan;
pub mod animation;
mod builtin_frames;
pub mod cast_completion;
#[cfg(feature = "retail-12-0-5")]
mod cast_success;
#[cfg(feature = "player-cast-durations")]
pub(crate) mod channeling;
pub mod chat_init;
mod diagnostics;
mod env;
mod env_convert;
mod env_events;
mod env_init;
mod env_rilua;
mod env_runtime;
pub(crate) mod frame;
pub(crate) mod frame_substates;
pub(crate) mod game_data;
#[allow(dead_code)]
// Track 3 sub-item 2: populator + read path; compiler fast path lands in sub-items 3-5
pub(crate) mod global_slots;
pub mod globals;
pub(crate) mod handler_timing;
#[cfg(feature = "retail-12-0-7")]
mod host_chat_events;
#[cfg(feature = "retail-12-0-7")]
pub mod host_chat_inputs;
#[cfg(feature = "retail-12-0-7")]
mod host_url_texture_events;
#[allow(dead_code)] // Track 1 sub-item 1: pure data, consumers land in sub-items 2-4
pub(crate) mod hot_literals;
mod key_dispatch;
mod layout;
pub(crate) mod loader_env;
pub mod message_frame;
#[allow(dead_code)] // Phase 3 infrastructure — callers added during VM switch
pub(crate) mod methods;
pub(crate) mod on_update;
#[cfg(feature = "retail-12-0-7")]
pub mod performance_inputs;
pub(crate) mod rect_geometry;
#[allow(dead_code)] // Phase 3 infrastructure — callers added during VM switch
pub(crate) mod script_helpers;
#[cfg(feature = "forbidden-aspects")]
mod script_object_transfer;
pub(crate) mod sim_substates;
pub mod simple_html;
#[cfg(feature = "retail-12-0-7")]
mod unsupported_unit_inputs;
#[cfg(feature = "retail-12-0-5")]
pub use cast_success::CastSuccess;
pub(crate) mod spellcast_events;
pub mod state;
mod state_defaults;
mod state_hit_grid;
pub(crate) mod state_render;
pub(crate) mod state_types;
pub(crate) mod string_format;
#[allow(dead_code)] // Phase 3 infrastructure — callers added during VM switch
pub(crate) mod taint;
pub(crate) mod talent_state;
#[allow(dead_code)] // Phase 3 infrastructure — callers added during VM switch
pub(crate) mod timer_layout;
mod timer_processing;
pub mod tooltip;
pub(crate) mod tracked_recipes;
pub mod workarounds;
pub(crate) mod workarounds_editmode;

// Re-export public types
#[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever", test))]
pub(crate) use env::WowLuaAppData;
pub use env::WowLuaEnv;
pub use globals::global_frames::hide_runtime_hidden_frames;
pub use layout::{
    LayoutRect, anchor_position, compute_frame_rect, frame_position_from_anchor, get_parent_depth,
};
pub use loader_env::LoaderEnv;
pub use message_frame::MessageFrameData;
pub use simple_html::SimpleHtmlData;
pub use state::{
    AddonEnableSnapshot, AddonInfo, AdventureMapInset, AdventureMapQuestInfo,
    AdventureMapQuestOffer, AdventureMapQuestPortrait, AdventureMapState, AdventureMapZoneChoice,
    AlliedRaceInfo, AlliedRaceRacialAbility, ArtifactInfo, AzeriteEmpoweredItemState,
    AzeriteEmpoweredPowerText, AzeriteEmpoweredSelectionKey, AzeriteItemState,
    BarberShopAlternateFormRace, BarberShopCategory, BarberShopCharacterData, BarberShopOption,
    BarberShopState, FactionParagonInfo, HousingState, ItemLocationData, LossOfControlInfo,
    MajorFactionData, PendingTimer, ProfessionQualityInfo, RenownLevelInfo, SimState,
    SpellFlyoutInfo, SpellFlyoutSlot, tick_party_health,
};
pub use tooltip::TooltipData;

// Crate-internal re-exports
pub(crate) use env::next_timer_id;
