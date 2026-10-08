//! Behavioral controls for distinct client histories, independent of patch numbering.
#![cfg(feature = "client-retail")]

#[path = "common/publication_sweep.rs"]
mod sweep;

use wow_ui_sim::lua_api::WowLuaEnv;

const OWN: &str =
    r#"{"entries":[{"id":"own","section":"global-api","direction":"added","symbol":"GetTime"}]}"#;
const MISTS_REMOVAL: &str = r#"{"client_line":"mists-classic","entries":[{"id":"mists","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;
const ERA_REMOVAL: &str = r#"{"client_line":"classic-era","entries":[{"id":"era","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;
const RETAIL_REMOVAL: &str = r#"{"entries":[{"id":"retail","section":"global-api","direction":"removed","symbol":"GetTime"}]}"#;
const RETAIL_ADDITION: &str = r#"{"entries":[{"id":"readded","section":"global-api","direction":"added","symbol":"GetTime"}]}"#;

fn run_control(register: &'static str, later: &'static [&'static str], gaps: &'static str) {
    let env = WowLuaEnv::new().expect("create publication control environment");
    sweep::run_publication_sweep(
        &env,
        &sweep::SweepSpec {
            register,
            known_gaps: gaps,
            row_count: 1,
            register_env: "CLIENT_LINE_CONTROL_REGISTER",
            out_env: "CLIENT_LINE_CONTROL_OUT",
            later_registers: later,
        },
    );
}

#[test]
fn publication_sweep_client_lines_ignore_classic_removals_for_retail() {
    run_control(OWN, &[MISTS_REMOVAL, ERA_REMOVAL], "[]");
}

#[test]
fn publication_sweep_client_lines_keep_same_line_ordering() {
    run_control(OWN, &[RETAIL_REMOVAL, MISTS_REMOVAL], r#"["own"]"#);
    run_control(OWN, &[RETAIL_REMOVAL, RETAIL_ADDITION, ERA_REMOVAL], "[]");
}

#[test]
#[should_panic(expected = "register client line does not match active profile")]
fn publication_sweep_client_lines_reject_wrong_profile() {
    run_control(MISTS_REMOVAL, &[], r#"["mists"]"#);
}
