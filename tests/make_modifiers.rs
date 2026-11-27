use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn make_modifiers_reads_modeled_keys() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    env.exec("A_Admin.SetShiftKeyDown(true); A_Admin.SetAltKeyDown(true)")
        .expect("modeled keys should be set");
    let shifted_alt: i64 = env.eval("return MakeModifiers()").expect("modifier mask");
    env.exec("A_Admin.SetShiftKeyDown(false); A_Admin.SetAltKeyDown(false); A_Admin.SetControlKeyDown(true)")
        .expect("modeled keys should change");
    let control: i64 = env.eval("return MakeModifiers()").expect("modifier mask");
    assert_eq!((shifted_alt, control), (5, 2));
}
