#![cfg(feature = "retail-12-0-7")]
use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.state().borrow_mut().housing.room_connection_door_types = [
        (("Room-A".into(), 7), [1, 2].into()),
        (("Room-A".into(), 8), [3].into()),
        (("Room-B".into(), 7), [3].into()),
    ]
    .into();
    env.exec(
        r#"
        function Door(room, component, kind, expected)
            local function check(...)
                assert(select('#', ...) == 1)
                local value = ...
                assert(type(value) == 'boolean' and value == expected and not issecretvalue(value))
            end
            check(C_HousingCustomizeMode.RoomConnectionSupportsDoorType(room, component, kind))
        end
    "#,
    )
    .unwrap();
    env
}

#[test]
fn door_compatibility_distinguishes_room_component_and_type_live() {
    let env = fixture();
    env.exec(
        r#"
        Door('Room-A', 7, 1, true); Door('Room-A', 7, 2, true)
        Door('Room-A', 7, 3, false); Door('Room-A', 8, 1, false)
        Door('Room-A', 8, 3, true); Door('Room-B', 7, 1, false)
        Door('Room-B', 7, 3, true); Door('Room-C', 7, 3, false)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .room_connection_door_types
        .insert(("Room-A".into(), 7), [3].into());
    env.exec("Door('Room-A', 7, 1, false); Door('Room-A', 7, 3, true)")
        .unwrap();
    env.state()
        .borrow_mut()
        .housing
        .room_connection_door_types
        .remove(&("Room-A".into(), 7));
    env.exec("Door('Room-A', 7, 3, false)").unwrap();
}

#[test]
fn door_empty_invalid_selectors_and_read_only_isolation() {
    let env = fixture();
    let before = env
        .state()
        .borrow()
        .housing
        .room_connection_door_types
        .clone();
    env.exec(
        r#"
        Door(nil, 7, 1, false); Door(17, 7, 1, false)
        Door('Room-A', nil, 1, false); Door('Room-A', '7', 1, false)
        Door('Room-A', 7.5, 1, false); Door('Room-A', 7, nil, false)
        Door('Room-A', 7, '1', false); Door('Room-A', 7, 1/0, false)
        Door('Room-A', 7, 1, true)
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().housing.room_connection_door_types,
        before
    );
    let other = WowLuaEnv::new().unwrap();
    let supported: bool = other
        .eval("return C_HousingCustomizeMode.RoomConnectionSupportsDoorType('Room-A', 7, 1)")
        .unwrap();
    assert!(!supported);
}

#[test]
fn door_authenticates_all_three_arguments_before_any_validation() {
    let env = fixture();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let room = wrap_host_secret_string(lua.state_mut(), "Room-A");
        lua.state_mut().push(room);
        lua.set_global_val("SecretRoom", room).unwrap();
        lua.state_mut().pop();
        for (name, number) in [("SecretComponent", 7.0), ("SecretDoor", 1.0)] {
            let value = wrap_host_secret_number(lua.state_mut(), number);
            lua.state_mut().push(value);
            lua.set_global_val(name, value).unwrap();
            lua.state_mut().pop();
        }
    }
    env.exec(r#"
        Door(SecretRoom, 7, 1, true); Door('Room-A', SecretComponent, 1, true)
        Door('Room-A', 7, SecretDoor, true); Door(SecretRoom, SecretComponent, SecretDoor, true)
        collectgarbage('collect'); Door(SecretRoom, SecretComponent, SecretDoor, true)
        local function probe()
            Door('Room-A', 7, 1, true)
            local calls = {
                function() return C_HousingCustomizeMode.RoomConnectionSupportsDoorType(SecretRoom, 7, 1) end,
                function() return C_HousingCustomizeMode.RoomConnectionSupportsDoorType(false, SecretComponent, 1) end,
                function() return C_HousingCustomizeMode.RoomConnectionSupportsDoorType(false, false, SecretDoor) end,
                function() return C_HousingCustomizeMode.RoomConnectionSupportsDoorType('Missing', 7, SecretDoor) end,
                function() return C_HousingCustomizeMode.RoomConnectionSupportsDoorType(false, false, false, SecretRoom) end,
            }
            for _, call in ipairs(calls) do
                local ok, err = pcall(call)
                assert(not ok and string.find(err, 'untainted caller', 1, true))
            end
            assert(debug.getstacktaint() == 'DoorProbe')
        end
        debug.setobjecttaint(probe, 'DoorProbe'); probe()
        assert(issecure() and issecretvalue(SecretRoom) and issecretvalue(SecretComponent) and issecretvalue(SecretDoor))
        Door(SecretRoom, SecretComponent, SecretDoor, true)
    "#).unwrap();
}
