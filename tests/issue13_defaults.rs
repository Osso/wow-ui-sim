//! Public Lua coverage for the issue 13 default surfaces.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn interaction_replacement_default() {
    WowLuaEnv::new()
        .unwrap()
        .exec("assert(C_PlayerInteractionManager.IsReplacingUnit() == false)")
        .unwrap();
}

#[test]
fn voice_settings_default() {
    // Native: false until the voice service is ready; the simulator has no voice
    // service, so it stays false. See docs/specs/voice-settings-access.md.
    WowLuaEnv::new()
        .unwrap()
        .exec("assert(C_VoiceChat.CanAccessSettings() == false)")
        .unwrap();
}

#[test]
fn power_display_matches_unmodified_power() {
    WowLuaEnv::new().unwrap().exec("assert(UnitPowerDisplayMod(8) == 1); assert(UnitPower('player', 8, true) == UnitPower('player', 8) * UnitPowerDisplayMod(8))").unwrap();
}

#[test]
fn flash_client_icon_has_no_return_values() {
    WowLuaEnv::new().unwrap().exec("assert(select('#', FlashClientIcon()) == 0); assert(select('#', FlashClientIcon(true)) == 0)").unwrap();
}
