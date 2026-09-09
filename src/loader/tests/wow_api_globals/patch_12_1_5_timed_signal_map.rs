//! Earlier-retail publication only; no timed-signal access or security claims.

#[test]
fn timed_signal_map_factory_remains_absent_on_retail() {
    let env = crate::lua_api::WowLuaEnv::new().unwrap();
    let lookup: (String, String) = env
        .eval("return type(rawget(C_Timer, 'NewTimedSignalMap')), type(C_Timer.NewTimedSignalMap)")
        .unwrap();
    assert_eq!(lookup, ("nil".to_owned(), "nil".to_owned()));
    env.exec("assert(C_Timer.NewTimedSignalMap == nil)")
        .unwrap();
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    env.exec("assert(C_Timer.NewTimedSignalMap == nil)")
        .unwrap();
}
