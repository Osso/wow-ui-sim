//! Original personal-resource display consumer of the game-rule predicate.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn personal_resource_display_shown_state_uses_game_rule_and_cvar(env: &WowLuaEnv) {
    env.exec(r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_PersonalResourceDisplay'))
        local frame = assert(PersonalResourceDisplayFrame)
        local source = debug.getinfo(PersonalResourceDisplayMixin.UpdateShownState, 'S').source
        assert(source:find('Blizzard_PersonalResourceDisplay.lua', 1, true))
        frame.isInEditMode = false
        frame.visibleSetting = Enum.PersonalResourceDisplayVisibleSetting.Always
        SetCVar('nameplateShowSelf', '0')
        A_Admin.SetGameRule('PersonalResourceDisplayDisabled', false)
        frame:UpdateShownState()
        assert(not frame:IsShown())
        SetCVar('nameplateShowSelf', '1')
        frame:UpdateShownState()
        assert(frame:IsShown())
        A_Admin.SetGameRule('PersonalResourceDisplayDisabled', 1)
        frame:UpdateShownState()
        assert(not frame:IsShown())
    "#).unwrap();
}
}
