//! Original SettingsRegistrar consumer of the settings-loaded notification.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn settings_registrar_notifies_after_both_startup_events(env: &WowLuaEnv) {
    env.exec(r#"
        local source = debug.getinfo(SettingsRegistrar.OnLoad, 'S').source
        assert(source:find('Blizzard_SettingsRegistrar.lua', 1, true))
        local notifications = 0
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('SETTINGS_LOADED')
        observer:SetScript('OnEvent', function() notifications = notifications + 1 end)
        SettingsRegistrar:OnLoad()
        A_Admin.FireEvent('VARIABLES_LOADED')
        assert(notifications == 0)
        A_Admin.FireEvent('PLAYER_ENTERING_WORLD')
        assert(notifications == 1)
        assert(select('#', C_SettingsUtil.NotifySettingsLoaded()) == 0)
        assert(notifications == 2)
    "#).unwrap();
}
}
