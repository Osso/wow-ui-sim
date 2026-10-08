use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_8_0_1_world_position_requires_a_real_projection() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env.eval("return type(rawget(C_Map, 'GetMapPosFromWorldPos')) == 'function'").unwrap();
    assert!(published, "inverse projection has no raw-published implementation");
    let count: i32 = env.eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248))").unwrap();
    assert_eq!(count, 0);
}
