#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_input_util(env: &WowLuaEnv) {
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for file in ["Mainline/InputUtil.lua", "InputUtil.lua"] {
        let source = std::fs::read_to_string(root.join("Blizzard_SharedXML").join(file)).unwrap();
        env.exec(&source).unwrap();
    }
}

#[test]
fn forever_input_style_initializes_vendor_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    load_input_util(&env);
    env.exec(
        r#"
        assert(C_InputInterfaceStyle.GetCurrentStyle() == Enum.InputDeviceInterfaceType.Mkb)
        assert(Enum.InputDeviceInterfaceType.Mkb == 0)
        assert(Enum.InputDeviceInterfaceType.Gamepad == 1)
        local subscriber = {}
        local initialized = 0
        InputUtil.RegisterForInterfaceTransitions(subscriber)
        InputUtil.RegisterMKBInit(subscriber, function() initialized = initialized + 1 end)
        assert(initialized == 1)
        assert(InputUtil.IsMKBUIEnabled())
        assert(not InputUtil.IsGamepadUIEnabled())
    "#,
    )
    .unwrap();
}

#[test]
fn forever_input_style_dispatches_vendor_transitions_and_isolates_environments() {
    use rilua::Val;
    use wow_ui_sim::c_api::c_input_interface_style::InputInterfaceStyle;
    use wow_ui_sim::event::EventArg;
    let env = WowLuaEnv::new().unwrap();
    let other = WowLuaEnv::new().unwrap();
    load_input_util(&env);
    env.exec(
        r#"
        calls = {}
        local subscriber = {}
        InputUtil.RegisterForInterfaceTransitions(subscriber)
        InputUtil.RegisterMKBUninit(subscriber, function() table.insert(calls, 'leave') end)
        InputUtil.RegisterGamepadSetup(subscriber, function() table.insert(calls, 'setup') end)
        InputUtil.RegisterGamepadInit(subscriber, function() table.insert(calls, 'enter') end)
        InputUtil.RegisterInterfaceTransitionCallback(function(new, old)
            assert(new == 1 and old == 0)
            assert(C_InputInterfaceStyle.GetCurrentStyle() == new)
            table.insert(calls, 'payload')
        end)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Gamepad);
    let events = env.state().borrow_mut().events.drain();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].name, "INPUT_DEVICE_INTERFACE_TRANSITION");
    let args: Vec<Val> = events[0]
        .args
        .iter()
        .map(|arg| match arg {
            EventArg::Number(value) => Val::Num(*value),
            _ => panic!("expected numeric style payload"),
        })
        .collect();
    env.fire_event_with_args(&events[0].name, &args).unwrap();
    env.exec("assert(#calls == 4); local order = table.concat(calls, ','); assert(order == 'leave,setup,enter,payload' or order == 'payload,leave,setup,enter')").unwrap();
    assert_eq!(
        other
            .eval::<i32>("return C_InputInterfaceStyle.GetCurrentStyle()")
            .unwrap(),
        0
    );
    env.state()
        .borrow_mut()
        .set_input_interface_style(InputInterfaceStyle::Gamepad);
    assert!(env.state().borrow().events.is_empty());
}
