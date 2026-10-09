//! Supported retail bare factory only; no historical native or synchronization proof.
#![cfg(feature = "client-retail")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{Entry, SweepSpec, probe_entry, run_factory_publication_sweep};
use std::collections::BTreeMap;
use wow_ui_sim::lua_api::WowLuaEnv;

const INVENTORY: &str =
    include_str!("../data/patch-api/evidence/3.0.3-factory-2026-10-09/inventory.json");
const SPEC: SweepSpec = SweepSpec {
    register: INVENTORY,
    known_gaps: include_str!("../data/patch-api/evidence/3.0.3-factory-2026-10-09/known-gaps.json"),
    row_count: 3,
    register_env: "WOW_SIM_P303_FACTORY_REGISTER",
    out_env: "WOW_SIM_P303_FACTORY_OUT",
    later_registers: &[],
};

#[test]
fn patch_3_0_3_exact_factory_publication_gaps() {
    let env = WowLuaEnv::new().expect("bare retail Lua environment");
    run_factory_publication_sweep(&env, &SPEC);
}

#[test]
fn fabricated_unknown_cvar_is_not_published() {
    let env = WowLuaEnv::new().expect("bare retail Lua environment");
    let entry = Entry {
        id: "negative-p303-fabricated-cvar".into(),
        section: "cvars".into(),
        direction: "added".into(),
        symbol: "P303_FABRICATED_UNKNOWN_CVAR".into(),
        page_default: None,
        kind: None,
    };
    let observed = probe_entry(&env, &entry, false, &BTreeMap::new());
    eprintln!("fabricated unknown CVar: {observed:?}");
    assert_eq!(
        observed,
        (
            "cvar".into(),
            "value/default queried".into(),
            false,
            None,
            None
        )
    );
}
