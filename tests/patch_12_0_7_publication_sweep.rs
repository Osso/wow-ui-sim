//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_12_0_7_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
        known_gaps: include_str!("data/patch_12_0_7_sweep_known_gaps.json"),
        row_count: 174,
        register_env: "P1207_SWEEP_REGISTER",
        out_env: "P1207_SWEEP_OUT",
        // Retail builds the 12.1.0 surface; its add/remove rows supersede 12.0.7 ones.
        later_registers: &[include_str!(
            "../data/patch-api/sources/12.1.0-wikitext-register.json"
        )],
    });
}
}
