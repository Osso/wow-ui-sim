//! Mists Classic publication boundary, not retail MoP or native behavior parity.
#![cfg(feature = "client-mists")]

#[path = "common/prefork_full_ui_preload.rs"]
mod preload;
#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};

#[test]
fn patch_5_5_4_publication_sweep() {
    crate::common::with_timeout(90, || {
        assert_eq!(ACTIVE, ClientProfile::Mists);
        assert_eq!(ACTIVE_INTERFACE_VERSION, 50504);
        let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path()
            .expect("resolve Mists cached UI");
        assert!(cache.ends_with("mists/AddOns"), "wrong UI cache: {}", cache.display());
        let env = preload::preload_full_game_ui().expect("load cached Mists Game UI");
        assert!(env.eval::<bool>("return C_AddOns.IsAddOnLoaded('Blizzard_FrameXML') == true")
            .expect("query loaded Mists FrameXML"));
        let register = include_str!("../data/patch-api/sources/5.5.4-wikitext-register.json");
        let rows: serde_json::Value = serde_json::from_str(register).expect("parse Mists register");
        sweep::run_publication_sweep(&env, &sweep::SweepSpec {
            register,
            known_gaps: include_str!("data/patch_5_5_4_sweep_known_gaps.json"),
            row_count: rows["entries"].as_array().expect("inventory rows").len(),
            register_env: "P554_SWEEP_REGISTER",
            out_env: "P554_SWEEP_OUT",
            later_registers: &[],
        });
    });
}
