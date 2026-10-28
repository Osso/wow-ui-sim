//! Profile/epoch selection stays above the feature-neutral event crate.
use sim_event::valid_events::ValidationPolicy;
pub use sim_event::valid_events::{
    callback_events, is_callback_event, is_restricted_event, restricted_events,
};

const POLICY: ValidationPolicy = ValidationPolicy {
    permissive: matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Wrath
            | crate::client_profile::ClientProfile::Mists
            | crate::client_profile::ClientProfile::Era
            | crate::client_profile::ClientProfile::Anniversary
    ),
    forever: matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::WowForever
    ),
    retail_12_0_0: {
        #[cfg(feature = "retail-12-0-0")]
        {
            true
        }
        #[cfg(not(feature = "retail-12-0-0"))]
        {
            false
        }
    },
    retail_12_0_5: {
        #[cfg(feature = "retail-12-0-5")]
        {
            true
        }
        #[cfg(not(feature = "retail-12-0-5"))]
        {
            false
        }
    },
    retail_12_0_7: {
        #[cfg(feature = "retail-12-0-7")]
        {
            true
        }
        #[cfg(not(feature = "retail-12-0-7"))]
        {
            false
        }
    },
    retail_12_1_0: {
        #[cfg(feature = "retail-12-1-0")]
        {
            true
        }
        #[cfg(not(feature = "retail-12-1-0"))]
        {
            false
        }
    },
    retail_12_1_5: {
        #[cfg(feature = "retail-12-1-5")]
        {
            true
        }
        #[cfg(not(feature = "retail-12-1-5"))]
        {
            false
        }
    },
};
// PTR Blizzard_APIDocumentationGenerated: SystemDocumentation.lua:86,177,189;
// VoiceChatDocumentation.lua:919; UnitDocumentation.lua:4290.
fn is_ptr_event(name: &str) -> bool {
    crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Ptr
        && matches!(
            name,
            "ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE"
                | "EXTERNAL_EVENT_LAUNCH_URL_RESULT"
                | "GAME_MODE_EXP_BAR_DATA_UPDATED"
                | "GAME_RULES_CHANGED"
                | "PLAYER_PVP_FLAG_CHANGED"
                | "SECURE_TRANSFER_CONFIRM_VC_PRODUCT_PURCHASE"
                | "SHARD_TRANSFER_CANCELLED"
                | "UNIT_PET_TRAINING_POINTS"
                | "VOICE_CHAT_VOICE_PROVIDER_CHANGED"
                | "GLOBAL_REGION_MOUSE_DOWN"
                | "SHARD_TRANSFER"
                | "SHARD_TRANSFER_IMMINENT"
                | "VOICE_CHAT_ACTIVE_VOICE_PROVIDER_CHANGED"
                | "UNIT_HAPPINESS"
        )
}

pub fn is_registerable_event(name: &str) -> bool {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
    ) && POLICY.retail_12_1_5
        && name == "INPUT_DEVICE_INTERFACE_TRANSITION"
    {
        return true;
    }
    is_ptr_event(name)
        || crate::c_api::forever_startup::accepts_event(name)
        || sim_event::is_registerable_event(name, POLICY)
}
pub fn is_valid_event(name: &str) -> bool {
    is_ptr_event(name)
        || crate::c_api::forever_startup::accepts_event(name)
        || sim_event::is_valid_event(name, POLICY)
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
