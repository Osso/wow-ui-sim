//! Mists Classic publication boundary, not retail MoP or native behavior parity.
#![cfg(feature = "client-mists")]

#[path = "common/prefork_full_ui_preload.rs"]
mod preload;
#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::client_profile::{ACTIVE, ACTIVE_INTERFACE_VERSION, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

const MISTS_ADDITION: &str = r#"{"client_line":"mists-classic","entries":[{"id":"own","section":"global-api","direction":"added","symbol":"GetTime"}]}"#;
const MISTS_REMOVAL: &str = r#"{"client_line":"mists-classic","entries":[{"id":"later","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;
const RETAIL_REMOVAL: &str = r#"{"entries":[{"id":"retail","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;
const ERA_REMOVAL: &str = r#"{"client_line":"classic-era","entries":[{"id":"era","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;

fn run_mists_control(later: &'static [&'static str], gaps: &'static str) {
    let env = WowLuaEnv::new().expect("create Mists publication control environment");
    sweep::run_publication_sweep(&env, &sweep::SweepSpec {
        register: MISTS_ADDITION,
        known_gaps: gaps,
        row_count: 1,
        register_env: "MISTS_LINE_CONTROL_REGISTER",
        out_env: "MISTS_LINE_CONTROL_OUT",
        later_registers: later,
    });
}

#[test]
fn patch_5_5_4_client_line_excludes_retail_and_era() {
    run_mists_control(&[RETAIL_REMOVAL, ERA_REMOVAL], "[]");
    run_mists_control(&[MISTS_REMOVAL, RETAIL_REMOVAL], r#"["own"]"#);
}

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
