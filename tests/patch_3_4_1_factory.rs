//! Current bare Wrath interface 38001 measurement, not native Classic 30401.
#![cfg(feature = "client-wrath")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{
    Entry, SweepSpec, probe_entry, run_factory_publication_sweep, run_publication_sweep,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use wow_ui_sim::lua_api::WowLuaEnv;

const INVENTORY: &str =
    include_str!("../data/patch-api/evidence/3.4.1-factory-2026-10-09/inventory.json");
const SPEC: SweepSpec = SweepSpec {
    register: INVENTORY,
    known_gaps: include_str!("../data/patch-api/evidence/3.4.1-factory-2026-10-09/known-gaps.json"),
    row_count: 333,
    register_env: "WOW_SIM_P341_FACTORY_REGISTER",
    out_env: "WOW_SIM_P341_FACTORY_OUT",
    // 3.4.3 contains no explicit identities; foreign client lines are excluded.
    later_registers: &[include_str!(
        "../data/patch-api/sources/3.4.2-source-inventory.json"
    )],
};

#[test]
fn patch_3_4_1_exact_factory_publication_gaps() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    assert_eq!(wow_ui_sim::client_profile::ACTIVE_INTERFACE_VERSION, 38001);
    run_factory_publication_sweep(&env, &SPEC);
}

#[test]
fn literal_inventory_retains_every_historical_occurrence() {
    let source: Value = serde_json::from_str(include_str!(
        "../data/patch-api/sources/3.4.1-page-coverage.json"
    ))
    .unwrap();
    let current: Value = serde_json::from_str(INVENTORY).unwrap();
    let rows = current["entries"].as_array().unwrap();
    assert_eq!(rows.len(), 333);
    assert_eq!(current["source"], source["source"]);
    assert_eq!(current["header_counts"], source["header_counts"]);
    for (row, historical) in rows
        .iter()
        .zip(source["inventory_rows"].as_array().unwrap())
    {
        for (field, value) in row.as_object().unwrap() {
            assert_eq!(value, &historical[field], "{}: {field}", row["id"]);
        }
    }
}

#[test]
fn cvar_default_differences_are_not_publication_failures() {
    let env = WowLuaEnv::new().unwrap();
    let inventory: Value = serde_json::from_str(INVENTORY).unwrap();
    let mut differences = Vec::new();
    let mut counts = (0, 0, 0);
    let mut entries: Vec<Entry> = serde_json::from_value(inventory["entries"].clone()).unwrap();
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    for entry in entries {
        if entry.section != "cvars"
            || entry.kind.is_some()
            || entry.direction == "removed"
            || entry.symbol == "DynamicVRSSensitivityThreshold"
        {
            continue;
        }
        let (_, _, published, value, default) = probe_entry(&env, &entry, false, &BTreeMap::new());
        if !published {
            counts.0 += 1;
        } else if entry.page_default == default {
            counts.1 += 1;
        } else {
            counts.2 += 1;
            differences.push(json!({"symbol": entry.symbol,
                "page_default": entry.page_default, "value": value, "default": default}));
        }
    }
    assert_eq!(counts, (4, 42, 15));
    let expected: Value = serde_json::from_str(include_str!(
        "../data/patch-api/evidence/3.4.1-factory-2026-10-09/published-default-differences.json"
    ))
    .unwrap();
    assert_eq!(json!(differences), expected);
}

#[test]
fn wrath_event_acceptance_cannot_discriminate_native_names() {
    let env = WowLuaEnv::new().unwrap();
    let accepted: (bool, bool) = env
        .eval(
            r#"
        local f = CreateFrame('Frame')
        f:RegisterEvent('P341_NOT_A_NATIVE_EVENT')
        f:RegisterEvent('CURRENCY_DISPLAY_UPDATE')
        return f:IsEventRegistered('P341_NOT_A_NATIVE_EVENT'),
            f:IsEventRegistered('CURRENCY_DISPLAY_UPDATE')
    "#,
        )
        .unwrap();
    eprintln!("event discrimination control: {accepted:?}");
    assert_eq!(accepted, (true, true));
}

#[test]
fn console_cvar_record_does_not_publish_logfps_command() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("C_CVar.RegisterCVar('LogFps', '0')").unwrap();
    let catalog: (bool, i32) = env
        .eval(
            r#"
        local cvar, commands = false, 0
        for _, record in ipairs(C_Console.GetAllCommands()) do
            if record.commandType == Enum.ConsoleCommandType.Command then
                commands = commands + 1
            elseif record.command == 'LogFps' then
                cvar = true
            end
        end
        return cvar, commands
    "#,
        )
        .unwrap();
    assert_eq!(catalog, (true, 0));
    let inventory: Value = serde_json::from_str(INVENTORY).unwrap();
    let entry: Entry = serde_json::from_value(
        inventory["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["symbol"] == "LogFps")
            .unwrap()
            .clone(),
    )
    .unwrap();
    let result = probe_entry(&env, &entry, false, &BTreeMap::new());
    eprintln!("console catalog control: {catalog:?}; LogFps probe: {result:?}");
    assert_eq!(result.0, "command");
    assert!(!result.2);
}

#[test]
#[should_panic(expected = "register client line does not match active profile")]
fn factory_rejects_foreign_client_before_reading_cache() {
    let env = WowLuaEnv::new().unwrap();
    let spec = SweepSpec {
        register: r#"{"client_line":"retail","entries":[]}"#,
        known_gaps: "[]",
        row_count: 0,
        register_env: "WOW_SIM_P341_FOREIGN_REGISTER",
        out_env: "WOW_SIM_P341_FOREIGN_OUT",
        later_registers: &[],
    };
    run_publication_sweep(&env, &spec);
}
