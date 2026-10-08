//! Classic lookup retains its pre-existing namespace compatibility behavior.
#![cfg(feature = "client-mists")]

#[test]
fn patch_9_0_2_mists_members_preserved() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
for _, entry in ipairs({
    {'C_CovenantSanctumUI', 'GetRenownMilestones'},
    {'C_Soulbinds', 'AddPendingConduit'},
    {'C_Soulbinds', 'GetPendingConduitID'},
    {'C_Soulbinds', 'GetPendingNodeIDInSoulbind'},
    {'C_Soulbinds', 'HasPendingConduitInSoulbind'},
    {'C_Soulbinds', 'RemovePendingConduit'},
    {'C_Soulbinds', 'ResetSoulbindConduits'},
    {'C_Spell', 'GetMawPowerRarityStringAndBorderAtlasBySpellID'},
    {'C_UIWidgetManager', 'GetWidgetLayoutDirectionFromWidgetSetID'},
}) do
    local namespace, member = _G[entry[1]], entry[2]
    assert(type(namespace[member]) == "function", entry[1] .. "." .. member)
    assert(type(namespace[member]) == "function", entry[1] .. "." .. member)
end
"#).unwrap();
}
