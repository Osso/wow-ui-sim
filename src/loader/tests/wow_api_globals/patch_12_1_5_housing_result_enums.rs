//! Exact pinned enum publication; no housing gameplay or security semantics.

use crate::lua_api::WowLuaEnv;

fn pinned_housing_result(ptr: bool) -> serde_json::Value {
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    let parent = register["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["symbol"] == "Enum.HousingResult")
        .unwrap();
    parent[if ptr { "after" } else { "before" }].clone()
}

fn assert_housing_members(env: &WowLuaEnv, expected: &serde_json::Value) {
    for field in expected["Fields"].as_array().unwrap() {
        let name = field["Name"].as_str().unwrap();
        let value = field["EnumValue"].as_i64().unwrap();
        let actual: Option<i64> = env
            .eval(&format!("return Enum.HousingResult.{name}"))
            .unwrap();
        assert_eq!(actual, Some(value), "HousingResult.{name}");
    }
}

fn assert_housing_metadata(env: &WowLuaEnv, expected: &serde_json::Value) {
    let actual: (i64, i64, i64, i64) = env
        .eval(
            r#"
            local count = 0
            for _ in pairs(Enum.HousingResult) do count = count + 1 end
            local meta = Enum.HousingResultMeta
            return meta.MinValue, meta.MaxValue, meta.NumValues, count
            "#,
        )
        .unwrap();
    let count = expected["Fields"].as_array().unwrap().len() as i64;
    assert_eq!(
        actual,
        (
            expected["MinValue"].as_i64().unwrap(),
            expected["MaxValue"].as_i64().unwrap(),
            expected["NumValues"].as_i64().unwrap(),
            count,
        )
    );
}

fn assert_housing_publication(ptr: bool) {
    let expected = pinned_housing_result(ptr);
    assert_eq!(
        expected["Fields"].as_array().unwrap().len(),
        if ptr { 113 } else { 112 }
    );
    let env = WowLuaEnv::new().unwrap();
    for post_load in [false, true] {
        if post_load {
            crate::ptr::compat_bootstrap::apply_post_load(&env);
        }
        assert_housing_members(&env, &expected);
        assert_housing_metadata(&env, &expected);
        let message: Option<i64> = env
            .eval("return Enum.HousingResult.MessageTooLong")
            .unwrap();
        assert_eq!(message, if ptr { Some(71) } else { None });
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn patch_12_1_5_housing_result_enums_ptr() {
    assert_housing_publication(true);
}

#[cfg(feature = "client-retail")]
#[test]
fn patch_12_1_5_housing_result_enums_preserves_retail() {
    assert_housing_publication(false);
}
