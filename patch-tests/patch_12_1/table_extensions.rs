use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn table_extensions_preserve_mixed_table_and_array_contracts() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    let (
        first,
        past_nil,
        contains,
        count,
        empty_before,
        empty_after,
        keys,
        values,
        removed,
        default_removed,
        outside_removed,
        array,
        removed_count,
    ): (
        Option<f64>,
        Option<f64>,
        bool,
        f64,
        bool,
        bool,
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        f64,
    ) = env
        .eval(
            r#"
        local mixed = { "a", false, "c", flag = true }
        local keys = table.keys(mixed)
        local values = table.values(mixed)
        local key_text, value_text = {}, {}
        for _, key in ipairs(keys) do key_text[tostring(key)] = true end
        for _, value in ipairs(values) do value_text[tostring(value)] = true end

        local array = { "a", "b", "c", "b" }
        local removed = table.removeunordered(array, 2)
        local defaultRemoved = table.removeunordered({ "last" })
        local outsideRemoved = table.removeunordered(array, 99)
        local removed_count = table.removevalue(array, "b")
        local empty = {}
        return table.indexof(mixed, false),
            table.indexof({ "a", nil, "c" }, "c"),
            table.contains(mixed, false),
            table.count(mixed),
            table.isempty(empty),
            table.isempty(mixed),
            tostring(key_text["1"] and key_text["2"] and key_text["3"] and key_text.flag),
            tostring(value_text.a and value_text["false"] and value_text.c and value_text["true"]),
            removed,
            defaultRemoved,
            outsideRemoved,
            table.concat(array, ","),
            removed_count
    "#,
        )
        .expect("table extension behavior");

    assert_eq!(first, Some(2.0));
    assert_eq!(past_nil, None);
    assert!(contains);
    assert_eq!(count, 4.0);
    assert!(empty_before);
    assert!(!empty_after);
    assert_eq!(keys, "true");
    assert_eq!(values, "true");
    assert_eq!(removed, "b");
    assert_eq!(default_removed, "last");
    assert_eq!(outside_removed, None);
    assert_eq!(array, "a,c");
    assert_eq!(removed_count, 1.0);
}
