#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

const EVENT: &str = "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX";

fn registered_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create callback environment");
    env.exec(
        r#"
        calls = 0
        callback = function(...)
            calls = calls + 1
            receivedCount = select('#', ...)
            receivedOwner, receivedIndex, receivedLabel, receivedFlag = ...
        end
        RegisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', callback)
        "#,
    )
    .expect("register global callback");
    env
}

fn fire_from_rust(env: &WowLuaEnv) {
    env.fire_event_with_args(
        EVENT,
        &[
            rilua::Val::Num(3.0),
            env.lua_string("raid"),
            rilua::Val::Bool(false),
        ],
    )
    .expect("fire callback event from Rust");
}

fn fire_from_lua(env: &WowLuaEnv) {
    env.exec("FireEvent('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', 3, 'raid', false)")
        .expect("fire callback event from Lua");
}

fn assert_lifecycle(fire: fn(&WowLuaEnv)) {
    let env = registered_env();
    fire(&env);
    env.exec(
        r#"
        assert(calls == 1, 'registered callback must run once')
        assert(receivedCount == 4, 'nil owner plus exact payload arity')
        assert(receivedOwner == nil, 'global callback owner must be nil')
        assert(receivedIndex == 3 and receivedLabel == 'raid' and receivedFlag == false,
            'callback must receive unchanged payload')
        UnregisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', function() end)
        "#,
    )
    .expect("check payload and remove different callback identity");
    fire(&env);
    env.exec(
        r#"
        assert(calls == 2, 'removing another identity must preserve callback')
        UnregisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', callback)
        "#,
    )
    .expect("remove exact registered callback");
    fire(&env);
    assert_eq!(env.eval::<i64>("return calls").unwrap(), 2);
}

#[test]
fn global_event_callback_rust_dispatch_lifecycle() {
    assert_lifecycle(fire_from_rust);
}

#[test]
fn global_event_callback_lua_dispatch_lifecycle() {
    assert_lifecycle(fire_from_lua);
}

#[test]
fn global_event_callback_return_arities() {
    let env = WowLuaEnv::new().expect("create callback environment");
    env.exec(
        r#"
        local callback = function() end
        local function checkRegistration(...)
            assert(select('#', ...) == 1, 'register must return exactly one value')
            assert(type((...)) == 'boolean', 'register must return a boolean')
        end
        checkRegistration(RegisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', callback))
        assert(select('#', UnregisterEventCallback(
            'CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', callback)) == 0,
            'unregister must return no values')
        "#,
    )
    .expect("global callback return contract");
}

#[test]
fn global_event_callback_environments_are_isolated() {
    let first = registered_env();
    let second = registered_env();
    fire_from_rust(&first);
    assert_eq!(first.eval::<i64>("return calls").unwrap(), 1);
    assert_eq!(second.eval::<i64>("return calls").unwrap(), 0);
    fire_from_lua(&second);
    assert_eq!(first.eval::<i64>("return calls").unwrap(), 1);
    assert_eq!(second.eval::<i64>("return calls").unwrap(), 1);
    first
        .exec("UnregisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', callback)")
        .unwrap();
    fire_from_lua(&first);
    fire_from_rust(&second);
    assert_eq!(first.eval::<i64>("return calls").unwrap(), 1);
    assert_eq!(second.eval::<i64>("return calls").unwrap(), 2);
}
