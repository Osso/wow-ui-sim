//! Retail 12.1.0 intrinsic creation and combat construction proof.
//!
//! The cached Blizzard_AuraContainer XML declares AuraContainer inside a
//! `useForbiddenObjectTable allowUntaintedCreation` ScopedModifier and AuraButton
//! inside one without `allowUntaintedCreation`: addons create AuraContainers, but
//! "Addons no longer create AuraButtons directly."
#![cfg(feature = "retail-12-1-0")]

use crate::common::aura_container_harness::load_env;

#[test]
fn patch_12_1_0_aura_creation_in_combat_preserves_parent_and_frame_state() {
    let env = load_env();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(r#"
        local function addon()
            assert(InCombatLockdown())
            local parent = CreateFrame('Frame', nil, UIParent)
            local container = CreateFrame('AuraContainer', 'AuditCombatContainer', parent)
            assert(container:GetObjectType() == 'AuraContainer')
            assert(container:GetParent() == parent)
            local ok, err = pcall(CreateFrame, 'AuraButton', 'AuditAddonButton', container)
            assert(not ok and err:find('cannot be created by addons'), tostring(err))
            assert(AuditAddonButton == nil)
            assert(debug.getstacktaint() == 'AuditCombatAddon')
            return container
        end
        debug.setobjecttaint(addon, 'AuditCombatAddon')
        AuditCombatContainerRef = addon()
    "#).unwrap();
    // Secure code (Blizzard's frame providers) still creates AuraButtons.
    env.exec(r#"
        assert(issecure())
        local container = AuditCombatContainerRef
        local button = CreateFrame('AuraButton', 'AuditCombatButton', container)
        assert(button:GetObjectType() == 'AuraButton')
        assert(button:GetParent() == container)
        container:SetSize(137, 49)
        button:SetSize(23, 31)
        assert(container:GetWidth() == 137 and container:GetHeight() == 49)
        assert(button:GetWidth() == 23 and button:GetHeight() == 31)
        container:Hide()
        assert(not button:IsVisible())
        container:Show()
        assert(button:IsVisible())
    "#).unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
