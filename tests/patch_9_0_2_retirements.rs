//! Removed members must not be fabricated on ordinary or repeated lookup.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
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
    assert(rawget(namespace, member) == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
    assert(namespace[member] == nil, entry[1] .. "." .. member)
end
"#;

#[test]
fn patch_9_0_2_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}

prefork_full_ui_case! {
fn patch_9_0_2_cached_retirements(env: &WowLuaEnv) {
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
}
