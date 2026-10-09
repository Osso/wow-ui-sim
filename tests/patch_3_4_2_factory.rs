//! Bare simulator Wrath (interface 38001), not native Wrath Classic 3.4.2 (30402).
#![cfg(feature = "client-wrath")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{
    Entry, SweepSpec, probe_entry, run_factory_publication_sweep, run_publication_sweep,
};
use std::collections::BTreeMap;
use wow_ui_sim::lua_api::WowLuaEnv;

const INVENTORY: &str = include_str!("../data/patch-api/sources/3.4.2-source-inventory.json");
const SPEC: SweepSpec = SweepSpec {
    register: INVENTORY,
    known_gaps: include_str!("../data/patch-api/3.4.2-factory-known-gaps.json"),
    row_count: 155,
    register_env: "WOW_SIM_P342_FACTORY_REGISTER",
    out_env: "WOW_SIM_P342_FACTORY_OUT",
    later_registers: &[],
};

#[test]
fn bare_wrath_factories_probe_literal_widget_owners() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    let inventory: serde_json::Value = serde_json::from_str(INVENTORY).unwrap();
    let mut count = 0;
    for value in inventory["entries"].as_array().unwrap() {
        let entry: Entry = serde_json::from_value(value.clone()).unwrap();
        if entry.section == "widgets" {
            let result = probe_entry(&env, &entry, entry.direction == "removed", &BTreeMap::new());
            eprintln!("{}: {result:?}", entry.id);
            assert_eq!(result.0, "object-method", "{}", entry.id);
            count += 1;
        }
    }
    assert_eq!(count, 18);
}

#[test]
fn patch_3_4_2_exact_factory_publication_gaps() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    assert_eq!(wow_ui_sim::client_profile::ACTIVE_INTERFACE_VERSION, 38001);
    run_factory_publication_sweep(&env, &SPEC);
}

#[test]
fn cvar_defaults_remain_separate_from_publication() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    let inventory: serde_json::Value = serde_json::from_str(INVENTORY).unwrap();
    let mut published_default_differences = Vec::new();
    let mut counts = (0, 0, 0);
    for value in inventory["entries"].as_array().unwrap() {
        let entry: Entry = serde_json::from_value(value.clone()).unwrap();
        if entry.section != "cvars" || entry.direction == "removed" {
            continue;
        }
        let (_, _, published, _, default) = probe_entry(&env, &entry, false, &BTreeMap::new());
        if !published {
            counts.0 += 1;
        } else if entry.page_default == default {
            counts.1 += 1;
        } else {
            counts.2 += 1;
            published_default_differences.push((entry.symbol, entry.page_default, default));
        }
    }
    assert_eq!(counts, (14, 11, 1));
    assert_eq!(
        published_default_differences,
        vec![("TargetAutoLock".into(), Some("1".into()), Some("0".into()))]
    );
}

#[test]
fn wrath_event_acceptance_is_nondiscriminating() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    let registered: (bool, bool) = env
        .eval(
            r#"
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('P342_NOT_A_NATIVE_EVENT')
        frame:RegisterEvent('TWITTER_POST_RESULT')
        return frame:IsEventRegistered('P342_NOT_A_NATIVE_EVENT'),
            frame:IsEventRegistered('TWITTER_POST_RESULT')
    "#,
        )
        .unwrap();
    assert_eq!(registered, (true, true));
}

#[test]
#[should_panic(expected = "register client line does not match active profile")]
fn wrath_factory_rejects_retail_inventory() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    let spec = SweepSpec {
        register: r#"{"client_line":"retail","entries":[]}"#,
        known_gaps: "[]",
        row_count: 0,
        register_env: "WOW_SIM_P342_RETAIL_CONTROL_REGISTER",
        out_env: "WOW_SIM_P342_RETAIL_CONTROL_OUT",
        later_registers: &[],
    };
    run_publication_sweep(&env, &spec);
}

#[test]
fn foreign_successors_do_not_remove_wrath_publication() {
    let env = WowLuaEnv::new().expect("bare Wrath Lua environment");
    env.exec("FactoryLineControl = function() return 342 end")
        .unwrap();
    let spec = SweepSpec {
        register: r#"{"client_line":"wrath-classic","entries":[{"id":"own","section":"global-api","direction":"added","symbol":"FactoryLineControl"}]}"#,
        known_gaps: "[]",
        row_count: 1,
        register_env: "WOW_SIM_P342_SUCCESSOR_CONTROL_REGISTER",
        out_env: "WOW_SIM_P342_SUCCESSOR_CONTROL_OUT",
        later_registers: &[
            r#"{"client_line":"retail","entries":[{"id":"foreign-retail","section":"global-api","direction":"removed","symbol":"FactoryLineControl"}]}"#,
            r#"{"client_line":"mists-classic","entries":[{"id":"foreign-classic","section":"global-api","direction":"removed","symbol":"FactoryLineControl"}]}"#,
            r#"{"client_line":"wrath-classic","entries":[]}"#,
        ],
    };
    run_factory_publication_sweep(&env, &spec);
    assert_eq!(env.eval::<i32>("return FactoryLineControl()").unwrap(), 342);
}
