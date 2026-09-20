#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn forever_publishes_lair_lfg_category_constant() {
    let env = WowLuaEnv::new().unwrap();
    assert_eq!(env.eval::<i32>("return LE_LFG_CATEGORY_LAIR").unwrap(), 8);
}
