//! Current cached ping templates, not historical ping delivery/security parity.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_1_7_cached_ping_receiver_attributes(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(r#"
            local receiver = CreateFrame('Frame', nil, UIParent, 'PingReceiverAttributeTemplate')
            local passThrough = CreateFrame('Frame', nil, UIParent, 'PingTopLevelPassThroughAttributeTemplate')
            assert(receiver:GetAttribute('ping-receiver') == true)
            assert(passThrough:GetAttribute('ping-top-level-pass-through') == true)
            assert(receiver:GetAttribute('ping-top-level-pass-through') == nil)
            local unitFrame = CreateFrame('Frame', nil, UIParent, 'PingableUnitFrameTemplate')
            unitFrame:SetAttribute('unit', 'player')
            assert(unitFrame:GetAttribute('ping-receiver') == true)
            assert(unitFrame:GetIsPingable() == true)
            assert(unitFrame:GetAllowRadialWheel() == true)
            local info = unitFrame:GetTargetInfo()
            assert(info.guid == UnitGUID('player') and info.guid ~= nil)
            receiver:Hide()
            passThrough:Hide()
            unitFrame:Hide()
        "#).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
