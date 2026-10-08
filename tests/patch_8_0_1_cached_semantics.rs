//! Bounded cached-UI proof for the two explicit 8.0.1 input changes.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_8_0_1_cached_event_and_aura_input_changes(env: &WowLuaEnv) {
    env.eval::<()>(r#"
        local frame = CreateFrame('Frame')
        assert(not pcall(frame.RegisterEvent, frame, 'P801_NOT_A_REAL_EVENT'))
        assert(not frame:IsEventRegistered('P801_NOT_A_REAL_EVENT'))
        assert(pcall(frame.RegisterEvent, frame, 'UNIT_POWER_UPDATE'))
        assert(frame:IsEventRegistered('UNIT_POWER_UPDATE'))
        frame:UnregisterEvent('UNIT_POWER_UPDATE')
        assert(not pcall(UnitAura, 'player', 'P801_NOT_AN_AURA_NAME'))
        assert(type(AuraUtil.FindAuraByName) == 'function')
        assert(AuraUtil.FindAuraByName('P801_NOT_AN_AURA_NAME', 'player', 'HELPFUL') == nil)
    "#).unwrap();
}
}
