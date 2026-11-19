use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "client-wrath")]
fn test_tfilter_skips_nil_holes() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let result: String = env
        .eval(
            r#"
            local t = { 10, false, 30 }
            t[2] = nil
            local sawNil = false
            tFilter(t, function(v)
                if v == nil then
                    sawNil = true
                    return false
                end
                return true
            end, true)
            return tostring(sawNil) .. ":" .. tostring(#t) .. ":" .. tostring(t[1]) .. ":" .. tostring(t[2])
            "#,
        )
        .unwrap();
    assert_eq!(result, "false:2:10:30");
}

#[test]
#[cfg(not(feature = "client-wrath"))]
fn test_tfilter_skips_nil_holes() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_SharedXMLBase"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_SharedXMLBase"));
    let result: String = env
        .eval(
            r#"
            local input = { { value = 10 }, nil, { value = 30 } }
            local filtered = TableUtil.CopyUniqueByPredicate(input, true, function(value)
                return value.value
            end)
            return tostring(filtered ~= input) .. ":" .. tostring(filtered[1].value)
                .. ":" .. tostring(filtered[2]) .. ":" .. tostring(input[3].value)
            "#,
        )
        .unwrap();
    assert_eq!(result, "true:10:nil:30");
}
