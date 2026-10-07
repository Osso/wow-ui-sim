//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_11_1_7_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/11.1.7-wikitext-register.json"),
        known_gaps: include_str!("data/patch_11_1_7_sweep_known_gaps.json"),
        row_count: 48,
        register_env: "P1117_SWEEP_REGISTER",
        out_env: "P1117_SWEEP_OUT",
        later_registers: &[
            include_str!("../data/patch-api/sources/11.2.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.2.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/11.2.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.0-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        ],
    });
}
}
