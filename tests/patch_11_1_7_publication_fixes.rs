//! Bounded current simulator contracts, not native 11.1.7 parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_11_1_7_debug_retirements_survive_repeated_lookup() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for i = 1, 2 do
            assert(C_Debug.PrintToDebugWindow == nil)
            assert(C_Debug.ViewInDebugWindow == nil)
        end
        assert(type(debugprofilestart) == 'function')
        assert(type(debugprofilestop) == 'function')
        debugprofilestart()
        assert(debugprofilestop() >= 0)
        "#,
    )
    .unwrap();
}

#[test]
fn patch_11_1_7_graphics_commands_are_catalog_records_not_cvars() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'GxMemReport', 'GxSimulateEvent'}) do
            local count = 0
            for _, record in ipairs(C_Console.GetAllCommands()) do
                if record.command == name then
                    count = count + 1
                    assert(record.commandType == Enum.ConsoleCommandType.Command)
                    assert(record.category == Enum.ConsoleCategory.None)
                end
            end
            assert(count == 1, name)
            assert(C_CVar.GetCVar(name) == nil)
            assert(C_CVar.GetCVarDefault(name) == nil)
        end
        "#,
    )
    .unwrap();
}
