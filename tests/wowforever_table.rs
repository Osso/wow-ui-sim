//! Forever's real TableUtil consumers of native table extensions.

#[test]
#[cfg(feature = "client-wowforever")]
fn wowforever_table_util_uses_native_extensions() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let source = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_SharedXMLBase/TableUtil.lua");
    env.exec(&std::fs::read_to_string(source).expect("synced Forever TableUtil.lua"))
        .unwrap();
    env.exec(
        r#"
        local entries = {"alpha", "beta", named = false}
        assert(tIndexOf(entries, "beta") == 2)
        assert(tContains(entries, false))
        assert(CountTable(entries) == 3)
        assert(not TableIsEmpty(entries) and TableIsEmpty({}))
        assert(TableUtil.SafeCountTable(entries, false) == 3)
        local keys = GetKeysArray(entries)
        assert(#keys == 3 and tContains(keys, "named"))
        local values = GetValuesArray(entries)
        assert(#values == 3 and tContains(values, false))
        local array = {"a", "b", "c"}
        tUnorderedRemove(array, 1)
        assert(#array == 2 and array[1] == "c" and array[2] == "b")
        local repeated = {"a", "b", "a"}
        assert(tDeleteItem(repeated, "a") == 2)
        assert(#repeated == 1 and repeated[1] == "b")
        "#,
    )
    .unwrap();
}

#[test]
#[cfg(not(any(feature = "retail-12-1-5", feature = "client-wowforever")))]
fn wowforever_table_does_not_leak_into_earlier_profiles() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({"indexof", "contains", "count", "isempty",
            "removeunordered", "removevalue", "keys", "values"}) do
            assert(table[name] == nil, name)
        end
        "#,
    )
    .unwrap();
}
