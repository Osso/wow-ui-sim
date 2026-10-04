//! Retail 12.1.0 intrinsic creation and combat construction proof.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_1_0_aura_creation_in_combat_preserves_parent_and_frame_state() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(r#"
        local function addon()
            assert(InCombatLockdown())
            local parent = CreateFrame('Frame', nil, UIParent)
            local container = CreateFrame('AuraContainer', 'AuditCombatContainer', parent)
            local button = CreateFrame('AuraButton', 'AuditCombatButton', container)
            assert(container:GetObjectType() == 'AuraContainer')
            assert(button:GetObjectType() == 'AuraButton')
            assert(container:GetParent() == parent and button:GetParent() == container)
            container:SetSize(137, 49)
            button:SetSize(23, 31)
            assert(container:GetWidth() == 137 and container:GetHeight() == 49)
            assert(button:GetWidth() == 23 and button:GetHeight() == 31)
            container:Hide()
            assert(not button:IsVisible())
            container:Show()
            assert(button:IsVisible())
            assert(debug.getstacktaint() == 'AuditCombatAddon')
        end
        debug.setobjecttaint(addon, 'AuditCombatAddon')
        addon()
        assert(issecure())
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
