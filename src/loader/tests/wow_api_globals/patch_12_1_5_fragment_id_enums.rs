//! Exact enum publication only; no FragmentID consumer semantics.

use crate::lua_api::WowLuaEnv;

fn assert_fragment_publication(env: &WowLuaEnv, expected: &str, count: usize) {
    env.exec(&format!(
        r#"
        local expected = {{ {expected} }}
        assert(type(Enum.FragmentID) == "table")
        for name, value in pairs(expected) do
            assert(Enum.FragmentID[name] == value,
                name .. ": expected " .. value .. ", got " .. tostring(Enum.FragmentID[name]))
        end
        local count = 0
        for name in pairs(Enum.FragmentID) do
            assert(expected[name] ~= nil, "unexpected member: " .. name)
            count = count + 1
        end
        assert(count == {count}, "member count mismatch")
        "#,
    ))
    .expect("exact FragmentID publication");
}

fn assert_fragment_metadata(env: &WowLuaEnv, count: i32) {
    let actual: (i32, i32, i32, i32) = env
        .eval(
            r#"
            local count = 0
            for _ in pairs(Enum.FragmentIDMeta) do count = count + 1 end
            return Enum.FragmentIDMeta.MinValue, Enum.FragmentIDMeta.MaxValue,
                Enum.FragmentIDMeta.NumValues, count
            "#,
        )
        .expect("FragmentID metadata");
    assert_eq!(actual, (0, 255, count, 3));
}

/// Frozen semantic fixture: Gethe a89e9d0c (`before`, current retail) -> 49b69918 (`after`).
fn register_fields(side: &str) -> (String, usize) {
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    let fields = register["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "Enum.FragmentID")
        .unwrap()[side]["Fields"]
        .as_array()
        .unwrap();
    let expected = fields
        .iter()
        .map(|field| {
            format!(
                "{} = {},",
                field["Name"].as_str().unwrap(),
                field["EnumValue"].as_i64().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    (expected, fields.len())
}

fn assert_publication_survives_post_load(side: &str, count: usize) {
    let (expected, fields) = register_fields(side);
    assert_eq!(fields, count);
    let env = WowLuaEnv::new().unwrap();
    assert_fragment_publication(&env, &expected, count);
    assert_fragment_metadata(&env, count as i32);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_fragment_publication(&env, &expected, count);
    assert_fragment_metadata(&env, count as i32);
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_fragment_id_enums_ptr() {
    assert_publication_survives_post_load("after", 78);
}

// Cached retail WowCSConstantsDocumentation.lua:6–91 equals the register's `before`.
#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_fragment_id_enums_preserves_retail() {
    assert_publication_survives_post_load("before", 77);
}
