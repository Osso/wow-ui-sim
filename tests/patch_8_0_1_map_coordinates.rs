use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_8_0_1_world_position_requires_a_real_projection() {
    let env = WowLuaEnv::new().unwrap();
    let published: bool = env
        .eval("return type(rawget(C_Map, 'GetMapPosFromWorldPos')) == 'function'")
        .unwrap();
    assert!(
        published,
        "inverse projection has no raw-published implementation"
    );
    let count: i32 = env
        .eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248))")
        .unwrap();
    assert_eq!(count, 0);
}

// Explicit world-rectangle fixtures, not native map geography parity.

#[test]
fn patch_8_0_1_map_position_to_world_prefers_explicit_rectangle() {
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
    let (continent_id, x, y): (i32, f64, f64) = env
        .eval(
            r#"
            local id, worldPos = C_Map.GetWorldPosFromMapPos(2248, {x=0.25, y=0.75})
            local x, y = worldPos:GetXY()
            return id, x, y
            "#,
        )
        .unwrap();

    assert_eq!(continent_id, 42);
    assert_eq!(x, 0.0);
    assert_eq!(y, 400.0);
    let outside_count: i32 = env
        .eval("return select('#', C_Map.GetWorldPosFromMapPos(2248, {x=1.01, y=0.75}))")
        .unwrap();
    assert_eq!(outside_count, 0);
}

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
    env.state()
        .borrow_mut()
        .map_world_rects
        .get_mut(&2248)
        .unwrap()
        .right = 200.0;
    let x: f64 = env
        .eval("local _, p = C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248); return p.x")
        .unwrap();
    assert_eq!(x, 0.5);
    let rectangle = env.state().borrow().map_world_rects[&2248];
    env.state()
        .borrow_mut()
        .map_world_rects
        .insert(84, rectangle);
    env.eval::<()>(
        r#"
        assert(select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400})) == 0)
        local id, p = C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 84)
        assert(id == 84 and p.x == 0.5)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .map_world_rects
        .get_mut(&84)
        .unwrap()
        .left = 200.0;
    env.state()
        .borrow_mut()
        .map_world_rects
        .get_mut(&84)
        .unwrap()
        .right = -200.0;
    env.eval::<()>(
        r#"
        local id, p = C_Map.GetMapPosFromWorldPos(42, {x=100,y=400}, 84)
        assert(id == 84 and p.x == 0.25)
        assert(select('#', C_Map.GetMapPosFromWorldPos(42, {x=0/0,y=400}, 84)) == 0)
        assert(select('#', C_Map.GetMapPosFromWorldPos(42, {x=math.huge,y=400}, 84)) == 0)
        "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .map_world_rects
        .get_mut(&84)
        .unwrap()
        .right = 200.0;
    let degenerate: i32 = env
        .eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=200,y=400}, 84))")
        .unwrap();
    assert_eq!(degenerate, 0);
    let other = WowLuaEnv::new().unwrap();
    let count: i32 = other
        .eval("return select('#', C_Map.GetMapPosFromWorldPos(42, {x=0,y=400}, 2248))")
        .unwrap();
    assert_eq!(count, 0);
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_world_to_map_argument_errors_include_exact_usage() {
    let env = WowLuaEnv::new().unwrap();
    env.eval::<()>(
        r#"
        local usage = "Usage: local uiMapID, mapPosition = C_Map.GetMapPosFromWorldPos(continentID, worldPosition [, overrideUiMapID])"
        local function expectUsage(...)
            local ok, message = pcall(C_Map.GetMapPosFromWorldPos, ...)
            assert(not ok, "invalid world-to-map arguments must raise an argument error")
            assert(string.find(tostring(message), usage, 1, true), tostring(message))
        end
        expectUsage("bad", {x=0, y=400}, 2248)
        expectUsage(42)
        expectUsage(42, nil, 2248)
        expectUsage(42, 42, 2248)
        expectUsage(42, {x="0", y=400}, 2248)
        expectUsage(42, {x=0, y=400}, "bad")
        "#,
    )
    .unwrap();
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
#[test]
fn retail_world_to_map_truncates_fractional_ids() {
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
        local id, point = C_Map.GetMapPosFromWorldPos(42.9, {x=0, y=400}, 2248.9)
        assert(id == 2248 and point.x == 0.25 and point.y == 0.75)
        local autoID, autoPoint = C_Map.GetMapPosFromWorldPos(42.9, {x=0, y=400}, nil)
        assert(autoID == 2248 and autoPoint.x == 0.25 and autoPoint.y == 0.75)
        "#,
    )
    .unwrap();
}
