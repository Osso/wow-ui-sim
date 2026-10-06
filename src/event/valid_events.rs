//! WoW event name validation.
//! Generated from wowless data/products/wow/events.yaml (mainline).
//!
//! Two concepts:
//! - **Registerable**: events that addons can pass to `RegisterEvent()`.
//!   Split across valid_events_a/b/c submodules.
//! - **Non-registerable**: valid events that exist in the client but
//!   `RegisterEvent()` rejects them.
//!
//! `is_valid_event` = registerable OR non-registerable (for C_EventUtils).
//! `is_registerable_event` = only registerable (for RegisterEvent).

/// Check if an event can be passed to `RegisterEvent()`.
///
/// Under non-mainline client profiles the validator is permissive: the
/// wrath/mists/era/anniversary event lists predate the events.yaml dataset
/// (which is mainline-only), so rejecting unknown events would break legitimate
/// WotLK/MoP/Vanilla code paths. Retail and PTR keep strict validation against
/// the generated event tables.
#[cfg(any(
    feature = "client-wrath",
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
pub fn is_registerable_event(name: &str) -> bool {
    crate::wrath::is_registerable_event(name)
}

// Forever extends the finite table with events published by its API docs.
#[cfg(feature = "client-wowforever")]
const FOREVER_REGISTERABLE_EVENTS: &[&str] = &[
    "CHAT_MSG_COLLECTED_APPEARANCE",
    "CHAT_MSG_GUILD_DISCORD",
    "CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW",
    "DIEL_CYCLE_CHANGED",
    "DISCORD_GUILD_LOBBY_UPDATE",
    "DISCORD_GUILD_SETTINGS_UPDATE",
    "DISCORD_LINK_UPDATE",
    "DISCORD_SERVER_LIST_UPDATE",
    "DISCORD_STATUS_UPDATE",
    "EXTERNAL_EVENT_LAUNCH_URL_FAILED",
    "GAMEPAD_POSSESS_BAR_OVERRIDE_CHANGED",
    "GAMEPAD_STANCE_BAR_OVERRIDE_CHANGED",
    "GAME_PAD_ALLOW_HOVER_EVENTS_WITH_FREE_LOOK_CHANGED",
    "GROUP_BUFF_VISUAL_ALERTS_CHANGED",
    "GUILD_PREFERRED_PLAY_SETTINGS_UPDATED",
    "GUILD_RANKS_UPDATE_ACTIVE_PLAYER",
    "HIDDEN_GROUP_BUFFS_CHANGED",
    "INPUT_DEVICE_INTERFACE_TRANSITION",
    "LEGACY_FRIEND_SYSTEM_STATUS_UPDATED",
    "LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY",
    "PET_STATS_UPDATE",
    "PLAYER_SWING",
    "PLAYER_SWING_RANGE_UPDATE",
    "SHARD_TRANSFER",
    "SHARD_TRANSFER_IMMINENT",
    "SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED",
    "SOCIAL_UI_SYSTEM_STATUS_UPDATED",
    "UNIT_AURA_BLOCKED",
    "UNIT_HAPPINESS",
    "UNIT_PET_TRAINING_POINTS",
    "UNIT_PING_PIN_ADDED",
    "UNIT_PING_PIN_REMOVED",
];
#[cfg(any(feature = "retail-12-0-0", feature = "client-wowforever"))]
pub fn is_registerable_event(name: &str) -> bool {
    // 12.0.0 retains these as callback events, not script registrations.
    #[cfg(feature = "retail-12-0-0")]
    if matches!(name, "COMBAT_LOG_EVENT" | "COMBAT_LOG_EVENT_UNFILTERED") {
        return false;
    }
    #[cfg(feature = "client-wowforever")]
    if FOREVER_REGISTERABLE_EVENTS.binary_search(&name).is_ok() {
        return true;
    }
    #[cfg(feature = "retail-12-1-5")]
    if name == "WEATHER_CHANGED" {
        return true;
    }
    #[cfg(feature = "retail-12-1-0")]
    if PATCH_12_1_REMOVED_REGISTERABLE_EVENTS
        .binary_search(&name)
        .is_ok()
    {
        return false;
    }
    #[cfg(feature = "retail-12-1-0")]
    if PATCH_12_1_REGISTERABLE_EVENTS.binary_search(&name).is_ok() {
        return true;
    }
    #[cfg(feature = "retail-12-0-7")]
    if PATCH_12_0_7_REGISTERABLE_EVENTS
        .binary_search(&name)
        .is_ok()
    {
        return true;
    }
    #[cfg(feature = "retail-12-0-5")]
    if name == PATCH_12_0_5_REMOVED_REGISTERABLE_EVENT {
        return false;
    }
    #[cfg(feature = "retail-12-0-5")]
    if name == PATCH_12_0_5_REGISTERABLE_EVENT {
        return true;
    }
    #[cfg(feature = "retail-12-0-0")]
    if PATCH_12_0_0_REGISTERABLE_EVENTS
        .binary_search(&name)
        .is_ok()
    {
        return true;
    }
    super::known_events::contains(name)
}

// Published 12.0.0 names missing from the generated historical event table.
#[cfg(feature = "retail-12-0-0")]
const PATCH_12_0_0_REGISTERABLE_EVENTS: &[&str] = &[
    "CHAT_MSG_ENCOUNTER_EVENT",
    "COMBAT_LOG_APPLY_FILTER_SETTINGS",
    "COMBAT_LOG_EVENT_INTERNAL_UNFILTERED",
    "COMBAT_LOG_REFILTER_ENTRIES",
    "TOOLTIP_SHOW_ITEM_COMPARISON",
];

// The 12.0.5 consolidated table; the other added events are already known or
// are noscript callback events (CLASS_TALENTS_SWITCH_TO_*).
#[cfg(feature = "retail-12-0-5")]
const PATCH_12_0_5_REGISTERABLE_EVENT: &str = "HOUSE_EXTERIOR_DECOR_HIDDEN_CHANGED";

#[cfg(feature = "retail-12-0-5")]
const PATCH_12_0_5_REMOVED_REGISTERABLE_EVENT: &str = "CATALOG_SHOP_PMT_IMAGE_DOWNLOADED";

#[cfg(feature = "retail-12-0-7")]
const PATCH_12_0_7_REGISTERABLE_EVENTS: &[&str] = &["ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED"];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_REMOVED_REGISTERABLE_EVENTS: &[&str] =
    &["BATTLETAG_INVITE_SHOW", "HOUSING_LAYOUT_NUM_FLOORS_CHANGED"];

#[cfg(feature = "retail-12-1-0")]
const PATCH_12_1_REGISTERABLE_EVENTS: &[&str] = &[
    "BATTLE_NET_FRIEND_TAG_ENABLED_STATUS_UPDATED",
    "BATTLE_NET_TITLE_FRIEND_CUSTOM_NAME_ENABLED_STATUS_UPDATED",
    "CHAT_MSG_GUILD_DISCORD",
    "CONFIRM_BATTLE_NET_FRIEND_INVITE_SHOW",
    "DISCORD_GUILD_ACHIEVEMENT",
    "DISCORD_GUILD_LOBBY_UPDATE",
    "DISCORD_GUILD_SETTINGS_UPDATE",
    "DISCORD_LINK_UPDATE",
    "DISCORD_SERVER_LIST_UPDATE",
    "DISCORD_STATUS_UPDATE",
    "EXTERNAL_EVENT_LAUNCH_URL_FAILED",
    "FULLSCREEN_BROWSER_SPINNER_HIDE",
    "FULLSCREEN_BROWSER_SPINNER_SHOW",
    "GROUP_BUFF_VISUAL_ALERTS_CHANGED",
    "GUILD_RANKS_UPDATE_ACTIVE_PLAYER",
    "HIDDEN_GROUP_BUFFS_CHANGED",
    "HOUSE_RESET_COMPLETED",
    "HOUSE_RESET_FAILED",
    "HOUSING_BLUEPRINTS_AVAILABILITY_CHANGED",
    "HOUSING_BLUEPRINT_COLLECTION_FAILURE",
    "HOUSING_BLUEPRINT_COLLECTION_RECEIVED",
    "HOUSING_BLUEPRINT_CONTENTS_FAILURE",
    "HOUSING_BLUEPRINT_CONTENTS_RECEIVED",
    "HOUSING_BLUEPRINT_DELETE_FAILURE",
    "HOUSING_BLUEPRINT_DELETE_SUCCESS",
    "HOUSING_BLUEPRINT_EXPORT_FAILURE",
    "HOUSING_BLUEPRINT_EXPORT_SUCCESS",
    "HOUSING_BLUEPRINT_IMPORT_FAILURE",
    "HOUSING_BLUEPRINT_IMPORT_STARTED",
    "HOUSING_BLUEPRINT_IMPORT_SUCCESS",
    "HOUSING_BLUEPRINT_RENAME_FAILURE",
    "HOUSING_BLUEPRINT_RENAME_SUCCESS",
    "HOUSING_LAYOUT_OCCUPIED_FLOOR_RANGE_CHANGED",
    "HOUSING_NEW_DECOR_PLACE_COMPLETE",
    "IGNORE_NEIGHBORHOOD_RESPONSE",
    "LEGACY_FRIEND_SYSTEM_STATUS_UPDATED",
    "LFG_LIST_CENSORED_ACTIVE_ENTRY_UPDATE",
    "LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY",
    "SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED",
    "SOCIAL_UI_SOCIAL_QUEUE_SYSTEM_STATUS_UPDATED",
    "SOCIAL_UI_SYSTEM_STATUS_UPDATED",
    "UNIT_AURA_BLOCKED",
    "UNIT_AURA_BLOCK_LIST_CLEARED",
    "UNIT_PING_PIN_ADDED",
    "UNIT_PING_PIN_REMOVED",
];

/// Check if an event name is known to the WoW client (registerable or not).
pub fn is_valid_event(name: &str) -> bool {
    is_registerable_event(name) || NON_REGISTERABLE_EVENTS.binary_search(&name).is_ok()
}

/// Restricted events cannot be registered by addons (returns false as second value).
const RESTRICTED_EVENTS: &[&str] = &[
    "COMBAT_LOG_APPLY_FILTER_SETTINGS",
    "COMBAT_LOG_EVENT",
    "COMBAT_LOG_EVENT_UNFILTERED",
    "COMBAT_LOG_REFILTER_ENTRIES",
    "MINIMAP_PING",
    "TUTORIAL_COMBAT_EVENT",
];

pub fn is_restricted_event(name: &str) -> bool {
    RESTRICTED_EVENTS.binary_search(&name).is_ok()
}

/// Events that support RegisterEventCallback (from wowless events.yaml callback: true).
const CALLBACK_EVENTS: &[&str] = &[
    "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX",
    "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_NAME",
    "CLASS_TALENTS_SWITCH_TO_SPECIALIZATION_BY_INDEX",
    "CLASS_TALENTS_SWITCH_TO_SPECIALIZATION_BY_NAME",
    "COMBAT_LOG_APPLY_FILTER_SETTINGS",
    "COMBAT_LOG_EVENT",
    "COMBAT_LOG_EVENT_UNFILTERED",
    "COMBAT_LOG_REFILTER_ENTRIES",
    "ENCOUNTER_STATE_CHANGED",
    "MINIMAP_PING",
    "TOOLTIP_SHOW_ITEM_COMPARISON",
];

pub fn is_callback_event(name: &str) -> bool {
    CALLBACK_EVENTS.binary_search(&name).is_ok()
}

#[cfg(all(test, feature = "profile-retail"))]
mod retail_tests {
    use super::is_registerable_event;

    #[test]
    fn url_texture_request_result_is_registerable() {
        assert!(is_registerable_event("URL_TEXTURE_REQUEST_RESULT"));
    }

    #[cfg(feature = "retail-12-0-5")]
    #[test]
    fn patch_12_0_5_event_additions_and_removals() {
        assert!(is_registerable_event("HOUSE_EXTERIOR_DECOR_HIDDEN_CHANGED"));
        assert!(!is_registerable_event("CATALOG_SHOP_PMT_IMAGE_DOWNLOADED"));
    }

    #[cfg(feature = "retail-12-0-7")]
    #[test]
    fn patch_12_0_7_events_are_registerable() {
        assert!(is_registerable_event(
            "ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED"
        ));
    }
}

#[cfg(all(test, feature = "retail-12-1-0"))]
mod patch_12_1_tests {
    use super::is_registerable_event;

    #[test]
    fn patch_12_1_events_are_registerable() {
        assert!(is_registerable_event(
            "BATTLE_NET_FRIEND_TAG_ENABLED_STATUS_UPDATED"
        ));
        assert!(is_registerable_event("EXTERNAL_EVENT_LAUNCH_URL_FAILED"));
        assert!(is_registerable_event("GROUP_BUFF_VISUAL_ALERTS_CHANGED"));
        assert!(is_registerable_event("HOUSING_BLUEPRINT_IMPORT_STARTED"));
        assert!(is_registerable_event("UNIT_PING_PIN_ADDED"));
        assert!(!is_registerable_event("BATTLETAG_INVITE_SHOW"));
    }
}

pub fn callback_events() -> &'static [&'static str] {
    CALLBACK_EVENTS
}
pub fn restricted_events() -> &'static [&'static str] {
    RESTRICTED_EVENTS
}

/// Events that exist in the WoW client but cannot be registered by addons.
/// From wowless events.yaml: registerable = false.
const NON_REGISTERABLE_EVENTS: &[&str] = &["COMBAT_LOG_EVENT", "COMBAT_LOG_EVENT_UNFILTERED"];
