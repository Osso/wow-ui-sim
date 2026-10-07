//! Unused retail namespace retirement; cached wrappers and classic stay unchanged.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
local removed = {
    { C_CharacterServices, 'AssignPFCDistribution' },
    { C_LootHistory, 'CanMasterLoot' },
    { C_TooltipInfo, 'GetQuestLogRewardSpell' },
}
for _, entry in ipairs(removed) do
    local namespace, member = entry[1], entry[2]
    assert(rawget(namespace, member) == nil, member)
    assert(namespace[member] == nil, member)
    assert(namespace[member] == nil, member)
end
assert(type(C_LootHistory.GetAllEncounterInfos) == 'function')
assert(type(C_TooltipInfo.GetSpellByID) == 'function')
"#;

#[test]
fn patch_10_1_0_unused_namespace_members_are_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
