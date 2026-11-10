use super::{
    assert_ptr_source_omits_qualified_methods, load_full_game_ui_with_all_lod,
    missing_runtime_functions,
};

const PTR_METHODS: &[&str] = &[
    "SocialUIUtil.AddSeparatorToTooltip",
    "SocialUIUtil.GetBlockedName",
    "SocialUIUtil.GetIconForPresenceType",
    "SocialUIUtil.GetLabelForBattleNetFriendTag",
    "SocialUIUtil.GetLabelForPresenceType",
    "SocialUIUtil.GetPresenceTypeForBattleNetAccountInfo",
    "SocialUIUtil.GetPresenceTypeSelf",
    "SocialUIUtil.GetSupportedBattleNetFriendTagInterestsForCurrentGameType",
    "SocialUIUtil.GetSupportedBattleNetFriendTagRolesForCurrentGameType",
    "SocialUIUtil.InitializeUserScaledDropdownButton",
    "SocialUIUtil.InitializeUserScaledDropdownMainTitle",
    "SocialUIUtil.InitializeUserScaledDropdownTitle",
    "SocialUIUtil.SetBattleNetPresenceFromSocialUIPresence",
];

const REPLACED_ORDER_GETTERS: &[&str] = &[
    "GetBattleNetFriendTagInterestsUIOrder",
    "GetBattleNetFriendTagRoleUIOrder",
];

/// Proves PTR publishes the current tag-order accessors, not the former names.
#[test]
fn ptr_social_ui_uses_current_tag_order_accessors() {
    assert_ptr_source_omits_qualified_methods("SocialUIUtil", REPLACED_ORDER_GETTERS);

    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_METHODS);
    assert_eq!(
        missing, "",
        "pinned PTR SocialUIUtil methods must remain published"
    );

    let old_getters_present: bool = env
        .eval(
            r#"
            return SocialUIUtil.GetBattleNetFriendTagInterestsUIOrder ~= nil
                or SocialUIUtil.GetBattleNetFriendTagRoleUIOrder ~= nil
            "#,
        )
        .expect("previous tag-order getter probe succeeds");
    assert!(!old_getters_present);
}
