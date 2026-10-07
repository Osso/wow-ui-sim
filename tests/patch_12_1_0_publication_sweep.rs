//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_12_1_0_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        known_gaps: include_str!("data/patch_12_1_0_sweep_known_gaps.json"),
        row_count: 778,
        register_env: "P1210_SWEEP_REGISTER",
        out_env: "P1210_SWEEP_OUT",
        later_registers: &[],
    });
}
}
