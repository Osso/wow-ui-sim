use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("create Lua env")
}

#[test]
fn unit_level_non_attackable_color_is_available() {
    let env = env();
    let result: (f64, f64, f64) = env
        .eval("return UNIT_LEVEL_NON_ATTACKABLE.r, UNIT_LEVEL_NON_ATTACKABLE.g, UNIT_LEVEL_NON_ATTACKABLE.b")
        .expect("read unit-level color");

    assert_eq!(result, (1.0, 1.0, 0.0));
}
