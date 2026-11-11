#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn get_pvp_talent_is_registered_on_retail() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let getter_type: String = env
        .eval(
            r#"
            local namespace = rawget(_G, "C_TooltipInfo")
            return type(namespace and rawget(namespace, "GetPvpTalent"))
            "#,
        )
        .expect("GetPvpTalent registration probe should run");

    assert_eq!(getter_type, "function");
}
