//! Current-retail absence, not historical signature or native parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
    for _, entry in ipairs({
        {C_MajorFactions, 'GetFeatureAbilities'},
        {C_MajorFactions, 'IsPlayerInRenownCatchUpMode'},
        {C_MajorFactions, 'RequestCatchUpState'},
        {C_Map, 'IsMapValidForNavBarDropDown'},
        {C_PvP, 'GetSoloRBGMinItemLevel'},
        {C_Scenario, 'GetCriteriaInfo'},
        {C_Scenario, 'GetCriteriaInfoByStep'},
        {C_Traits, 'GetStagedPurchases'},
        {C_TransmogSets, 'GetBaseSetsCounts'},
    }) do
        for i = 1, 2 do
            assert(entry[1][entry[2]] == nil, entry[2])
            assert(rawget(entry[1], entry[2]) == nil, entry[2])
        end
    end
    assert(type(C_MajorFactions.GetMajorFactionIDs) == 'function')
    assert(type(C_Map.GetMapInfo) == 'function')
    assert(type(C_Traits.GetNodeInfo) == 'function')
"#;

#[test]
fn patch_11_0_0_retired_members_do_not_regrow() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
