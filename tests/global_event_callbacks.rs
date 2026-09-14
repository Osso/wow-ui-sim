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

fn event_helper_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Event.lua environment");
    let path = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("synced Blizzard cache")
        .join("Blizzard_SharedXMLBase/Event.lua");
    let source = std::fs::read_to_string(&path).expect("read actual pinned Event.lua");
    env.exec(&source).expect("load actual pinned Event.lua");
    env
}

fn assert_event_helper_lifecycle(fire: fn(&WowLuaEnv)) {
    let env = event_helper_env();
    env.exec(
        r#"
        calls = 0
        handle = Event.RegisterCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', function(...)
            calls = calls + 1
            assert(select('#', ...) == 3, 'Event.lua strips exactly the nil owner')
            local index, label, flag = ...
            assert(index == 3 and label == 'raid' and flag == false)
            collectgarbage('collect')
        end)
        assert(calls == 0, 'validation must not execute callback')
        collectgarbage('collect')
        "#,
    )
    .expect("register through actual Event.lua");
    fire(&env);
    assert_eq!(env.eval::<i64>("return calls").unwrap(), 1);
    env.exec("handle:Unregister(); handle = nil; collectgarbage('collect')")
        .unwrap();
    fire(&env);
    assert_eq!(env.eval::<i64>("return calls").unwrap(), 1);
}

#[test]
fn global_event_callback_event_lua_rust_lifecycle() {
    assert_event_helper_lifecycle(fire_from_rust);
}

#[test]
fn global_event_callback_event_lua_synchronous_lifecycle() {
    assert_event_helper_lifecycle(fire_from_lua);
}

#[test]
fn global_event_callback_container_identity_cancellation_and_roots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        calls = 0
        local function callback(...)
            calls = calls + 1
            assert(select('#', ...) == 4)
            local owner, index, label, flag = ...
            assert(owner == nil and index == 3 and label == 'raid' and flag == false)
            collectgarbage('collect')
        end
        container = C_FunctionContainers.CreateCallback(callback)
        local other = C_FunctionContainers.CreateCallback(callback)
        RegisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', container)
        UnregisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', other)
        assert(calls == 0, 'validation must not invoke a container')
        collectgarbage('collect')
        "#,
    ).unwrap();
    fire_from_rust(&env);
    env.exec("assert(calls == 1); container:Cancel()").unwrap();
    fire_from_lua(&env);
    env.exec(
        r#"
        assert(calls == 1, 'cancelled container must not deliver')
        UnregisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', container)
        container._cancelled = false
        "#,
    ).unwrap();
    fire_from_rust(&env);
    env.exec(
        r#"
        assert(calls == 1, 'removal must preserve exact container identity')
        RegisterEventCallback('CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', container)
        container = nil
        collectgarbage('collect')
        "#,
    ).unwrap();
    fire_from_lua(&env);
    assert_eq!(env.eval::<i64>("return calls").unwrap(), 2);
}

#[test]
fn global_event_callback_rejects_unbranded_userdata_without_invoking() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local reads = 0
        local fake = newproxy(true)
        getmetatable(fake).__index = function()
            reads = reads + 1
            return function() error('must not execute') end
        end
        assert(not pcall(RegisterEventCallback, 'CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', fake))
        assert(not pcall(UnregisterEventCallback, 'CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX', fake))
        assert(reads == 0, 'validation must not duck-type userdata')
        "#,
    ).unwrap();
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
