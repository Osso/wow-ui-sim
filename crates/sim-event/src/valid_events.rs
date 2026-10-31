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

/// Selection supplied by the composition crate; this package has no client features.
#[derive(Debug, Clone, Copy)]
pub struct ValidationPolicy {
    pub permissive: bool,
    pub forever: bool,
    pub retail_12_0_0: bool,
    pub retail_12_0_5: bool,
    pub retail_12_0_7: bool,
    pub retail_12_1_0: bool,
    pub retail_12_1_5: bool,
}

/// Check if an event can be passed to `RegisterEvent()`.
///
/// Under non-mainline client profiles the validator is permissive: the
/// wrath/mists/era/anniversary event lists predate the events.yaml dataset
/// (which is mainline-only), so rejecting unknown events would break legitimate
/// WotLK/MoP/Vanilla code paths. Retail and PTR keep strict validation against
/// the generated event tables.
// Forever extends the finite table with events published by its API docs.
const FOREVER_REGISTERABLE_EVENTS: &[&str] = &[
    "ALERT_AGE_VERIFICATION_RESTRICTED",
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
    "GLOBAL_REGION_MOUSE_DOWN",
    "GROUP_BUFF_VISUAL_ALERTS_CHANGED",
    "GUILD_PREFERRED_PLAY_SETTINGS_UPDATED",
    "GUILD_RANKS_UPDATE_ACTIVE_PLAYER",
    "HIDDEN_GROUP_BUFFS_CHANGED",
    "INPUT_DEVICE_INTERFACE_TRANSITION",
    "LEGACY_FRIEND_SYSTEM_STATUS_UPDATED",
    "LFG_LIST_REVEALED_CENSORED_ACTIVE_ENTRY",
    "PET_STATS_UPDATE",
    "PLAYER_PVP_FLAG_CHANGED",
    "PLAYER_SWING",
    "PLAYER_SWING_RANGE_UPDATE",
    "SHARD_TRANSFER",
    "SHARD_TRANSFER_IMMINENT",
    "SOCIAL_UI_FRIENDS_LIST_SYSTEM_STATUS_UPDATED",
    "SOCIAL_UI_SYSTEM_STATUS_UPDATED",
    "UNIT_AURA_BLOCKED",
    "UNIT_AURA_BLOCK_LIST_CLEARED",
    "UNIT_HAPPINESS",
    "UNIT_PET_TRAINING_POINTS",
    "UNIT_PING_PIN_ADDED",
    "UNIT_PING_PIN_REMOVED",
    "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED",
    "VOICE_CHAT_DISCORD_SETTINGS_UPDATED",
    "VOICE_CHAT_VOICE_PROVIDERS_AVAILABLE_CHANGED",
    "VOICE_CHAT_VOICE_PROVIDER_CHANGED",
];
pub fn is_registerable_event(name: &str, policy: ValidationPolicy) -> bool {
    if policy.permissive {
        return !name.is_empty();
    }
    // 12.0.0 retains these as callback events, not script registrations.
    if policy.retail_12_0_0 && matches!(name, "COMBAT_LOG_EVENT" | "COMBAT_LOG_EVENT_UNFILTERED") {
        return false;
    }
    if policy.forever && FOREVER_REGISTERABLE_EVENTS.binary_search(&name).is_ok() {
        return true;
    }
    if policy.retail_12_1_5
        && matches!(
            name,
            "WEATHER_CHANGED" | "ALERT_AGE_VERIFICATION_RESTRICTED"
        )
    {
        return true;
    }
    if policy.retail_12_1_0
        && PATCH_12_1_REMOVED_REGISTERABLE_EVENTS
            .binary_search(&name)
            .is_ok()
    {
        return false;
    }
    if policy.retail_12_1_0 && PATCH_12_1_REGISTERABLE_EVENTS.binary_search(&name).is_ok() {
        return true;
    }
    if policy.retail_12_0_7
        && PATCH_12_0_7_REGISTERABLE_EVENTS
            .binary_search(&name)
            .is_ok()
    {
        return true;
    }
    // 12.0.5 is the first supported epoch after the 12.0.1 change.
    if policy.retail_12_0_5 && name == "CHAT_MSG_ENCOUNTER_EVENT" {
        return false;
    }
    if policy.retail_12_0_5 && name == "UNIT_ARENA_COOLDOWNS_UPDATE" {
        return true;
    }
    if policy.retail_12_0_5 && name == PATCH_12_0_5_REMOVED_REGISTERABLE_EVENT {
        return false;
    }
    if policy.retail_12_0_5 && name == PATCH_12_0_5_REGISTERABLE_EVENT {
        return true;
    }
    if policy.retail_12_0_0
        && PATCH_12_0_0_REGISTERABLE_EVENTS
            .binary_search(&name)
            .is_ok()
    {
        return true;
    }
    super::known_events::contains(name)
}

// Published 12.0.0 names missing from the generated historical event table.
const PATCH_12_0_0_REGISTERABLE_EVENTS: &[&str] = &[
    "CHAT_MSG_ENCOUNTER_EVENT",
    "COMBAT_LOG_APPLY_FILTER_SETTINGS",
    "COMBAT_LOG_EVENT_INTERNAL_UNFILTERED",
    "COMBAT_LOG_REFILTER_ENTRIES",
    "TOOLTIP_SHOW_ITEM_COMPARISON",
];

// The 12.0.5 consolidated table; the other added events are already known or
// are noscript callback events (CLASS_TALENTS_SWITCH_TO_*).
const PATCH_12_0_5_REGISTERABLE_EVENT: &str = "HOUSE_EXTERIOR_DECOR_HIDDEN_CHANGED";

const PATCH_12_0_5_REMOVED_REGISTERABLE_EVENT: &str = "CATALOG_SHOP_PMT_IMAGE_DOWNLOADED";

const PATCH_12_0_7_REGISTERABLE_EVENTS: &[&str] = &["ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED"];

const PATCH_12_1_REMOVED_REGISTERABLE_EVENTS: &[&str] =
    &["BATTLETAG_INVITE_SHOW", "HOUSING_LAYOUT_NUM_FLOORS_CHANGED"];

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
pub fn is_valid_event(name: &str, policy: ValidationPolicy) -> bool {
    is_registerable_event(name, policy) || NON_REGISTERABLE_EVENTS.binary_search(&name).is_ok()
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

pub fn callback_events() -> &'static [&'static str] {
    CALLBACK_EVENTS
}
pub fn restricted_events() -> &'static [&'static str] {
    RESTRICTED_EVENTS
}

/// Events that exist in the WoW client but cannot be registered by addons.
/// From wowless events.yaml: registerable = false.
const NON_REGISTERABLE_EVENTS: &[&str] = &["COMBAT_LOG_EVENT", "COMBAT_LOG_EVENT_UNFILTERED"];
