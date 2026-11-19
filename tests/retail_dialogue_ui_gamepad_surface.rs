//! Retail gamepad members used by DialogueUI with no simulated devices.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_gamepad_queries_use_empty_device_state() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_GamePad) == "table")
        local ids = C_GamePad.GetAllDeviceIDs()
        assert(type(ids) == "table" and next(ids) == nil)
        assert(C_GamePad.GetDeviceRawState(1) == nil)
        assert(C_GamePad.IsEnabled() == GetCVarBool("GamePadEnable"))
        SetCVar("GamePadEnable", "1")
        assert(C_GamePad.IsEnabled())
        "#,
    )
    .unwrap();
}
