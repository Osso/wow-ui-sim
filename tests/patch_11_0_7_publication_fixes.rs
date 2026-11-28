//! Bounded simulator state and absence contracts, not native 11.0.7 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_0_7_retired_members_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, entry in ipairs({
            {C_ArrowCalloutManager, 'HideWorldLootObjectCallout'},
            {C_ArrowCalloutManager, 'SetWorldLootObjectCalloutFromGUID'},
            {C_ArrowCalloutManager, 'SwapWorldLootObjectCallout'},
            {C_WorldLootObject, 'GetCurrentWorldLootObjectSwapInventoryType'},
        }) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        assert(type(C_ArrowCalloutManager.AcknowledgeCallout) == 'function')
        "#,
    )
    .unwrap();
}
