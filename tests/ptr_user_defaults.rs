//! PTR user-decided scenario defaults through selected Blizzard consumers.
#![cfg(feature = "client-ptr")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

#[test]
fn social_ui_loads_and_exposes_supported_tags() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SocialUIShared"], &[]);
    env.exec(r#"
        assert(type(SocialUIUtil.GetSupportedBattleNetFriendTagInterestsForCurrentGameType) == 'function')
        assert(type(SocialUIScrollableElementExtentPreviewerMixin) == 'table')
        local interests = SocialUIUtil.GetSupportedBattleNetFriendTagInterestsForCurrentGameType()
        assert(#interests == 7 and interests[1] == Enum.BattleNetFriendTag.Professions and interests[2] == Enum.BattleNetFriendTag.PvP)
        assert(C_BattleNet.IsFriendTagSupportedForCurrentGameType(0) == true)
        assert(C_BattleNet.IsFriendTagSupportedForCurrentGameType(999) == true)
    "#).expect("original SocialUIUtil chunk must finish and include supported tags");
}

#[test]
fn recent_allies_consumer_includes_supported_filters() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_RecentAllies"], &[]);
    env.exec(r#"
        local filters = RecentAlliesUtil.GetSupportedInteractionCategoryFiltersForCurrentGameType()
        assert(#filters == 6 and filters[1] == Enum.RecentAlliesInteractionCategoryFilter.Professions and filters[2] == Enum.RecentAlliesInteractionCategoryFilter.PvP)
        assert(C_RecentAllies.IsInteractionCategoryFilterSupportedForCurrentGameType(0) == true)
        assert(C_RecentAllies.IsInteractionCategoryFilterSupportedForCurrentGameType(999) == true)
    "#).expect("original RecentAlliesUtil must include supported filters");
}
