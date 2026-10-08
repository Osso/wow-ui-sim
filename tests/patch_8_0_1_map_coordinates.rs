use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_8_0_1_world_position_requires_a_real_projection() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env.eval("return type(rawget(C_Map, 'GetMapPosFromWorldPos')) == 'function'").unwrap();
    assert!(published, "inverse projection has no raw-published implementation");
    let count: i32 = env.eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248))").unwrap();
    assert_eq!(count, 0);
}

// Explicit world-rectangle fixtures, not native map geography parity.

#[test]
fn patch_8_0_1_world_position_projects_into_explicit_map_rectangles() {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().map_world_rects.insert(
        2248,
        wow_ui_sim::c_api::map_world_coordinates::MapWorldRect {
            continent_id: 42,
            left: -200.0,
            right: 600.0,
            top: 100.0,
            bottom: 500.0,
        },
    );
    env.eval::<()>(
        r#"
        assert(type(rawget(C_Map, 'GetMapPosFromWorldPos')) == 'function')
        local id, point = C_Map.GetMapPosFromWorldPos(42, {x=0, y=400}, 2248)
        assert(id == 2248)
        local x, y = point:GetXY()
        assert(x == 0.25 and y == 0.75)
        local autoID, autoPoint = C_Map.GetMapPosFromWorldPos(42, {x=0, y=400})
        assert(autoID == id and autoPoint.x == x and autoPoint.y == y)
        local _, edge = C_Map.GetMapPosFromWorldPos(42, {x=600, y=100}, 2248)
        assert(edge.x == 1 and edge.y == 0)
        assert(select('#', C_Map.GetMapPosFromWorldPos(43, {x=0,y=400}, 2248)) == 0)
        assert(select('#', C_Map.GetMapPosFromWorldPos(42, {x=601,y=400}, 2248)) == 0)
        assert(select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 84)) == 0)
        assert(not pcall(C_Map.GetMapPosFromWorldPos, 42, {x='bad',y=400}, 2248))
        "#,
    )
    .unwrap();
    // Rectangle updates change output; no cached/fabricated map result.
    env.state().borrow_mut().map_world_rects.get_mut(&2248).unwrap().right = 200.0;
    let x: f64 = env
        .eval("local _, p = C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248); return p.x")
        .unwrap();
    assert_eq!(x, 0.5);
    let other = WowLuaEnv::new().unwrap();
    let count: i32 = other
        .eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248))")
        .unwrap();
    assert_eq!(count, 0);
}
