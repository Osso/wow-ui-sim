#![cfg(feature = "timed-signal-maps")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn timed_signal_map_replaces_cancels_and_dispatches_due_key() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r##"
        signal_calls = {}
        signal_map = C_Timer.NewTimedSignalMap(function(...)
            assert(select("#", ...) == 1)
            local key = ...
            assert(type(key) == "number")
            table.insert(signal_calls, key)
        end)
        signal_map.custom_field = "kept"
        signal_map:SignalAt(1, 100)
        signal_map:SignalAt(2, 0)
        signal_map:SignalAt(1, 0)
        signal_map:CancelSignal(2)
        signal_count_before = signal_map:GetSignalCount()
        signal_next_key, signal_next_time = signal_map:GetNextSignal()
        signal_map:SignalAfter(3, 100)
        signal_map:CancelAllSignals()
        signal_map:SignalAt(7, 0)
        "##,
    )
    .unwrap();

    assert_eq!(
        env.eval::<String>("return type(signal_map)").unwrap(),
        "userdata"
    );
    assert_eq!(
        env.eval::<String>("return signal_map.custom_field")
            .unwrap(),
        "kept"
    );
    assert_eq!(env.eval::<i64>("return signal_count_before").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_next_key").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_next_time").unwrap(), 0);

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return #signal_calls").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_calls[1]").unwrap(), 7);
    assert_eq!(
        env.eval::<bool>("return signal_map:HasSignal(7)").unwrap(),
        false
    );
    assert_eq!(
        env.eval::<Option<f64>>("return signal_map:GetSignalTime(7)")
            .unwrap(),
        None
    );
}

#[test]
fn timed_signal_map_accepts_secure_secret_deadline_before_enforcing_map_access() {
    use rilua::table_security::wrap_host_secret_number;
    use rilua::LuaApiMut;

    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    for filename in ["MathUtil.lua", "TimerUtil.lua"] {
        let path = addons.join("Blizzard_SharedXMLBase").join(filename);
        env.exec_named(&std::fs::read_to_string(path).unwrap(), filename)
            .unwrap();
    }
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let deadline = wrap_host_secret_number(lua.state_mut(), 25.0);
        lua.state_mut().push(deadline);
        let inserted = lua.set_global_val("secret_deadline", deadline);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret deadline");
    }
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        secret_map = TimerUtil.CreateTimedSignalCallbackMap()
        secret_key = secret_map:RegisterCallback(function() end)
        secret_map:SignalAt(secret_key, 12.5)
        assert(secret_map:GetSignalCount() == 1)
        assert(secret_map:GetSignalTime(secret_key) == 12.5)
        public_map = TimerUtil.CreateTimedSignalCallbackMap()
        public_key = public_map:RegisterCallback(function() end)
        public_map:SignalAt(public_key, 12.5)
        "#,
    )
    .expect("unchanged cached wrapper schedules public deadlines");
    env.exec(
        r##"
        local function ingress()
            assert(debug.getstacktaint() == nil)
            assert(issecretvalue(secret_deadline))
            assert(select("#", secret_map:SignalAt(secret_key, secret_deadline)) == 0)
            assert(issecretvalue(secret_deadline))
            assert(secretunwrap(secret_deadline) == 25.0)
            assert(debug.getstacktaint() == nil)
        end
        ingress()
        "##,
    )
    .expect("untainted SignalAt accepts an authentic host-secret argument 3");
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        assert(secret_map:GetSignalCount() == 1)
        assert(secret_map:HasSignal(secret_key))
        local deadline = secret_map:GetSignalTime(secret_key)
        if issecretvalue(deadline) then
            deadline = secretunwrap(deadline)
        end
        assert(deadline == 25.0)
        local function guarded_probe()
            assert(debug.getstacktaint() == "TimedSignalMapProbe")
            local ok = pcall(function() return secret_map:GetSignalCount() end)
            assert(not ok, "tainted access to a secret-containing map must fail")
            assert(public_map:GetSignalCount() == 1)
            assert(public_map:GetSignalTime(public_key) == 12.5)
            assert(issecretvalue(secret_deadline))
            assert(debug.getstacktaint() == "TimedSignalMapProbe")
        end
        debug.setobjecttaint(guarded_probe, "TimedSignalMapProbe")
        guarded_probe()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("trusted replacement and tainted guarded access respect held secrets");
}

fn secret_deadline_fixture(time: f64) -> WowLuaEnv {
    use rilua::LuaApiMut;

    let env = WowLuaEnv::new().unwrap();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        for (name, time) in [("secret_deadline", time), ("secret_later_deadline", 111.5)] {
            let deadline = rilua::table_security::wrap_host_secret_number(lua.state_mut(), time);
            lua.state_mut().push(deadline);
            let inserted = lua.set_global_val(name, deadline);
            lua.state_mut().pop();
            inserted.unwrap();
        }
    }
    env.exec(
        r#"
        map = C_Timer.NewTimedSignalMap(function(key)
            -- ConditionalSecret callback keys are explicitly outside this lane.
            assert(type(key) == "number" and not issecretvalue(key))
            calls = (calls or 0) + 1
        end)
        map:SignalAt(2, 2000)
        "#,
    )
    .unwrap();
    env
}

fn run_tainted(env: &WowLuaEnv, body: &str) {
    env.exec(&format!(
        r#"
        assert(debug.getstacktaint() == nil)
        local function addon()
            assert(debug.getstacktaint() == "TimedSignalSecretTest")
            {body}
            assert(debug.getstacktaint() == "TimedSignalSecretTest")
        end
        debug.setobjecttaint(addon, "TimedSignalSecretTest")
        addon()
        assert(debug.getstacktaint() == nil)
        "#,
    ))
    .expect(body);
}

fn assert_secret_map_denied(env: &WowLuaEnv) {
    run_tainted(
        env,
        r#"
        local ok = pcall(function() return map:GetSignalCount() end)
        assert(not ok, "secret metadata must deny tainted map access")
        "#,
    );
}

#[test]
fn timed_signal_map_public_replacement_clears_deadline_secrecy() {
    let env = secret_deadline_fixture(73.125);
    env.exec("map:SignalAt(1, secret_deadline)").unwrap();
    assert_secret_map_denied(&env);
    env.exec("map:SignalAt(1, 37.75)").unwrap();
    run_tainted(
        &env,
        "assert(map:GetSignalCount() == 2); assert(map:GetSignalTime(1) == 37.75)",
    );
}

#[test]
fn timed_signal_map_cancel_clears_only_removed_deadline_secrecy() {
    let env = secret_deadline_fixture(61.875);
    env.exec("map:SignalAt(1, secret_deadline); map:SignalAt(3, secret_deadline)")
        .unwrap();
    assert_secret_map_denied(&env);
    env.exec("map:CancelSignal(1)").unwrap();
    assert_secret_map_denied(&env);
    env.exec("map:CancelSignal(3)").unwrap();
    run_tainted(
        &env,
        "assert(map:GetSignalCount() == 1); assert(map:GetSignalTime(2) == 2000)",
    );
}

#[test]
fn timed_signal_map_cancel_all_clears_deadline_secrecy() {
    let env = secret_deadline_fixture(92.625);
    env.exec("map:SignalAt(1, secret_deadline)").unwrap();
    assert_secret_map_denied(&env);
    env.exec("map:CancelAllSignals()").unwrap();
    run_tainted(
        &env,
        "assert(map:GetSignalCount() == 0); map:SignalAt(4, 16.25); assert(map:GetSignalTime(4) == 16.25)",
    );
}

#[test]
fn timed_signal_map_due_removal_clears_only_removed_deadline_secrecy() {
    let env = secret_deadline_fixture(0.0);
    env.exec("map:SignalAt(1, secret_deadline); map:SignalAt(3, secret_later_deadline)")
        .unwrap();
    assert_secret_map_denied(&env);
    assert_eq!(env.process_timers().unwrap(), 1);
    assert_secret_map_denied(&env);
    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(120);
    assert_eq!(env.process_timers().unwrap(), 1);
    run_tainted(
        &env,
        "assert(map:GetSignalCount() == 1); assert(map:GetSignalTime(2) == 2000)",
    );
    assert_eq!(env.eval::<i64>("return calls").unwrap(), 2);
}

#[test]
fn timed_signal_map_secret_after_retains_absolute_deadline_and_access_guard() {
    let env = secret_deadline_fixture(47.375);
    env.exec(
        r#"
        local before = GetTime()
        map:SignalAfter(1, secret_deadline)
        local deadline = map:GetSignalTime(1)
        assert(deadline >= before + 47.375 and deadline <= GetTime() + 47.375)
        assert(issecretvalue(secret_deadline) and secretunwrap(secret_deadline) == 47.375)
        "#,
    )
    .unwrap();
    assert_secret_map_denied(&env);
}

#[test]
fn timed_signal_map_rejected_tainted_secret_ingress_preserves_public_state() {
    for method in ["SignalAt", "SignalAfter"] {
        let env = secret_deadline_fixture(29.625);
        run_tainted(
            &env,
            &format!(
                r#"
                assert(not pcall(function() map:{method}(2, secret_deadline) end))
                assert(not pcall(function() map:{method}(7, secret_deadline) end))
                assert(map:GetSignalCount() == 1)
                assert(map:GetSignalTime(2) == 2000 and not map:HasSignal(7))
                assert(issecretvalue(secret_deadline))
                "#,
            ),
        );
        env.exec("assert(secretunwrap(secret_deadline) == 29.625)")
            .unwrap();
    }
}

#[test]
fn timed_signal_map_secret_holding_guard_covers_all_eight_methods_without_mutation() {
    for call in [
        "CancelAllSignals()",
        "CancelSignal(1)",
        "GetNextSignal()",
        "GetSignalCount()",
        "GetSignalTime(2)",
        "HasSignal(2)",
        "SignalAfter(3, 100)",
        "SignalAt(1, 100)",
    ] {
        let env = secret_deadline_fixture(84.375);
        env.exec(
            r#"
            map:SignalAt(1, secret_deadline)
            public_map = C_Timer.NewTimedSignalMap(function() end)
            public_map:SignalAt(1, 10)
            public_map:SignalAt(2, 20)
            "#,
        )
        .unwrap();
        run_tainted(
            &env,
            &format!(
                r#"
                assert(not pcall(function() return map:{call} end), "{call} must reject tainted access")
                assert(pcall(function() return public_map:{call} end), "{call} must allow public-only maps")
                "#,
            ),
        );
        env.exec(
            r#"
            assert(map:GetSignalCount() == 2)
            assert(map:GetSignalTime(1) == 84.375 and map:GetSignalTime(2) == 2000)
            assert(not map:HasSignal(3))
            local key, time = map:GetNextSignal()
            assert(key == 1 and time == 84.375)
            "#,
        )
        .expect(call);
    }
}

#[test]
fn timed_signal_map_rejects_nonfinite_secret_deadlines_without_mutation() {
    for time in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let env = secret_deadline_fixture(time);
        env.exec(
            r#"
            for _, method in ipairs({"SignalAt", "SignalAfter"}) do
                assert(not pcall(function() map[method](map, 2, secret_deadline) end))
                assert(map:GetSignalCount() == 1 and map:GetSignalTime(2) == 2000)
                assert(issecretvalue(secret_deadline))
            end
            "#,
        )
        .unwrap();
    }
}

#[test]
fn timed_signal_map_secret_finalizer_releases_state_from_tainted_collection() {
    let env = secret_deadline_fixture(0.0);
    env.exec("map:SignalAt(1, secret_deadline)").unwrap();
    assert_secret_map_denied(&env);
    run_tainted(&env, "map = nil; collectgarbage('collect')");
    assert_eq!(env.process_timers().unwrap(), 0);
    assert_eq!(env.eval::<Option<i64>>("return calls").unwrap(), None);
}

#[test]
fn timed_signal_map_reports_present_key_and_exact_time() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local map = C_Timer.NewTimedSignalMap(function() end)
        map:SignalAt(7, 12.5)
        assert(map:HasSignal(7) == true)
        assert(map:GetSignalTime(7) == 12.5)
        "#,
    )
    .unwrap();
}

#[test]
fn timed_signal_map_cancels_all_pending_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        cancelled_signal_calls = 0
        cancelled_map = C_Timer.NewTimedSignalMap(function()
            cancelled_signal_calls = cancelled_signal_calls + 1
        end)
        cancelled_map:SignalAfter(11, 10)
        cancelled_map:SignalAfter(22, 20)
        assert(cancelled_map:GetSignalCount() == 2)
        cancelled_map:CancelAllSignals()
        assert(cancelled_map:GetSignalCount() == 0)
        assert(cancelled_map:HasSignal(11) == false)
        assert(cancelled_map:HasSignal(22) == false)
        assert(cancelled_map:GetSignalTime(11) == nil)
        assert(cancelled_map:GetSignalTime(22) == nil)
        "#,
    )
    .unwrap();

    env.state().borrow_mut().start_time -= std::time::Duration::from_secs(30);
    assert_eq!(env.process_timers().unwrap(), 0);
    assert_eq!(env.process_timers().unwrap(), 0);
    assert_eq!(env.eval::<i64>("return cancelled_signal_calls").unwrap(), 0);
}

#[test]
fn timed_signal_map_defers_reentrant_signal_until_next_timer_pass() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        signal_calls = {}
        local map
        map = C_Timer.NewTimedSignalMap(function(key)
            table.insert(signal_calls, key)
            if key == 1 then
                map:SignalAt(2, 0)
            end
        end)
        map:SignalAt(1, 0)
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return #signal_calls").unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_calls[2]").unwrap(), 2);
}

#[test]
fn timed_signal_map_finalizer_drops_pending_signals() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local map = C_Timer.NewTimedSignalMap(function()
            error("a finalized signal map must not dispatch")
        end)
        map:SignalAt(1, 0)
        map = nil
        collectgarbage("collect")
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 0);
}

#[test]
fn timer_util_callback_map_uses_native_timed_signal_userdata() {
    let env = timer_util_env();
    env.exec(
        r#"
        timer_util_calls = 0
        timer_util_map = TimerUtil.CreateTimedSignalCallbackMap()
        timer_util_key = timer_util_map:RegisterCallback(function()
            timer_util_calls = timer_util_calls + 1
        end)
        timer_util_map:SignalAt(timer_util_key, 0)
        "#,
    )
    .unwrap();

    assert_eq!(
        env.eval::<String>("return type(timer_util_map)").unwrap(),
        "userdata"
    );
    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return timer_util_calls").unwrap(), 1);
}

/// TimerUtil.lua:7 calls CreateCounter, owned by Blizzard_SharedXMLBase
/// FunctionUtil.lua since 096d8a1f8c retired the bootstrap copy.
fn timer_util_env() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXMLBase"], &[]);
    let source = std::fs::read_to_string(timer_util_path()).unwrap();
    env.exec_named(&source, "TimerUtil.lua").unwrap();
    env
}

#[cfg(not(feature = "client-wowforever"))]
fn timer_util_path() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ptr-fixtures/TimerUtil-12.1.5.69594.lua"
    ))
}

#[cfg(feature = "client-wowforever")]
fn timer_util_path() -> std::path::PathBuf {
    wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_SharedXMLBase/TimerUtil.lua")
}
