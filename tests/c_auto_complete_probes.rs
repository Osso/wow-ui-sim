use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[cfg(not(feature = "retail-12-0-7"))]
#[test]
fn autocomplete_realms_are_empty_for_namespace_and_global_callers() {
    let env = env();
    let (namespace_count, global_count, namespace_type, global_type): (i32, i32, String, String) = env
        .eval(
            r#"
            local namespaceRealms = C_AutoComplete.GetAutoCompleteRealms()
            local globalRealms = GetAutoCompleteRealms()
            return #namespaceRealms, #globalRealms,
                type(C_AutoComplete.GetAutoCompleteRealms),
                type(GetAutoCompleteRealms)
            "#,
        )
        .expect("autocomplete realm queries should be callable");

    assert_eq!(namespace_count, 0);
    assert_eq!(global_count, 0);
    assert_eq!(namespace_type, "function");
    assert_eq!(global_type, "function");
}

#[cfg(feature = "retail-12-0-7")]
#[test]
fn autocomplete_namespace_remains_live_without_retired_native_globals() {
    let env = env();
    env.exec(r#"
        assert(rawget(_G, "GetAutoCompleteRealms") == nil)
        assert(rawget(_G, "GetAutoCompleteResults") == nil)
        assert(type(C_AutoComplete.GetAutoCompleteRealms) == "function")
        assert(#C_AutoComplete.GetAutoCompleteRealms() == 0)
    "#).unwrap();
}
