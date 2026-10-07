//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_12_0_5_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
        known_gaps: include_str!("data/patch_12_0_5_sweep_known_gaps.json"),
        row_count: 363,
        register_env: "P1205_SWEEP_REGISTER",
        out_env: "P1205_SWEEP_OUT",
        // Retail builds the 12.1.0 surface; 12.0.7 then 12.1.0 add/remove rows supersede.
        later_registers: &[
            include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        ],
    });
}
}
