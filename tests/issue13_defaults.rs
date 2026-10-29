//! Public Lua coverage for the issue 13 default surfaces.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn approved_interaction_voice_and_host_defaults() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_PlayerInteractionManager.IsReplacingUnit() == false)
        assert(C_VoiceChat.CanAccessSettings() == true)
        assert(UnitPowerDisplayMod(8) == 1)
        assert(UnitPower('player', 8, true) == UnitPower('player', 8) * UnitPowerDisplayMod(8))
        assert(select('#', FlashClientIcon()) == 0)
        assert(select('#', FlashClientIcon(true)) == 0)
    "#,
    )
    .unwrap();
}
