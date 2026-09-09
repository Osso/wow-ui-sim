#![cfg(feature = "retail-12-1-5")]

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

    assert_eq!(env.eval::<String>("return type(signal_map)").unwrap(), "userdata");
    assert_eq!(env.eval::<String>("return signal_map.custom_field").unwrap(), "kept");
    assert_eq!(env.eval::<i64>("return signal_count_before").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_next_key").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_next_time").unwrap(), 0);

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return #signal_calls").unwrap(), 1);
    assert_eq!(env.eval::<i64>("return signal_calls[1]").unwrap(), 7);
    assert_eq!(env.eval::<bool>("return signal_map:HasSignal(7)").unwrap(), false);
    assert_eq!(env.eval::<Option<f64>>("return signal_map:GetSignalTime(7)").unwrap(), None);
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
    let env = WowLuaEnv::new().unwrap();
    let timer_util = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/data/ptr-fixtures/TimerUtil-12.1.5.69594.lua"
    );
    env.exec_named(&std::fs::read_to_string(timer_util).unwrap(), "TimerUtil.lua")
        .unwrap();
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

    assert_eq!(env.eval::<String>("return type(timer_util_map)").unwrap(), "userdata");
    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i64>("return timer_util_calls").unwrap(), 1);
}
