//! Forever CombatLog roleset publication and membership.
#![cfg(all(feature = "client-wowforever", feature = "optional-tests"))]

#[test]
fn combat_log_rolesets_preserve_membership_and_frame_isolation() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local button = CreateFrame("Frame", "CombatLogQuickButtonFrame", UIParent)
        assert(type(getmetatable(button).__index.SetRolesets) == "function", "SetRolesets is published")

        -- Blizzard_CombatLog assigns this exact roleset to its quick-button frame.
        button:SetRolesets("chat")
        local names = button:GetRolesetNames()
        assert(#names == 1 and names[1] == "chat" and names.chat == nil, "roleset result is an array")

        button:AddRoleset("combat")
        names = button:GetRolesetNames()
        assert(#names == 2, "AddRoleset adds one name")
        assert((names[1] == "chat" and names[2] == "combat")
            or (names[1] == "combat" and names[2] == "chat"), "multiple names use array keys")

        button:RemoveRoleset("chat")
        names = button:GetRolesetNames()
        assert(#names == 1 and names[1] == "combat", "RemoveRoleset removes one name")

        button:SetRolesets(" chat, ,log ")
        names = button:GetRolesetNames()
        assert(#names == 2, "SetRolesets parses two nonempty names")
        assert((names[1] == "chat" and names[2] == "log")
            or (names[1] == "log" and names[2] == "chat"), "SetRolesets trims spaces and skips empty parts")
        assert(names.chat == nil and names.log == nil and names.combat == nil, "roleset names are not set keys")

        local other = CreateFrame("Frame")
        other:SetRolesets("other")
        C_Roleset.ApplyRolesetFilters({ "chat" }, {})
        assert(not button:IsVisible() and other:IsVisible(), "chat filtering affects only tagged consumer")

        button:SetRolesets("")
        C_Roleset.ApplyRolesetFilters({ "roleless" }, {})
        assert(button:IsVisible(), "clearing does not store roleless")

        -- INFERRED: the optional-string convention rejects a table with the verified usage text.
        local ok, message = pcall(function() button:SetRolesets({}) end)
        assert(not ok and message == "Usage: self:SetRolesets([rolesetsString])", "SetRolesets reports the verified usage text")
    "#,
    )
    .unwrap();
}
