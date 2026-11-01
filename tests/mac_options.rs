//! `C_MacOptions` best-guess behaviour on a non-macOS host.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn mac_options_report_no_os_permissions_and_persist_shortcut_request() {
    let env = WowLuaEnv::new().expect("create mac options environment");
    let result: String = env
        .eval(
            r#"
            local before_cvar, before_applied = C_MacOptions.AreOSShortcutsDisabled()
            C_MacOptions.SetOSShortcutsDisabled(true)
            local after_cvar, after_applied = C_MacOptions.AreOSShortcutsDisabled()
            return table.concat({
                C_MacOptions.GetGameBundleName(),
                tostring(before_cvar), tostring(before_applied),
                tostring(after_cvar), tostring(after_applied),
                GetCVar("MacDisableOsShortcuts"),
                tostring(C_MacOptions.IsMicrophoneEnabled()),
                tostring(C_MacOptions.IsInputMonitoringEnabled()),
                tostring(C_MacOptions.IsUniversalAccessEnabled()),
                tostring(C_MacOptions.HasNewStyleInputMonitoring()),
            }, "|")
            "#,
        )
        .expect("query C_MacOptions");
    assert_eq!(
        result,
        "World of Warcraft|false|false|true|false|1|false|false|false|false"
    );
}
