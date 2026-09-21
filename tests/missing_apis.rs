//! Tests for APIs we need to implement.
//! These tests document what's missing and will fail until implemented.

use wow_ui_sim::lua_api::WowLuaEnv;

/// strsplit is a WoW utility function used in slash command parsing.
#[test]
fn test_strsplit() {
    let env = WowLuaEnv::new().unwrap();

    let result: (String, String) = env
        .eval(
            r#"
        local cmd, arg = strsplit(" ", "toggle debug", 2)
        return cmd, arg
        "#,
        )
        .unwrap();

    assert_eq!(result.0, "toggle");
    assert_eq!(result.1, "debug");
}

/// SlashCmdList is a global table for registering slash commands.
#[test]
#[ignore = "SlashCmdList not implemented"]
fn test_slash_command_registration() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.cmdExecuted = false
        SlashCmdList["MYCOMMAND"] = function(msg)
            _G.cmdExecuted = true
            _G.cmdMsg = msg
        end
        SLASH_MYCOMMAND1 = "/mycommand"
        "#,
    )
    .unwrap();

    // Would need a way to execute slash commands
    // env.execute_slash_command("/mycommand test");
}

/// TimerCallback has no arguments; zero delay still waits for timer processing.
#[test]
fn timer_after_plain_callback_is_deferred_once_without_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local calls = 0
        timerAfter = { calls = 0, nargs = -1 }
        local function callback(...)
            calls = calls + 1
            timerAfter.calls = calls
            timerAfter.nargs = select('#', ...)
        end
        assert(select('#', C_Timer.After(0, callback)) == 0)
        assert(timerAfter.calls == 0)
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 0);
    let (calls, nargs): (i32, i32) = env
        .eval("return timerAfter.calls, timerAfter.nargs")
        .unwrap();
    assert_eq!(calls, 1, "After must invoke the captured closure once");
    assert_eq!(nargs, 0, "After must not inject a callback argument");
}

#[test]
fn timer_after_container_callback_is_deferred_once_without_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local calls = 0
        timerAfter = { calls = 0, nargs = -1 }
        timerAfter.container = C_FunctionContainers.CreateCallback(function(...)
            calls = calls + 1
            timerAfter.calls = calls
            timerAfter.nargs = select('#', ...)
        end)
        assert(type(timerAfter.container) == 'userdata')
        assert(select('#', C_Timer.After(0, timerAfter.container)) == 0)
        assert(timerAfter.calls == 0)
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 0);
    let (calls, nargs): (i32, i32) = env
        .eval("return timerAfter.calls, timerAfter.nargs")
        .unwrap();
    assert_eq!(calls, 1, "After must invoke the container's closure once");
    assert_eq!(nargs, 0, "After must not inject a container proxy");
}

/// TickerCallback retains its one-proxy argument, unlike After's TimerCallback.
#[test]
fn timer_after_new_timer_control_receives_handle_proxy() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        timerControl = { calls = 0 }
        local handle
        handle = C_Timer.NewTimer(0, function(...)
            timerControl.calls = timerControl.calls + 1
            timerControl.nargs = select('#', ...)
            timerControl.proxy = ...
        end)
        handle.marker = 'new-timer'
        timerControl.handle = handle
        assert(type(handle) == 'userdata' and timerControl.calls == 0)
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 0);
    env.exec(
        r#"
        local result = timerControl
        assert(result.calls == 1 and result.nargs == 1)
        assert(result.proxy == result.handle)
        assert(({[result.handle] = true})[result.proxy] == nil)
        assert(result.proxy.marker == 'new-timer')
        "#,
    )
    .unwrap();
}

#[test]
fn timer_after_new_ticker_control_receives_handle_proxy_per_tick() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        tickerControl = { calls = 0, arguments = {}, proxies = {} }
        tickerControl.handle = C_Timer.NewTicker(0, function(...)
            local result = tickerControl
            result.calls = result.calls + 1
            result.arguments[result.calls] = select('#', ...)
            result.proxies[result.calls] = ...
        end, 2)
        tickerControl.handle.marker = 'new-ticker'
        assert(type(tickerControl.handle) == 'userdata' and tickerControl.calls == 0)
        "#,
    )
    .unwrap();

    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.eval::<i32>("return tickerControl.calls").unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 1);
    assert_eq!(env.process_timers().unwrap(), 0);
    env.exec(
        r#"
        local result = tickerControl
        assert(result.calls == 2)
        for i = 1, 2 do
            assert(result.arguments[i] == 1)
            assert(result.proxies[i] == result.handle)
            assert(({[result.handle] = true})[result.proxies[i]] == nil)
            assert(result.proxies[i].marker == 'new-ticker')
        end
        "#,
    )
    .unwrap();
}

#[test]
fn timer_nested_callbacks_preserve_children_and_existing_tickers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(include_str!("fixtures/nested_timers.lua"))
        .unwrap();
    for (pass, expected_fired) in [(1, 2), (2, 4), (3, 2), (4, 0)] {
        assert_eq!(env.process_timers().unwrap(), expected_fired);
        env.exec(&format!("CheckNestedTimerPass({pass})")).unwrap();
    }
}

#[test]
fn timer_nested_callbacks_preserve_pending_timers_and_cancel_taken_timers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        pendingCalls, childCalls, cancelledCalls = 0, 0, 0
        C_Timer.NewTimer(0.2, function() pendingCalls = pendingCalls + 1 end)
        local cancelled
        C_Timer.After(0, function()
            cancelled:Cancel()
            C_Timer.NewTimer(0, function() childCalls = childCalls + 1 end)
        end)
        cancelled = C_Timer.NewTimer(0, function() cancelledCalls = cancelledCalls + 1 end)
        "#,
    )
    .unwrap();
    assert_eq!(env.process_timers().unwrap(), 1);
    env.exec("assert(pendingCalls == 0 and childCalls == 0 and cancelledCalls == 0)")
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(250));
    assert_eq!(env.process_timers().unwrap(), 2);
    assert_eq!(env.process_timers().unwrap(), 0);
    env.exec("assert(pendingCalls == 1 and childCalls == 1 and cancelledCalls == 0)")
        .unwrap();
}

/// Game API functions that need mocking.
#[test]
#[ignore = "Game APIs not implemented"]
fn test_game_api_stubs() {
    let env = WowLuaEnv::new().unwrap();

    // These should return sensible mock values
    env.exec(
        r#"
        local name, instanceType, difficultyID, difficultyName, maxPlayers,
              dynamicDifficulty, isDynamic, instanceID = GetInstanceInfo()

        -- Should not error, should return something
        assert(name ~= nil, "GetInstanceInfo should return name")
        "#,
    )
    .unwrap();
}

/// hooksecurefunc is used to hook into existing functions.
#[test]
fn test_hooksecurefunc() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.originalCalled = false
        _G.hookCalled = false

        function SomeGlobalFunction()
            _G.originalCalled = true
        end

        hooksecurefunc("SomeGlobalFunction", function()
            _G.hookCalled = true
        end)

        SomeGlobalFunction()
        "#,
    )
    .unwrap();

    let orig: bool = env.eval("return _G.originalCalled").unwrap();
    let hook: bool = env.eval("return _G.hookCalled").unwrap();

    assert!(orig, "Original function should be called");
    assert!(hook, "Hook should be called after original");
}

/// wipe() clears a table in place.
#[test]
fn test_wipe() {
    let env = WowLuaEnv::new().unwrap();

    let count: i32 = env
        .eval(
            r#"
        local t = {1, 2, 3, a = "b"}
        wipe(t)
        local count = 0
        for _ in pairs(t) do count = count + 1 end
        return count
        "#,
        )
        .unwrap();

    assert_eq!(count, 0, "Table should be empty after wipe");
}

/// tinsert is table.insert but global.
#[test]
fn test_tinsert() {
    let env = WowLuaEnv::new().unwrap();

    let result: i32 = env
        .eval(
            r#"
        local t = {}
        tinsert(t, "a")
        tinsert(t, "b")
        return #t
        "#,
        )
        .unwrap();

    assert_eq!(result, 2);
}

/// _G global table access.
#[test]
fn test_global_table_access() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.myGlobal = "hello"
        _G["anotherGlobal"] = 42
        "#,
    )
    .unwrap();

    let s: String = env.eval("return _G.myGlobal").unwrap();
    let n: i32 = env.eval("return _G.anotherGlobal").unwrap();

    assert_eq!(s, "hello");
    assert_eq!(n, 42);
}

/// Frame:CreateTexture for creating texture widgets.
#[test]
#[ignore = "CreateTexture not implemented"]
fn test_create_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local tex = frame:CreateTexture("MyTexture")
        tex:SetSize(100, 100)
        tex:SetTexture("Interface\\Icons\\Spell_Nature_Heal")
        tex:SetTexCoord(0, 1, 0, 1)
        tex:SetPoint("CENTER")
        "#,
    )
    .unwrap();
}

/// Frame:CreateFontString for creating text widgets.
#[test]
#[ignore = "CreateFontString not implemented"]
fn test_create_font_string() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame")
        local fs = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        fs:SetText("Hello World")
        fs:SetPoint("CENTER")
        "#,
    )
    .unwrap();
}

/// LibStub is commonly used for library management.
#[test]
#[ignore = "LibStub not implemented"]
fn test_libstub() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        -- LibStub pattern
        local lib = LibStub("SomeLib-1.0")
        "#,
    )
    .unwrap();
}

/// GetBuildInfo returns game version info.
#[test]
fn test_get_build_info() {
    let env = WowLuaEnv::new().unwrap();

    let version: String = env
        .eval(
            r#"
        local version = GetBuildInfo()
        return version
        "#,
        )
        .unwrap();

    // Should return something like "11.0.0"
    assert!(!version.is_empty());
}

/// Settings API for modern addon options.
#[test]
#[ignore = "Settings API not implemented"]
fn test_settings_api() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local category, layout = Settings.RegisterVerticalLayoutCategory("MyAddon")
        local setting = Settings.RegisterAddOnSetting(
            category, "mykey", "varkey", {}, "boolean", "My Setting", true
        )
        Settings.CreateCheckbox(category, setting, "Tooltip text")
        Settings.RegisterAddOnCategory(category)
        "#,
    )
    .unwrap();
}
