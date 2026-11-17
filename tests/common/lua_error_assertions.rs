use wow_ui_sim::lua_api::WowLuaEnv;

pub fn read_lua_error(env: &WowLuaEnv, call: &str) -> String {
    env.eval(&format!(
        "local ok, err = pcall(function() {call} end); assert(not ok); return err"
    ))
    .expect("pcall should return the Lua error")
}

/// Assert stable error fragments without matching runtime caller-name wrappers.
pub fn assert_lua_error_contains(env: &WowLuaEnv, call: &str, fragments: &[&str]) {
    let actual = read_lua_error(env, call);
    for fragment in fragments {
        assert!(
            actual.contains(fragment),
            "{call}: missing {fragment:?} in {actual:?}"
        );
    }
}
