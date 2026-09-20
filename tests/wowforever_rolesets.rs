//! Forever CombatLog roleset publication and membership.
#![cfg(feature = "client-wowforever")]

#[test]
fn combat_log_rolesets_preserve_membership_and_frame_isolation() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local button = CreateFrame("Button", "CombatLogQuickButtonFrame")
        assert(type(getmetatable(button).__index.SetRolesets) == "function")
        button:SetRolesets("chat")
        assert(button:GetRolesetNames().chat == true)
        button:AddRoleset("combat")
        assert(button:GetRolesetNames().combat == true)
        button:RemoveRoleset("chat")
        assert(button:GetRolesetNames().chat == nil)
        button:SetRolesets("chat", "log")
        assert(button:GetRolesetNames().chat == true)
        assert(button:GetRolesetNames().log == true)
        assert(button:GetRolesetNames().combat == nil)
        local other = CreateFrame("Frame")
        assert(next(other:GetRolesetNames()) == nil)
        button:SetRolesets()
        assert(next(button:GetRolesetNames()) == nil)
    "#,
    )
    .unwrap();
}
