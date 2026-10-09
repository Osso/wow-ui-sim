//! Current Era bare-factory observations only, not historical/native compatibility.
//! No Blizzard UI, CASC, event producers or SOURCE-ledger mutation.

#[path = "../tests/common/publication_probe.rs"]
mod publication_probe;

use publication_probe::{Entry, probe_entry};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const INVENTORY: &str = include_str!(
    "../data/patch-api/evidence/1.14.3-session-2026-10-09/original/default-register.json"
);
const KNOWN_GAPS: &str =
    include_str!("../data/patch-api/evidence/1.14.3-session-2026-10-09/current-known-gaps.json");
const INVENTORY_ROWS: usize = 412;
const OUTPUT_ENV: &str = "WOW_SIM_P1143_FACTORY_OUT";

#[derive(Deserialize)]
struct Inventory {
    entries: Vec<Entry>,
}

fn observe_entry(env: &WowLuaEnv, entry: &Entry) -> Value {
    let removed = entry.direction == "removed";
    let (kind, detail, matches, value, default) =
        probe_entry(env, entry, removed, &BTreeMap::new());
    json!({
        "id": entry.id,
        "section": entry.section,
        "direction": entry.direction,
        "symbol": entry.symbol,
        "page_default": entry.page_default,
        "kind": kind,
        "detail": detail,
        "publication_direction_match": matches,
        "value": value,
        "default": default,
        "model_credit": false,
        "native_credit": false,
    })
}

fn query_factory_observations() -> Value {
    assert_eq!(ACTIVE, ClientProfile::Era);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 11507);
    let inventory: Inventory =
        serde_json::from_str(INVENTORY).expect("owned exact source inventory");
    assert_eq!(inventory.entries.len(), INVENTORY_ROWS);
    let env = WowLuaEnv::new().expect("standalone current Era factory");
    let rows: Vec<Value> = inventory
        .entries
        .iter()
        .map(|entry| observe_entry(&env, entry))
        .collect();
    let identity: (String, String, String) = env
        .eval("local v, b, _, i = GetBuildInfo(); return tostring(v), tostring(b), tostring(i)")
        .expect("observe compatibility-default identity, not infer source identity");
    let nonsense_event: bool = env
        .eval(
            "local f = CreateFrame('Frame'); f:RegisterEvent('P1143_NOT_A_NATIVE_EVENT'); \
             return f:IsEventRegistered('P1143_NOT_A_NATIVE_EVENT') == true",
        )
        .expect("record Classic event name nondiscrimination control");
    json!({
        "scope": "current bare Era factory; publication is not model/native proof",
        "configured_interface": ACTIVE_INTERFACE_VERSION,
        "source_interface": 11403,
        "source_build": 43639,
        "observed_compatibility_identity": identity,
        "later_registers": [],
        "nonsense_event_accepted": nonsense_event,
        "rows": rows,
    })
}

#[test]
fn record_exact_inventory_factory_observations() {
    let output = std::env::var_os(OUTPUT_ENV).expect("absolute owned output path required");
    assert!(std::path::Path::new(&output).is_absolute());
    let observations = query_factory_observations();
    let bytes = serde_json::to_vec_pretty(&observations).expect("serialize observations");
    std::fs::write(&output, bytes).expect("write owned current factory observations");
    let rows = observations["rows"]
        .as_array()
        .expect("observed source occurrences");
    assert_eq!(rows.len(), INVENTORY_ROWS);
    assert!(rows.iter().all(|row| row["kind"] != "probe-error"));
    let mut mismatches: Vec<String> = rows
        .iter()
        .filter(|row| row["publication_direction_match"] == false)
        .map(|row| row["id"].as_str().expect("source ID").to_owned())
        .collect();
    mismatches.sort();
    let reviewed: Vec<String> = serde_json::from_str(KNOWN_GAPS).expect("reviewed current gaps");
    assert_eq!(
        mismatches, reviewed,
        "current strict mismatches, not native gaps"
    );
}

#[test]
fn generic_lookup_does_not_count_as_raw_member_registration() {
    let env = WowLuaEnv::new().expect("standalone current Era factory");
    let control: Entry = serde_json::from_value(json!({
        "id": "negative-control",
        "section": "global-api",
        "direction": "added",
        "symbol": "C_P1143UnmodeledNamespace.NotARealMember",
    }))
    .expect("owned nonsense control");
    let (kind, detail, matches, _, _) = probe_entry(&env, &control, false, &BTreeMap::new());
    assert_eq!(kind, "member");
    assert!(
        !matches,
        "generic lookup is not a raw member registration: {detail}"
    );
}
