//! Current cached ping templates, not historical ping delivery/security parity.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
    fn patch_10_1_7_cached_ping_receiver_attributes(env: &WowLuaEnv) {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(r#"
            local receiver = CreateFrame('Frame', nil, UIParent, 'PingReceiverAttributeTemplate')
            local passThrough = CreateFrame('Frame', nil, UIParent, 'PingTopLevelPassThroughAttributeTemplate')
            assert(receiver:GetAttribute('ping-receiver') == true, 'receiver attribute')
            assert(passThrough:GetAttribute('ping-top-level-pass-through') == true, 'pass-through attribute')
            assert(receiver:GetAttribute('ping-top-level-pass-through') == nil, 'receiver pass-through isolation')
            local unitFrame = CreateFrame('Frame', nil, UIParent, 'PingableUnitFrameTemplate')
            unitFrame:SetAttribute('unit', 'player')
            assert(unitFrame:GetAttribute('ping-receiver') == true, 'inherited receiver attribute')
            assert(unitFrame:GetIsPingable() == true, 'unit ping eligibility')
            assert(unitFrame:GetAllowRadialWheel() == true, 'unit radial eligibility')
            local info = unitFrame:GetTargetInfo()
            assert(info.guid == UnitGUID('player') and info.guid ~= nil, 'unit target GUID')
            receiver:Hide()
            passThrough:Hide()
            unitFrame:Hide()
        "#).unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    }
}
