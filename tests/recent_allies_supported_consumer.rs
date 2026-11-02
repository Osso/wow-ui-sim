//! Original Social UI tab filter consumer of Recent Allies support.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn social_ui_filters_recent_allies_by_supplied_support_inputs(env: &WowLuaEnv) {
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.recent_allies.enabled = true;
        sim.recent_allies.support_feature_enabled = false;
        sim.recent_allies.support_excluded = false;
    }
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_SocialUI'), 'SocialUI addon not loaded')
        local source = debug.getinfo(SocialUIFrameMixin.FilterAvailableTabData, 'S').source
        assert(source:find('SocialUI.lua', 1, true), 'unexpected tab filter source')
        local tab
        for _, value in pairs(SocialUIFrame.tabDefinitions) do
            if value.socialSystem == Enum.SocialSystemType.RecentAllies then tab = value end
        end
        assert(tab and tab.IsAvailable(), 'Recent Allies tab is absent or unavailable')
        assert(tab.IsSupported == C_RecentAllies.IsSystemSupported, 'tab did not capture supported query')
        assert(#SocialUIFrame:FilterAvailableTabData({tab}) == 0, 'disabled support did not filter tab')
    "#).unwrap();
    env.state().borrow_mut().recent_allies.support_feature_enabled = true;
    env.exec(r#"
        local tab
        for _, value in pairs(SocialUIFrame.tabDefinitions) do
            if value.socialSystem == Enum.SocialSystemType.RecentAllies then tab = value end
        end
        assert(#SocialUIFrame:FilterAvailableTabData({tab}) == 1)
    "#).unwrap();
    env.state().borrow_mut().recent_allies.support_excluded = true;
    env.exec(r#"
        local tab
        for _, value in pairs(SocialUIFrame.tabDefinitions) do
            if value.socialSystem == Enum.SocialSystemType.RecentAllies then tab = value end
        end
        assert(#SocialUIFrame:FilterAvailableTabData({tab}) == 0)
    "#).unwrap();
}
}
