//! Current bare Wrath interface 38001 measurement, not native Classic 30401.
#![cfg(feature = "client-wrath")]
#[path = "common/publication_sweep.rs"]
mod publication_sweep;

use publication_sweep::{SweepSpec, run_factory_publication_sweep};
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
