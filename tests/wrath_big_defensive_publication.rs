//! Wrath must not publish the defensive classification API.
#![cfg(feature = "client-wrath")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn wrath_does_not_publish_big_defensive() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_UnitAuras.AuraIsBigDefensive == nil)")
        .unwrap();
}
