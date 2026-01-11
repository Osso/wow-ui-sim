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

/// C_Timer.After is used for delayed execution.
#[test]
#[ignore = "C_Timer not implemented"]
fn test_c_timer_after() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        _G.timerFired = false
        C_Timer.After(0.1, function()
            _G.timerFired = true
        end)
        "#,
    )
    .unwrap();

    // Would need to advance time/tick the timer system
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
