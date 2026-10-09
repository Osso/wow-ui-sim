//! Current headless Era11507 getters, not native11402 or physical LED effects.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn frozen_page_cvar_defaults_against_current_era_getters() {
    assert_eq!(wow_ui_sim::client_profile::ACTIVE_INTERFACE_VERSION, 11507);
    let env = WowLuaEnv::new().expect("current headless Era environment");
    for (name, page_default, expected_current) in [
        ("GamePadFactionColor", "1", Some("1")),
        ("GamePadVibrationStrength", "1", Some("1")),
        (
            "telemetryTargetPackage",
            "Blizzard.Telemetry.Wow_Mainline",
            None,
        ),
        ("P1142_UNKNOWN_CVAR_CONTROL", "not-a-page-default", None),
    ] {
        let observed: (Option<String>, Option<String>) = env
            .eval(&format!(
                "return GetCVar({name:?}), GetCVarDefault({name:?})"
            ))
            .expect("read existing CVar state without registering or setting it");
        eprintln!(
            "P1142_CVAR name={name:?} page_default={page_default:?} current={:?} default={:?}",
            observed.0, observed.1
        );
        let expected = expected_current.map(str::to_string);
        assert_eq!(observed, (expected.clone(), expected), "{name}");
    }
}
