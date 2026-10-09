//! Current bare Era CVar getters; not native11306 defaults or modeled effects.
#[path = "../tests/common/publication_probe.rs"]
mod publication_probe;

use publication_probe::{Entry, probe_entry};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const REGISTER: &str =
    include_str!("../data/patch-api/evidence/1.13.6-session-2026-10-09/default-register.json");
const KNOWN_GAPS: &str =
    include_str!("../data/patch-api/evidence/1.13.6-session-2026-10-09/current-known-gaps.json");

#[test]
fn exact_current_era_publication_gaps() {
    assert_eq!(ACTIVE, ClientProfile::Era);
    assert_eq!(ACTIVE_INTERFACE_VERSION, 11507);
    let register: Value = serde_json::from_str(REGISTER).expect("own literal inventory");
    let entries: Vec<Entry> =
        serde_json::from_value(register["entries"].clone()).expect("three CVar occurrences");
    assert_eq!(entries.len(), 3);
    let env = WowLuaEnv::new().expect("current bare Era factory");
    let mut rows = Vec::new();
    let mut gaps = Vec::new();
    for entry in entries {
        let (kind, detail, published, value, default) =
            probe_entry(&env, &entry, false, &BTreeMap::new());
        assert_eq!(kind, "cvar", "{}", entry.symbol);
        assert_eq!(entry.direction, "added");
        assert_eq!(entry.page_default, None, "source specifies no defaults");
        if !published {
            gaps.push(entry.id.clone());
        }
        rows.push(
            json!({"id":entry.id,"symbol":entry.symbol,"section":entry.section,
            "direction":entry.direction,"page_default":entry.page_default,
            "kind":kind,"detail":detail,"published":published,"value":value,
            "default":default,"model_credit":false,"native_credit":false}),
        );
    }
    let raw_getters: (String, String) = env
        .eval("return type(rawget(C_CVar,'GetCVar')), type(rawget(C_CVar,'GetCVarDefault'))")
        .expect("raw namespace registration, not generic lookup");
    assert_eq!(raw_getters, ("function".into(), "function".into()));
    let identity: (String, String, String) = env
        .eval("local v,b,_,i=GetBuildInfo(); return tostring(v),tostring(b),tostring(i)")
        .expect("observe compatibility identity separately from active profile");
    let observed = json!({"scope":"current bare Era getters only; no SOURCE ledger mutation",
        "configured_interface":ACTIVE_INTERFACE_VERSION,"source_interface":11306,
        "raw_getter_types":raw_getters,"observed_compatibility_identity":identity,
        "rows":rows,"later_registers":[]});
    eprintln!("P1136_FACTORY {observed}");
    let output = std::env::var_os("WOW_SIM_P1136_FACTORY_OUT")
        .expect("absolute owned observation output required");
    assert!(std::path::Path::new(&output).is_absolute());
    std::fs::write(output, serde_json::to_vec_pretty(&observed).unwrap())
        .expect("write separate current observations");
    let reviewed: Value = serde_json::from_str(include_str!(
        "../data/patch-api/evidence/1.13.6-session-2026-10-09/current-reviewed-observations.json"
    ))
    .expect("reviewed concrete current getter observations");
    assert_eq!(
        observed, reviewed,
        "exact current getter values and identity"
    );
    gaps.sort();
    let expected: Vec<String> = serde_json::from_str(KNOWN_GAPS).expect("reviewed current gaps");
    assert_eq!(
        gaps, expected,
        "exact current publication mismatches, not native gaps"
    );
}

#[test]
fn unknown_cvar_is_not_fabricated_by_getters() {
    let env = WowLuaEnv::new().expect("current bare Era factory");
    let entry: Entry = serde_json::from_value(json!({"id":"negative-p1136",
        "section":"cvars","direction":"added","symbol":"P1136_UNKNOWN_CVAR_CONTROL"}))
    .expect("unknown negative control");
    let actual = probe_entry(&env, &entry, false, &BTreeMap::new());
    assert_eq!(
        actual,
        (
            "cvar".into(),
            "value/default queried".into(),
            false,
            None,
            None
        )
    );
}
