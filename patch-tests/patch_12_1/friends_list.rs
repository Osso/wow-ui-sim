use super::{load_full_game_ui_with_all_lod, missing_runtime_functions};

const PTR_METHODS: &[&str] = &[
    "FriendsListUtil.BuildCharacterClassDisplayText",
    "FriendsListUtil.BuildCharacterLevelDisplayText",
    "FriendsListUtil.BuildCharacterNameDisplayText",
    "FriendsListUtil.BuildFriendNameDisplayText",
    "FriendsListUtil.BuildLocationDisplayText",
    "FriendsListUtil.BuildTooltipBroadcastText",
    "FriendsListUtil.GameStateUsesFactions",
    "FriendsListUtil.GetBattleNetFriendGameAccountInfoIfExactlyOneDirectInviteTargetExists",
    "FriendsListUtil.GetBattleNetFriendInviteInfo",
    "FriendsListUtil.GetBattleNetFriendInviteTypeLabel",
    "FriendsListUtil.GetBattleNetFriendPartyInviteRestrictionText",
    "FriendsListUtil.GetBattleNetFriendPartyInviteRestriction",
    "FriendsListUtil.GetFormattedCharacterName",
    "FriendsListUtil.GetFriendAccountNameText",
    "FriendsListUtil.GetFriendNameColorForFriendType",
    "FriendsListUtil.GetFriendNameDisplayColor",
    "FriendsListUtil.GetFriendNameOfflineDisplayColor",
    "FriendsListUtil.GetGameAccountPartyInviteRestriction",
    "FriendsListUtil.GetLastOnlineText",
    "FriendsListUtil.GetRegionName",
    "FriendsListUtil.GetRelativeTimeText",
    "FriendsListUtil.HasMultipleGameAccounts",
    "FriendsListUtil.InviteOrRequestToJoin",
    "FriendsListUtil.IsPlayingDifferentWoWProject",
    "FriendsListUtil.IsPlayingSameWoWProject",
    "FriendsListUtil.IsPlayingWoW",
    "FriendsListUtil.IsRequestInviteType",
    "FriendsListUtil.IsTitleFriend",
    "FriendsListUtil.ShouldShowRichPresenceOnly",
];

/// Proves the pinned PTR FriendsListUtil methods are published after their LoD owners load.
#[test]
fn ptr_friends_list_methods_are_published_by_the_loaded_owner() {
    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_METHODS);
    assert_eq!(missing, "", "pinned PTR methods must remain published");
}
