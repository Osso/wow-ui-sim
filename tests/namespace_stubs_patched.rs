//! Pins namespace entries that must not fall through to generic nil stubs.
//!
//! `C_UIWidgetManager.GetPowerBarWidgetSetID` is a modeled HUD selector,
//! while the `C_PlayerInfo` probes are state-backed through
//! `missing_surface::player_info`.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn c_ui_widget_manager_get_power_bar_widget_set_id_returns_native_id() {
    let env = env();
    let value: f64 = env
        .eval("return C_UIWidgetManager.GetPowerBarWidgetSetID()")
        .expect("eval should succeed");
    assert_eq!(value, 283.0);
}

/// Regression guard: the PlayerInfo methods are registered by
/// `missing_surface::player_info`. If they ever fall back to generic nil
/// stubs, these calls fail.
#[test]
fn c_player_info_stubs_are_overridden_by_missing_surface_state() {
    let env = env();
    let (rpe_type, alt_type): (String, String) = env
        .eval(
            r#"
            return type(C_PlayerInfo.IsPlayerInRPE),
                   type(C_PlayerInfo.GetAlternateFormInfo)
            "#,
        )
        .expect("eval should succeed");
    assert_eq!(rpe_type, "function");
    assert_eq!(alt_type, "function");

    // Both methods must complete without error — proves some
    // implementation is wired, not the stub_nil fallback.
    let (rpe_ok, alt_ok): (bool, bool) = env
        .eval(
            r#"
            local rpe_ok = pcall(function() return C_PlayerInfo.IsPlayerInRPE() end)
            local alt_ok = pcall(function() return C_PlayerInfo.GetAlternateFormInfo() end)
            return rpe_ok, alt_ok
            "#,
        )
        .expect("eval should succeed");
    assert!(rpe_ok);
    assert!(alt_ok);
}
