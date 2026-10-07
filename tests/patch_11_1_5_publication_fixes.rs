//! Bounded current simulator contracts, not native 11.1.5 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_1_5_retired_members_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local retired = {
            {C_GameEnvironmentManager, 'GetCurrentEventRealmQueues'},
            {C_GameEnvironmentManager, 'GetCurrentGameEnvironment'},
            {C_GameEnvironmentManager, 'RequestGameEnvironment'},
            {C_GameModeManager, 'GetCurrentGameModeRecordID'},
            {C_GameModeManager, 'GetCurrentGameMode'},
            {C_GameModeManager, 'GetGameModeDisplayInfo'},
            {C_SpellBook, 'GetTrackedNameplateCooldownSpells'},
        }
        for _, entry in ipairs(retired) do
            for i = 1, 2 do
                assert(entry[1][entry[2]] == nil, entry[2])
                assert(rawget(entry[1], entry[2]) == nil, entry[2])
            end
        end
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_1_5_global_environment_tracks_secure_swap_and_custom_environment() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(IsInGlobalEnvironment())").unwrap();
    env.exec_rilua_secure(
        r#"
        assert(not IsInGlobalEnvironment())
        SwapToGlobalEnvironment()
        assert(IsInGlobalEnvironment())
        "#,
    )
    .unwrap();
    env.exec(
        r#"
        local function probe() return IsInGlobalEnvironment() end
        setfenv(probe, setmetatable({}, {__index = _G}))
        assert(not probe())
        assert(IsInGlobalEnvironment())
        "#,
    )
    .unwrap();
}
