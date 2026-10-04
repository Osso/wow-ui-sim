#![cfg(feature = "retail-12-0-7")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_secret};
use wow_ui_sim::lua_api::WowLuaEnv;

fn callback_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("pending callback environment");
    env.exec(
        r#"
        Slots = {
            {SetSecurePendingButtonCallback, GetSecurePendingButtonCallback},
            {SetSecurePendingPingOffScreenCallback, GetSecurePendingPingOffScreenCallback},
            {SetSecurePendingToggleRunCallback, GetSecurePendingToggleRunCallback},
        }
        Calls = {}
        Probe = CreateFrame('Frame')
        Probe:RegisterEvent('UNIT_HEALTH')
        Probe:SetScript('OnEvent', function(self, event, unit, amount, flag)
            assert(self == Probe and event == 'UNIT_HEALTH')
            assert(unit == 'player' and amount == 17 and flag == false)
            -- Test-owned consumer, NOT a native secure-input producer.
            for _, slot in ipairs(Slots) do
                local callback = slot[2]()
                if callback then callback() end
            end
        end)
        "#,
    )
    .expect("install real frame event listener and explicit consumer");
    env
}

fn fire_probe(env: &WowLuaEnv) {
    env.fire_event_with_args(
        "UNIT_HEALTH",
        &[
            env.lua_string("player"),
            rilua::Val::Num(17.0),
            rilua::Val::Bool(false),
        ],
    )
    .expect("real Rust event dispatcher");
}

fn install_secret_callback(env: &WowLuaEnv) {
    env.exec("SecretOriginal = function() table.insert(Calls, 'secret-function') end")
        .unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let original = lua.get_global_val("SecretOriginal");
    lua.state_mut().push(original);
    let wrapper = wrap_secret(lua.state_mut(), original).expect("authentic function secret");
    lua.state_mut().push(wrapper);
    lua.set_global_val("PendingSecretFunction", wrapper)
        .unwrap();
    lua.state_mut().pop();
    lua.state_mut().pop();
    let number = wrap_host_secret_number(lua.state_mut(), 23.0);
    lua.state_mut().push(number);
    lua.set_global_val("PendingSecretNumber", number).unwrap();
    lua.state_mut().pop();
}

#[test]
fn slots_replace_identity_clear_and_have_exact_public_arities() {
    let env = callback_env();
    env.exec(
        r#"
        for index, slot in ipairs(Slots) do
            assert(select('#', slot[2]()) == 1 and slot[2]() == nil)
            local first = function() table.insert(Calls, index * 10) end
            local second = function() table.insert(Calls, index * 100) end
            assert(select('#', slot[1](first)) == 0)
            assert(slot[2]() == first and select('#', slot[2]()) == 1)
            assert(select('#', slot[1](second)) == 0 and slot[2]() == second)
        end
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec(
        r#"
        assert(table.concat(Calls, ',') == '100,200,300')
        for _, slot in ipairs(Slots) do
            -- INFERRED global nil-clearing policy, no generated declaration.
            assert(select('#', slot[1](nil)) == 0 and slot[2]() == nil)
        end
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(#Calls == 3)").unwrap();
}

#[test]
fn namespace_and_legacy_ping_share_one_live_slot() {
    let env = callback_env();
    env.exec(
        r#"
        first = function() table.insert(Calls, 'namespace') end
        second = function() table.insert(Calls, 'legacy') end
        assert(select('#', C_PingSecure.SetPendingPingOffScreenCallback(first)) == 0)
        assert(GetSecurePendingPingOffScreenCallback() == first)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("SetSecurePendingPingOffScreenCallback(second)")
        .unwrap();
    fire_probe(&env);
    env.exec(
        r#"
        assert(table.concat(Calls, ',') == 'namespace,legacy')
        assert(select('#', C_PingSecure.ClearPendingPingOffScreenCallback()) == 0)
        assert(GetSecurePendingPingOffScreenCallback() == nil)
        C_PingSecure.ClearPendingPingOffScreenCallback()
        assert(GetSecurePendingPingOffScreenCallback() == nil)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(#Calls == 2)").unwrap();
}

#[test]
fn clear_ping_preserves_button_and_run_slots_and_environment_isolation() {
    let first = callback_env();
    let second = callback_env();
    first
        .exec(
            r#"
        SetSecurePendingButtonCallback(function() table.insert(Calls, 'button') end)
        SetSecurePendingToggleRunCallback(function() table.insert(Calls, 'run') end)
        C_PingSecure.SetPendingPingOffScreenCallback(function() table.insert(Calls, 'ping') end)
        C_PingSecure.ClearPendingPingOffScreenCallback()
        "#,
        )
        .unwrap();
    second
        .exec(
            r#"
        assert(GetSecurePendingButtonCallback() == nil)
        assert(GetSecurePendingToggleRunCallback() == nil)
        assert(GetSecurePendingPingOffScreenCallback() == nil)
        SetSecurePendingPingOffScreenCallback(function() table.insert(Calls, 'other-ping') end)
        "#,
        )
        .unwrap();
    fire_probe(&first);
    fire_probe(&second);
    first
        .exec("assert(table.concat(Calls, ',') == 'button,run')")
        .unwrap();
    second
        .exec("assert(table.concat(Calls, ',') == 'other-ping')")
        .unwrap();
}

#[test]
fn all_slots_root_closures_and_upvalues_across_full_gc() {
    let env = callback_env();
    env.exec(
        r#"
        Weak = setmetatable({}, {__mode = 'v'})
        for index, slot in ipairs(Slots) do
            local payload = {value = index * 37}
            local callback = function() table.insert(Calls, payload.value) end
            Weak[index] = callback
            slot[1](callback)
        end
        collectgarbage('collect')
        for index, slot in ipairs(Slots) do
            assert(Weak[index] ~= nil and slot[2]() == Weak[index])
        end
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == '37,74,111')")
        .unwrap();
}

#[test]
fn storage_operations_preserve_secure_and_tainted_caller_context() {
    let env = callback_env();
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            for _, slot in ipairs(Slots) do
                local callback = function() end
                slot[1](callback)
                assert(slot[2]() == callback)
                slot[1](nil)
                assert(slot[2]() == nil)
            end
            C_PingSecure.SetPendingPingOffScreenCallback(function() end)
            C_PingSecure.ClearPendingPingOffScreenCallback()
            assert(debug.getstacktaint() == before)
        end
        assert(issecure()); probe(); assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'PendingPublicFixture')
            probe()
            assert(debug.getstacktaint() == 'PendingPublicFixture')
        end
        debug.setobjecttaint(addon, 'PendingPublicFixture')
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn retrieved_callback_keeps_its_taint_when_event_consumer_invokes_it() {
    let env = callback_env();
    env.exec(
        r#"
        local callback = function()
            assert(debug.getstacktaint() == 'PendingCallbackFixture')
            table.insert(Calls, 'tainted-body')
        end
        debug.setobjecttaint(callback, 'PendingCallbackFixture')
        C_PingSecure.SetPendingPingOffScreenCallback(callback)
        assert(GetSecurePendingPingOffScreenCallback() == callback)
        collectgarbage('collect')
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'tainted-body')")
        .unwrap();
}

#[test]
fn replacement_and_clear_inside_callback_take_effect_on_next_event() {
    let env = callback_env();
    env.exec(
        r#"
        local replacement = function()
            table.insert(Calls, 'second')
            C_PingSecure.ClearPendingPingOffScreenCallback()
        end
        C_PingSecure.SetPendingPingOffScreenCallback(function()
            table.insert(Calls, 'first')
            C_PingSecure.SetPendingPingOffScreenCallback(replacement)
        end)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    fire_probe(&env);
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'first,second')")
        .unwrap();
}

#[test]
fn nested_real_fire_event_observes_replacement_without_duplicate_old_call() {
    let env = callback_env();
    env.exec(
        r#"
        local nested = function() table.insert(Calls, 'nested-new') end
        SetSecurePendingPingOffScreenCallback(function()
            table.insert(Calls, 'outer-old')
            SetSecurePendingPingOffScreenCallback(nested)
            FireEvent('UNIT_HEALTH', 'player', 17, false)
            table.insert(Calls, 'outer-return')
        end)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'outer-old,nested-new,outer-return')")
        .unwrap();
}

#[test]
fn callback_error_does_not_change_slot_or_break_later_dispatch() {
    let env = callback_env();
    env.exec(
        r#"
        failures = 0
        local callback = function()
            failures = failures + 1
            error('pending-callback-fixture-error')
        end
        C_PingSecure.SetPendingPingOffScreenCallback(callback)
        Probe:SetScript('OnEvent', function()
            -- Error isolation belongs to this explicit consumer's pcall.
            local ok, err = pcall(GetSecurePendingPingOffScreenCallback())
            assert(not ok and string.find(err, 'pending%-callback%-fixture%-error'))
            assert(GetSecurePendingPingOffScreenCallback() == callback)
            table.insert(Calls, 'continued')
        end)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    fire_probe(&env);
    env.exec("assert(failures == 2 and table.concat(Calls, ',') == 'continued,continued')")
        .unwrap();
    env.exec(
        r#"
        C_PingSecure.ClearPendingPingOffScreenCallback()
        SetSecurePendingPingOffScreenCallback(function() table.insert(Calls, 'recovered') end)
        Probe:SetScript('OnEvent', function() GetSecurePendingPingOffScreenCallback()() end)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'continued,continued,recovered')")
        .unwrap();
}

#[test]
fn namespace_rejects_malformed_callback_atomically_and_recovers() {
    let env = callback_env();
    env.exec(
        r#"
        local original = function() table.insert(Calls, 'retained') end
        C_PingSecure.SetPendingPingOffScreenCallback(original)
        for _, invalid in ipairs({false, 23, 'callback', {}, CreateFrame('Frame')}) do
            local ok, err = pcall(C_PingSecure.SetPendingPingOffScreenCallback, invalid)
            assert(not ok and type(err) == 'string')
            assert(GetSecurePendingPingOffScreenCallback() == original)
        end
        assert(not pcall(C_PingSecure.SetPendingPingOffScreenCallback))
        assert(not pcall(C_PingSecure.SetPendingPingOffScreenCallback, nil))
        assert(GetSecurePendingPingOffScreenCallback() == original)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'retained')")
        .unwrap();
}

#[test]
fn namespace_unwraps_authentic_function_for_secure_caller() {
    let env = callback_env();
    install_secret_callback(&env);
    env.exec(
        r#"
        assert(issecure() and issecretvalue(PendingSecretFunction))
        C_PingSecure.SetPendingPingOffScreenCallback(PendingSecretFunction)
        assert(GetSecurePendingPingOffScreenCallback() == SecretOriginal)
        collectgarbage('collect')
        assert(issecretvalue(PendingSecretFunction))
        assert(GetSecurePendingPingOffScreenCallback() == SecretOriginal)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'secret-function')")
        .unwrap();
}

#[test]
fn namespace_authenticates_extra_arguments_before_type_validation() {
    let env = callback_env();
    install_secret_callback(&env);
    env.exec(
        r#"
        original = function() table.insert(Calls, 'preserved') end
        C_PingSecure.SetPendingPingOffScreenCallback(original)
        local function addon()
            local before = debug.getstacktaint()
            local function denied(...)
                local ok, err = pcall(C_PingSecure.SetPendingPingOffScreenCallback, ...)
                assert(not ok and string.find(err, 'untainted caller', 1, true))
                assert(GetSecurePendingPingOffScreenCallback() == original)
            end
            denied(PendingSecretFunction)
            denied(PendingSecretNumber)
            -- Malformed public arg1 must not mask denied extra arguments.
            denied(23, PendingSecretNumber)
            denied(original, false, PendingSecretFunction)
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(addon, 'PendingDeniedFixture')
        addon()
        collectgarbage('collect')
        assert(issecretvalue(PendingSecretFunction) and issecretvalue(PendingSecretNumber))
        assert(GetSecurePendingPingOffScreenCallback() == original)
        -- Secure callers authenticate public/secret extras, then validate arg1.
        assert(not pcall(C_PingSecure.SetPendingPingOffScreenCallback, 23, PendingSecretNumber))
        assert(not pcall(C_PingSecure.SetPendingPingOffScreenCallback, PendingSecretNumber))
        C_PingSecure.SetPendingPingOffScreenCallback(original, PendingSecretNumber)
        "#,
    )
    .unwrap();
    fire_probe(&env);
    env.exec("assert(table.concat(Calls, ',') == 'preserved')")
        .unwrap();
}
