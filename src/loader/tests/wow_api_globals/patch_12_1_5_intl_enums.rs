//! Publication only; no Unicode, locale, security, or native semantics.

use crate::lua_api::WowLuaEnv;
use serde_json::Value;

const ENUM_NAMES: [&str; 8] = [
    "BreakType",
    "CollationStrength",
    "CurrencyNameStyle",
    "DateTimeStyle",
    "LocaleTransform",
    "NormalizationForm",
    "NumberStyle",
    "PluralType",
];

fn pinned_enums() -> Vec<Value> {
    let register: Value = serde_json::from_str(include_str!(
        "../../../../data/patch-api/sources/12.1.5-register.json"
    ))
    .unwrap();
    ENUM_NAMES
        .iter()
        .map(|name| {
            let symbol = format!("Enum.{name}");
            let row = register["occurrences"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["symbol"] == symbol)
                .unwrap();
            assert!(row["before"].is_null());
            row["after"].clone()
        })
        .collect()
}

fn assert_members(env: &WowLuaEnv, definition: &Value) {
    let name = definition["Name"].as_str().unwrap();
    for field in definition["Fields"].as_array().unwrap() {
        let member = field["Name"].as_str().unwrap();
        let actual: i64 = env.eval(&format!("return Enum.{name}.{member}")).unwrap();
        assert_eq!(
            actual,
            field["EnumValue"].as_i64().unwrap(),
            "{name}.{member}"
        );
    }
}

fn assert_metadata(env: &WowLuaEnv, definition: &Value) {
    let name = definition["Name"].as_str().unwrap();
    let actual: (i64, i64, i64, i64) = env
        .eval(&format!(
            "local count = 0; for _ in pairs(Enum.{name}) do count = count + 1 end; \
         return Enum.{name}Meta.MinValue, Enum.{name}Meta.MaxValue, \
         Enum.{name}Meta.NumValues, count"
        ))
        .unwrap();
    assert_eq!(
        actual,
        (
            definition["MinValue"].as_i64().unwrap(),
            definition["MaxValue"].as_i64().unwrap(),
            definition["NumValues"].as_i64().unwrap(),
            definition["Fields"].as_array().unwrap().len() as i64,
        ),
        "{name} metadata and count"
    );
}

fn assert_publication(env: &WowLuaEnv) {
    for definition in pinned_enums() {
        let name = definition["Name"].as_str().unwrap();
        if cfg!(feature = "client-ptr") {
            assert_members(env, &definition);
            assert_metadata(env, &definition);
        } else {
            let absent: bool = env
                .eval(&format!(
                    "return Enum.{name} == nil and Enum.{name}Meta == nil"
                ))
                .unwrap();
            assert!(absent, "{name} and metadata must remain absent");
        }
    }
}

#[test]
fn patch_12_1_5_intl_enums_publication() {
    let env = WowLuaEnv::new().unwrap();
    assert_publication(&env);
    crate::ptr::compat_bootstrap::apply_post_load(&env);
    assert_publication(&env);
}
