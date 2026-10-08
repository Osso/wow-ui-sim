//! Numeric enum publication only: no consumer or domain behavior implied.
#![cfg(feature = "client-retail")]

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use wow_ui_sim::lua_api::WowLuaEnv;

// Frozen from the cached retail Blizzard_APIDocumentationGenerated declarations.
const DATA: &str = include_str!("data/patch_12_1_0_enums.json");
const KNOWN_GAPS: &str = include_str!("data/patch_12_1_0_enum_known_gaps.json");
const ROW_COUNT: usize = 58;

#[derive(Deserialize)]
struct Data {
    enums: BTreeMap<String, DocEnum>,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct DocEnum {
    meta: DocMeta,
    values: BTreeMap<String, i64>,
    // Members cached Blizzard Lua adds at load; the native metadata never counts them.
    #[serde(default)]
    blizzard_lua: Option<BlizzardLua>,
}

#[derive(Deserialize)]
struct BlizzardLua {
    values: BTreeMap<String, i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DocMeta {
    num_values: i64,
    min_value: i64,
    max_value: i64,
}

#[derive(Deserialize)]
struct Row {
    source_id: String,
    #[serde(rename = "enum")]
    enum_name: String,
    member: Option<String>,
    value: Option<i64>,
}

const OBSERVE_TABLE: &str = r#"
local name = ...
local values = rawget(Enum, name)
if type(values) ~= 'table' then return 'missing', 'missing' end
local parts = {}
for key, value in pairs(values) do
    local shown = type(value) == 'number' and string.format('%.17g', value) or type(value)
    parts[#parts + 1] = tostring(key) .. '=' .. shown
end
table.sort(parts)
local meta = rawget(Enum, name .. 'Meta')
if type(meta) ~= 'table' then return table.concat(parts, ';'), 'missing' end
return table.concat(parts, ';'),
    string.format('%s/%s/%s', tostring(meta.NumValues), tostring(meta.MinValue), tostring(meta.MaxValue))
"#;

const OBSERVE_MEMBER: &str = r#"
local name, member = ...
local values = rawget(Enum, name)
if type(values) ~= 'table' then return 'parent missing' end
local raw, lookup = rawget(values, member), values[member]
return 'raw=' .. tostring(raw) .. '; lookup=' .. tostring(lookup)
"#;

// Lua decimal byte escapes keep arbitrary names out of the script syntax.
fn quote_lua(value: &str) -> String {
    let escaped: String = value.bytes().map(|byte| format!("\\{byte:03}")).collect();
    format!("\"{escaped}\"")
}

fn expected_table(doc: &DocEnum) -> (String, String) {
    let additions = doc.blizzard_lua.iter().flat_map(|lua| &lua.values);
    let mut parts: Vec<_> = doc
        .values
        .iter()
        .chain(additions)
        .map(|(name, value)| format!("{name}={value}"))
        .collect();
    parts.sort();
    let meta = &doc.meta;
    let meta = format!("{}/{}/{}", meta.num_values, meta.min_value, meta.max_value);
    (parts.join(";"), meta)
}

fn expected_member(value: Option<i64>) -> String {
    let shown = value.map_or("nil".to_owned(), |value| value.to_string());
    format!("raw={shown}; lookup={shown}")
}

/// Returns (ok, detail) for one ledger row.
fn check_row(env: &WowLuaEnv, data: &Data, row: &Row) -> (bool, String) {
    let doc = &data.enums[&row.enum_name];
    let name = quote_lua(&row.enum_name);
    match &row.member {
        None => {
            let code = format!("return (function(...) {OBSERVE_TABLE} end)({name})");
            let observed: (String, String) = env.eval(&code).expect("observe enum table");
            let expected = expected_table(doc);
            (
                observed == expected,
                format!("expected {expected:?}; observed {observed:?}"),
            )
        }
        Some(member) => {
            let code = format!(
                "return (function(...) {OBSERVE_MEMBER} end)({name}, {})",
                quote_lua(member)
            );
            let observed: String = env.eval(&code).expect("observe enum member");
            let expected = expected_member(row.value);
            (
                observed == expected,
                format!("{member}: expected {expected}; observed {observed}"),
            )
        }
    }
}

fn read_data() -> Data {
    let data: Data = serde_json::from_str(DATA).expect("parse 12.1.0 enum data");
    assert_eq!(data.rows.len(), ROW_COUNT, "enum row count changed");
    let ids: BTreeSet<_> = data.rows.iter().map(|row| &row.source_id).collect();
    assert_eq!(ids.len(), ROW_COUNT, "duplicate source IDs");
    for row in &data.rows {
        let doc = &data.enums[&row.enum_name];
        // Added members carry their cached value; removed members are absent from the cache.
        if let Some(member) = &row.member {
            assert_eq!(
                doc.values.get(member).copied(),
                row.value,
                "{}",
                row.source_id
            );
        }
    }
    data
}

prefork_full_ui_case! {
fn patch_12_1_0_enum_publication(env: &WowLuaEnv) {
    let data = read_data();
    let known: BTreeSet<String> = serde_json::from_str(KNOWN_GAPS).expect("parse known gaps");
    let results = {
        data.rows
            .iter()
            .map(|row| (row.source_id.clone(), check_row(&env, &data, row)))
            .collect::<BTreeMap<_, _>>()
    };
    let mut non_ok = BTreeSet::new();
    for (id, (ok, detail)) in &results {
        if !ok {
            eprintln!("enum gap {id}: {detail}");
            non_ok.insert(id.clone());
        }
    }
    let new_gaps: Vec<_> = non_ok.difference(&known).collect();
    let resolved_gaps: Vec<_> = known.difference(&non_ok).collect();
    assert_eq!(
        non_ok, known,
        "new gaps: {new_gaps:?}; resolved/stale gaps: {resolved_gaps:?}"
    );
}
}
