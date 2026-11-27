#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};

fn frame_xml_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXML"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXML"));
    env
}

#[test]
fn client_identity_level_caps_and_ready_check_text_follow_loaded_data() {
    let env = frame_xml_env();
    let identity: (String, String, i32, i32, i32, i32, i32, i32, i32, String, bool, bool) = env
        .eval(
            r#"
            local version, build, _, interface = GetBuildInfo()
            return version, build, interface, GetExpansionLevel(),
                GetClientDisplayExpansionLevel(), GetAccountExpansionLevel(),
                GetMaxLevelForExpansionLevel(0), GetMaxLevelForExpansionLevel(3),
                GetMaxLevelForPlayerExpansion(), GetLocale(), IsMacClient(), IsWindowsClient()
            "#,
        )
        .unwrap();
    let expected = if cfg!(feature = "client-retail") {
        ("12.1.5", "70783", 120105, 10)
    } else {
        ("1.60.1", "70707", 16001, 0)
    };
    assert_eq!((identity.0.as_str(), identity.1.as_str(), identity.2), (expected.0, expected.1, expected.2));
    assert_eq!((identity.3, identity.4, identity.5), (expected.3, 10, 10));
    assert_eq!((identity.6, identity.7, identity.8), (60, 85, 80));
    assert_eq!(identity.9, "enUS");
    assert_eq!(identity.10, cfg!(target_os = "macos"));
    assert_eq!(identity.11, cfg!(target_os = "windows"));

    let texts: (String, String, String, String) = env
        .eval("return ReadyCheckFrameYesButton:GetText(), ReadyCheckFrameNoButton:GetText(), READY, NOT_READY")
        .unwrap();
    assert_eq!((texts.0.as_str(), texts.1.as_str()), (texts.2.as_str(), texts.3.as_str()));
    assert_ne!(texts.0, texts.1, "two original consumers must resolve distinct tokens");
}
