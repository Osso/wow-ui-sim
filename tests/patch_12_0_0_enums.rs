//! Frozen 12.0.0 extract rows against current cached Retail numeric publication.
//! Later epoch replacements are recorded in the fixture, not mistaken for gaps.
#![cfg(feature = "client-retail")]

use serde::Deserialize;
use std::collections::BTreeMap;
use wow_ui_sim::lua_api::WowLuaEnv;

#[derive(Deserialize)]
struct Data {
    enums: BTreeMap<String, DocEnum>,
    rows: BTreeMap<String, Row>,
}

#[derive(Deserialize)]
struct DocEnum {
    source: String,
    meta: BTreeMap<String, i64>,
    values: BTreeMap<String, i64>,
}

#[derive(Deserialize)]
struct Row {
    #[serde(rename = "enum")]
    name: String,
    member: Option<String>,
    value: Option<i64>,
    #[serde(default)]
    absent: Vec<String>,
    superseded_by: Option<String>,
}

fn check_value(env: &WowLuaEnv, name: &str, key: &str, value: Option<i64>) -> Result<(), String> {
    let expected = value.map_or("nil".into(), |n| n.to_string());
    let code = format!(
        r#"
        local parent = rawget(Enum, {name:?})
        if type(parent) ~= 'table' or Enum[{name:?}] ~= parent then return 'missing parent' end
        local raw, ordinary = rawget(parent, {key:?}), parent[{key:?}]
        return type(raw) .. ':' .. tostring(raw) .. '/' .. type(ordinary) .. ':' .. tostring(ordinary)
    "#
    );
    let observed: String = env.eval(&code).map_err(|e| e.to_string())?;
    let value_type = if value.is_some() { "number" } else { "nil" };
    let expected = format!("{value_type}:{expected}/{value_type}:{expected}");
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{name}.{key}: expected {expected}, observed {observed}"
        ))
    }
}

fn check_parent(env: &WowLuaEnv, name: &str, doc: &DocEnum) -> Vec<String> {
    let mut failures = Vec::new();
    for (key, value) in &doc.values {
        if let Err(error) = check_value(env, name, key, Some(*value)) {
            failures.push(error);
        }
    }
    for (key, value) in &doc.meta {
        if let Err(error) = check_value(env, &format!("{name}Meta"), key, Some(*value)) {
            failures.push(error);
        }
    }
    let code = format!(
        r#"
        local parent = rawget(Enum, {name:?})
        if type(parent) ~= 'table' then return -1 end
        local count = 0
        for _ in pairs(parent) do count = count + 1 end
        return count
    "#
    );
    let count: i64 = env.eval(&code).expect("count published members");
    if count != doc.values.len() as i64 {
        failures.push(format!(
            "{name}: expected {} members, observed {count}",
            doc.values.len()
        ));
    }
    failures
}

prefork_full_ui_case! {
fn patch_12_0_0_enum_publication(env: &WowLuaEnv) {
    let data: Data = serde_json::from_str(include_str!("data/patch_12_0_0_enums.json")).unwrap();
    assert_eq!(data.rows.len(), 32);
    assert_eq!(data.enums.len(), 12);
    let failures = {
        let mut failures = Vec::new();
        for (id, row) in &data.rows {
            let doc = &data.enums[&row.name];
            let mut errors = match &row.member {
                None => check_parent(&env, &row.name, doc),
                Some(key) => {
                    assert_eq!(doc.values.get(key).copied(), row.value, "{id}");
                    check_value(&env, &row.name, key, row.value)
                        .err()
                        .into_iter()
                        .collect()
                }
            };
            for key in &row.absent {
                errors.extend(check_value(&env, &row.name, key, None).err());
            }
            if errors.is_empty() {
                eprintln!(
                    "{id}: current publication proven; historical supersession={:?}",
                    row.superseded_by
                );
            } else {
                failures.push(format!("{id} ({}): {}", doc.source, errors.join("; ")));
            }
        }
        failures
    };
    assert!(failures.is_empty(), "enum gaps:\n{}", failures.join("\n"));
}
}
