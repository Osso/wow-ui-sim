#![cfg(any(feature = "client-retail", feature = "client-ptr", feature = "client-wowforever", feature = "client-mists", feature = "client-era", feature = "client-anniversary"))]

#[test]
fn min_item_level_has_no_floor_without_scaling() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec("assert(select('#', C_PaperDollInfo.GetMinItemLevel()) == 1); assert(C_PaperDollInfo.GetMinItemLevel() == nil)").unwrap();
}
