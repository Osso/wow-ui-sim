use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn lifetime_stats_read_each_supplied_field() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    env.exec("A_Admin.SetPVPLifetimeStats(91, 7, 10)")
        .expect("first lifetime snapshot should be seeded");
    let first: (i64, i64, i64) = env
        .eval("return GetPVPLifetimeStats()")
        .expect("first lifetime snapshot should be readable");
    env.exec("A_Admin.SetPVPLifetimeStats(22, 1, 4)")
        .expect("second lifetime snapshot should be seeded");
    let second: (i64, i64, i64) = env
        .eval("return GetPVPLifetimeStats()")
        .expect("second lifetime snapshot should be readable");
    assert_eq!(first, (91, 7, 10));
    assert_eq!(second, (22, 1, 4));
}
