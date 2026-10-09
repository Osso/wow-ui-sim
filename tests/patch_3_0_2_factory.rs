//! Bare current-retail publication measurement; no historical/native/model parity.
#![cfg(feature = "client-retail")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{SweepSpec, run_factory_publication_sweep, run_publication_sweep};
use wow_ui_sim::lua_api::WowLuaEnv;

const REGISTER: &str = include_str!("../data/patch-api/sources/3.0.2-wikitext-register.json");
const SPEC: SweepSpec = SweepSpec {
    register: REGISTER,
    known_gaps: include_str!("../data/patch-api/evidence/3.0.2-factory-2026-10-09/known-gaps.json"),
    row_count: 373,
    register_env: "WOW_SIM_P302_FACTORY_REGISTER",
    out_env: "WOW_SIM_P302_FACTORY_OUT",
    later_registers: &[
        include_str!("../data/patch-api/sources/3.0.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.0.8-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/3.3.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/4.3.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.0.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/5.4.8-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/6.2.4-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.0.3-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/7.3.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.3.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/8.3.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/9.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.1.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.6-wikitext-register.json"),
        include_str!("../data/patch-api/sources/10.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.2-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.1.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/11.2.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.0-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
        include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
    ],
};

#[test]
fn patch_3_0_2_exact_factory_publication_gaps() {
    let env = WowLuaEnv::new().expect("bare current-retail Lua environment");
    run_factory_publication_sweep(&env, &SPEC);
}

#[test]
#[should_panic(expected = "register client line does not match active profile")]
fn factory_rejects_classic_before_reading_cache() {
    let env = WowLuaEnv::new().unwrap();
    let foreign = SweepSpec {
        register: r#"{"client_line":"wrath-classic","entries":[]}"#,
        known_gaps: "[]",
        row_count: 0,
        register_env: "WOW_SIM_P302_FOREIGN_REGISTER",
        out_env: "WOW_SIM_P302_FOREIGN_OUT",
        later_registers: &[],
    };
    run_publication_sweep(&env, &foreign);
}
