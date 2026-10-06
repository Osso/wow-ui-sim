//! Publication/absence only: no signature, output, security, or behavior parity claim.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

#[test]
fn patch_12_0_1_publication_sweep() {
    sweep::run_publication_sweep(&sweep::SweepSpec {
        register: include_str!("../data/patch-api/sources/12.0.1-wikitext-register.json"),
        known_gaps: include_str!("data/patch_12_0_1_sweep_known_gaps.json"),
        row_count: 225,
        register_env: "P1201_SWEEP_REGISTER",
        out_env: "P1201_SWEEP_OUT",
        later_registers: &[
            include_str!("../data/patch-api/sources/12.0.5-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.0.7-wikitext-register.json"),
            include_str!("../data/patch-api/sources/12.1.0-wikitext-register.json"),
        ],
    });
}
