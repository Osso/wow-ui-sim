//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn patch_10_0_5_publication_sweep(env: &WowLuaEnv) {
    sweep::run_publication_sweep(env, &sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/10.0.5-wikitext-register.json"),
        known_gaps: include_str!("data/patch_10_0_5_sweep_known_gaps.json"),
        row_count: 93,
        register_env: "P1005_SWEEP_REGISTER",
        out_env: "P1005_SWEEP_OUT",
        later_registers: &[
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
    });
}
}
