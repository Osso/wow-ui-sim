//! Cached generated-documentation contracts for observed M3 consumers.
#![cfg(not(feature = "client-wrath"))]

use wow_ui_sim::lua_api::WowLuaEnv;

macro_rules! contract {
    ($name:ident, $lua:literal) => {
        #[test]
        fn $name() {
            WowLuaEnv::new().unwrap().exec($lua).unwrap();
        }
    };
}

#[cfg(not(feature = "client-retail"))]
mod non_retail_cross_faction_shim {
    use super::*;

    contract!(
        cross_faction_party_has_no_supplied_faction_roster,
        "assert(C_PartyInfo.IsCrossFactionParty() == false); assert(C_PartyInfo.IsCrossFactionParty(1) == false)"
    );
}
contract!(
    silence_policy_is_explicitly_inactive,
    "assert(C_SocialRestrictions.IsSilenced() == false)"
);
contract!(
    receive_chat_policy_is_explicitly_unrestricted,
    "assert(C_SocialRestrictions.CanReceiveChat() == true)"
);
contract!(
    lfd_service_is_explicitly_unavailable,
    "assert(C_LFGInfo.IsLFDEnabled() == false)"
);
contract!(
    group_finder_service_is_explicitly_unavailable,
    "assert(C_LFGInfo.IsGroupFinderEnabled() == false)"
);
contract!(
    lfg_authentication_is_explicitly_unavailable,
    "assert(C_LFGList.IsPlayerAuthenticatedForLFG() == false); assert(C_LFGList.IsPlayerAuthenticatedForLFG(2) == false)"
);

#[cfg(not(feature = "client-mists"))]
contract!(
    guild_bank_capability_is_explicitly_unavailable,
    "assert(C_GuildBank.IsGuildBankEnabled() == false)"
);
#[cfg(feature = "client-mists")]
contract!(
    guild_bank_keeps_existing_storage_capability,
    "assert(C_GuildBank.IsGuildBankEnabled() == true)"
);

#[cfg(any(
    feature = "profile-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
mod modern {
    use super::*;
    contract!(
        mentorship_has_no_membership_snapshot,
        "assert(C_PlayerMentorship.GetMentorshipStatus({}) == 0)"
    );
    contract!(
        language_capability_uses_explicit_legacy_language_defaults,
        "assert(C_ChatInfo.CanPlayerSpeakLanguage(1) == true); assert(C_ChatInfo.CanPlayerSpeakLanguage(999999) == false)"
    );
    contract!(
        recruit_service_is_explicitly_unavailable,
        "assert(C_RecruitAFriend.IsSystemEnabled() == false)"
    );
    contract!(
        legacy_friend_mode_is_explicitly_disabled,
        "assert(C_FriendList.IsLegacyFriendSystemEnabled() == false)"
    );
}

#[cfg(feature = "client-wowforever")]
contract!(
    friend_tag_game_type_capability_is_explicitly_unavailable,
    "assert(C_BattleNet.IsFriendTagSupportedForCurrentGameType(0) == false)"
);
