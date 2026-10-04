#![cfg(feature = "retail-12-0-7")]
use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().housing.viewed_floor_permissions =
        [(-1, true), (0, true), (1, false)].into();
    env.exec(r#"
        function Floor(value, expected)
            local function check(...)
                assert(select('#', ...) == 1)
                local result = ...
                assert(type(result) == 'boolean' and result == expected and not issecretvalue(result))
            end
            check(C_HousingLayout.CanSetViewedFloor(value))
        end
    "#).unwrap();
    env
}

#[test]
fn floor_permissions_are_explicit_live_and_read_only() {
    let env = fixture();
    let before = env
        .state()
        .borrow()
        .housing
        .viewed_floor_permissions
        .clone();
    env.exec("Floor(-1, true); Floor(0, true); Floor(1, false); Floor(2, false)")
        .unwrap();
    assert_eq!(
        env.state().borrow().housing.viewed_floor_permissions,
        before
    );
    env.state()
        .borrow_mut()
        .housing
        .viewed_floor_permissions
        .insert(1, true);
    env.state()
        .borrow_mut()
        .housing
        .viewed_floor_permissions
        .insert(0, false);
    env.exec("Floor(1, true); Floor(0, false)").unwrap();
    env.state()
        .borrow_mut()
        .housing
        .viewed_floor_permissions
        .remove(&1);
    env.exec("Floor(1, false); Floor(-1, true)").unwrap();
}

#[test]
fn floor_missing_malformed_and_independent_state() {
    let env = fixture();
    env.exec(
        r#"
        Floor(nil, false); Floor(false, false); Floor('0', false)
        Floor({}, false); Floor(0.5, false); Floor(0/0, false); Floor(1/0, false)
        Floor(-1, true)
    "#,
    )
    .unwrap();
    let other = WowLuaEnv::new().unwrap();
    let result: bool = other
        .eval("return C_HousingLayout.CanSetViewedFloor(-1)")
        .unwrap();
    assert!(!result);
}

#[test]
fn floor_secret_selector_and_extra_authentication_precede_miss() {
    let env = fixture();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let value = wrap_host_secret_number(lua.state_mut(), -1.0);
        lua.state_mut().push(value);
        lua.set_global_val("FloorSecret", value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(
        r#"
        Floor(FloorSecret, true); collectgarbage('collect'); Floor(FloorSecret, true)
        local function probe()
            Floor(-1, true)
            for _, call in ipairs({
                function() return C_HousingLayout.CanSetViewedFloor(FloorSecret) end,
                function() return C_HousingLayout.CanSetViewedFloor(false, FloorSecret) end,
                function() return C_HousingLayout.CanSetViewedFloor(1000, FloorSecret) end,
            }) do
                local ok, err = pcall(call)
                assert(not ok and string.find(err, 'untainted caller', 1, true))
            end
            assert(debug.getstacktaint() == 'FloorProbe')
        end
        debug.setobjecttaint(probe, 'FloorProbe'); probe()
        assert(issecure() and issecretvalue(FloorSecret)); Floor(FloorSecret, true)
    "#,
    )
    .unwrap();
}
