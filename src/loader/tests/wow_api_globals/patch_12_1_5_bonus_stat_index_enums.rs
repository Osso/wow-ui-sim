//! Numeric enum publication only; no stat or gameplay semantics.

use crate::lua_api::WowLuaEnv;

fn assert_bonus_stat_publication(env: &WowLuaEnv, ptr: bool) {
    assert_pinned_base_members(env);
    assert_complete_reserved_range(env, ptr);
    assert_metadata_and_member_count(env, ptr);
}

fn assert_pinned_base_members(env: &WowLuaEnv) {
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    let parent = register["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "Enum.BonusStatIndex")
        .unwrap();
    let fields = parent["before"]["Fields"].as_array().unwrap();
    assert_eq!(fields.len(), 83);
    for field in fields {
        let name = field["Name"].as_str().unwrap();
        let expected = field["EnumValue"].as_i64().unwrap();
        let actual: i64 = env
            .eval(&format!("return Enum.BonusStatIndex.{name}"))
            .unwrap();
        assert_eq!(actual, expected, "preserved member {name}");
    }
}

fn assert_complete_reserved_range(env: &WowLuaEnv, ptr: bool) {
    env.exec(&format!(
        r#"
        for value = 83, 141 do
            local name = "Reserved_" .. value
            local expected = nil
            if {ptr} then expected = value end
            assert(Enum.BonusStatIndex[name] == expected,
                name .. ": expected " .. tostring(expected) .. ", got " .. tostring(Enum.BonusStatIndex[name]))
        end
        "#
    ))
    .expect("complete reserved range publication");
}

fn assert_metadata_and_member_count(env: &WowLuaEnv, ptr: bool) {
    let actual: (i32, i32, i32, i32) = env
        .eval(
            r#"
        local count = 0
        for _ in pairs(Enum.BonusStatIndex) do count = count + 1 end
        return Enum.BonusStatIndexMeta.MinValue, Enum.BonusStatIndexMeta.MaxValue,
            Enum.BonusStatIndexMeta.NumValues, count
        "#,
        )
        .unwrap();
    let count = if ptr { 142 } else { 83 };
    assert_eq!(actual, (0, count - 1, count, count));
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_bonus_stat_index_enums_ptr() {
    let env = WowLuaEnv::new().unwrap();
    assert_bonus_stat_publication(&env, true);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_bonus_stat_publication(&env, true);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_bonus_stat_index_enums_preserves_retail() {
    let env = WowLuaEnv::new().unwrap();
    assert_bonus_stat_publication(&env, false);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_bonus_stat_publication(&env, false);
}
