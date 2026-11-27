use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn callstack_height_tracks_nested_lua_calls() {
    let env = WowLuaEnv::new().expect("lua env should initialize");
    let (shallow, nested): (i64, i64) = env
        .eval(
            r#"
            local shallow = GetCallstackHeight()
            local function nested_call() return GetCallstackHeight() end
            return shallow, nested_call()
            "#,
        )
        .expect("nested stack heights should be readable");
    assert!(nested > shallow, "nested={nested}, shallow={shallow}");
}
