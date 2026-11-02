//! Original Friends tab consumer of Recent Allies availability.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn friends_tab_visibility_follows_recent_allies_input(env: &WowLuaEnv) {
    env.state().borrow_mut().recent_allies.enabled = false;
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_FriendsFrame'))
        local source = debug.getinfo(FriendsTabHeaderMixin.RefreshTabVisibility, 'S').source
        assert(source:find('FriendsFrame.lua', 1, true))
        FriendsTabHeader:RefreshTabVisibility()
        local tab = assert(FriendsTabHeader:GetTabButton(FriendsTabHeader.recentAlliesTabID))
        assert(not tab:IsShown())
    "#).unwrap();
    env.state().borrow_mut().recent_allies.enabled = true;
    env.exec(r#"
        FriendsTabHeader:RefreshTabVisibility()
        local tab = assert(FriendsTabHeader:GetTabButton(FriendsTabHeader.recentAlliesTabID))
        assert(tab:IsShown())
    "#).unwrap();
    env.state().borrow_mut().recent_allies.enabled = false;
    env.exec(r#"
        FriendsTabHeader:RefreshTabVisibility()
        assert(not FriendsTabHeader:GetTabButton(FriendsTabHeader.recentAlliesTabID):IsShown())
    "#).unwrap();
}
}
