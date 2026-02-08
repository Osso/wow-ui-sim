//! Tests for WoW error handler system and internal table visibility.
//!
//! BugSack/BugGrabber pattern: addon calls seterrorhandler() to intercept
//! errors from script dispatch. These tests verify the full Rust→Lua→error→handler
//! pipeline works correctly.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ── seterrorhandler / geterrorhandler basics ─────────────────────────

#[test]
fn test_seterrorhandler_geterrorhandler_roundtrip() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local errs = {}
            seterrorhandler(function(msg) table.insert(errs, msg) end)
            local h = geterrorhandler()
            h("test error")
            return #errs
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_geterrorhandler_default_is_function() {
    let env = env();
    let is_func: bool = env
        .eval("return type(geterrorhandler()) == 'function'")
        .unwrap();
    assert!(is_func);
}

// ── Internal tables hidden from Lua ──────────────────────────────────

#[test]
fn test_internal_tables_hidden_from_lua() {
    let env = env();
    let (scripts, hooks, fields): (String, String, String) = env
        .eval("return type(__scripts), type(__script_hooks), type(__frame_fields)")
        .unwrap();
    assert_eq!(scripts, "nil", "__scripts should not be visible");
    assert_eq!(hooks, "nil", "__script_hooks should not be visible");
    assert_eq!(fields, "nil", "__frame_fields should not be visible");
}

#[test]
fn test_internal_tables_hidden_after_script_set() {
    // Even after SetScript creates entries, the tables stay hidden
    let env = env();
    let (scripts, fields): (String, String) = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            f:SetScript("OnEvent", function() end)
            f.customField = 42
            return type(__scripts), type(__frame_fields)
            "#,
        )
        .unwrap();
    assert_eq!(scripts, "nil");
    assert_eq!(fields, "nil");
}

// ── OnShow script errors route through error handler ─────────────────

#[test]
fn test_error_handler_receives_onshow_errors() {
    let env = env();
    let (count, msg): (i32, String) = env
        .eval(
            r#"
            local errs = {}
            seterrorhandler(function(msg) table.insert(errs, msg) end)
            local f = CreateFrame("Frame")
            f:SetScript("OnShow", function() error("onshow boom") end)
            f:Hide()
            f:Show()
            return #errs, errs[1] or ""
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert!(msg.contains("onshow boom"), "error message was: {msg}");
}

// ── Event dispatch errors route through error handler (BugSack path) ─

#[test]
fn test_error_handler_receives_event_dispatch_errors() {
    // This is the core BugSack pattern: addon sets error handler,
    // Rust fires an event, Lua OnEvent handler errors, error handler called.
    let env = env();
    env.exec(
        r#"
        BugSackErrors = {}
        seterrorhandler(function(msg)
            table.insert(BugSackErrors, msg)
        end)

        local f = CreateFrame("Frame")
        f:RegisterEvent("PLAYER_LOGIN")
        f:SetScript("OnEvent", function(self, event)
            error("addon crashed in " .. event)
        end)
        "#,
    )
    .unwrap();

    // Fire the event from Rust (the real dispatch path)
    env.fire_event("PLAYER_LOGIN").ok();

    let (count, msg): (i32, String) = env
        .eval("return #BugSackErrors, BugSackErrors[1] or ''")
        .unwrap();
    assert_eq!(count, 1, "error handler should have received exactly 1 error");
    assert!(
        msg.contains("addon crashed in PLAYER_LOGIN"),
        "error message was: {msg}"
    );
}

#[test]
fn test_error_handler_receives_event_args() {
    // Verify that when OnEvent errors, the error handler is called
    // even when the event has arguments
    let env = env();
    env.exec(
        r#"
        TestErrors = {}
        seterrorhandler(function(msg) table.insert(TestErrors, msg) end)

        local f = CreateFrame("Frame")
        f:RegisterEvent("ADDON_LOADED")
        f:SetScript("OnEvent", function(self, event, addonName)
            error("failed loading " .. tostring(addonName))
        end)
        "#,
    )
    .unwrap();

    let lua = env.lua();
    let addon_name = mlua::Value::String(lua.create_string("MyAddon").unwrap());
    env.fire_event_with_args("ADDON_LOADED", &[addon_name]).ok();

    let (count, msg): (i32, String) = env
        .eval("return #TestErrors, TestErrors[1] or ''")
        .unwrap();
    assert_eq!(count, 1);
    assert!(
        msg.contains("failed loading MyAddon"),
        "error message was: {msg}"
    );
}

// ── OnUpdate errors route through error handler ──────────────────────

#[test]
fn test_error_handler_receives_onupdate_errors() {
    let env = env();
    env.exec(
        r#"
        UpdateErrors = {}
        seterrorhandler(function(msg) table.insert(UpdateErrors, msg) end)

        local f = CreateFrame("Frame")
        f:SetScript("OnUpdate", function(self, elapsed)
            error("tick failed at " .. tostring(elapsed))
        end)
        "#,
    )
    .unwrap();

    // Fire OnUpdate from Rust
    env.fire_on_update(0.016).ok();

    let (count, msg): (i32, String) = env
        .eval("return #UpdateErrors, UpdateErrors[1] or ''")
        .unwrap();
    assert_eq!(count, 1, "error handler should have received exactly 1 error");
    assert!(
        msg.contains("tick failed"),
        "error message was: {msg}"
    );
}

// ── Multiple errors collected ────────────────────────────────────────

#[test]
fn test_error_handler_collects_multiple_errors() {
    // Two different frames error on the same event
    let env = env();
    env.exec(
        r#"
        AllErrors = {}
        seterrorhandler(function(msg) table.insert(AllErrors, msg) end)

        local f1 = CreateFrame("Frame")
        f1:RegisterEvent("PLAYER_LOGIN")
        f1:SetScript("OnEvent", function() error("error from frame 1") end)

        local f2 = CreateFrame("Frame")
        f2:RegisterEvent("PLAYER_LOGIN")
        f2:SetScript("OnEvent", function() error("error from frame 2") end)
        "#,
    )
    .unwrap();

    env.fire_event("PLAYER_LOGIN").ok();

    let count: i32 = env.eval("return #AllErrors").unwrap();
    assert_eq!(count, 2, "both errors should have been collected");
}

// ── Scripts still work when no errors occur ──────────────────────────

#[test]
fn test_scripts_work_despite_hidden_tables() {
    let env = env();
    let called: bool = env
        .eval(
            r#"
            SCRIPT_TEST_CALLED = false
            local f = CreateFrame("Frame")
            f:SetScript("OnShow", function() SCRIPT_TEST_CALLED = true end)
            f:Hide()
            f:Show()
            return SCRIPT_TEST_CALLED
            "#,
        )
        .unwrap();
    assert!(called);
}

#[test]
fn test_event_dispatch_works_without_error_handler() {
    // When no error handler is set, events still fire normally
    let env = env();
    env.exec(
        r#"
        EVENT_RECEIVED = false
        local f = CreateFrame("Frame")
        f:RegisterEvent("PLAYER_LOGIN")
        f:SetScript("OnEvent", function() EVENT_RECEIVED = true end)
        "#,
    )
    .unwrap();

    env.fire_event("PLAYER_LOGIN").ok();

    let received: bool = env.eval("return EVENT_RECEIVED").unwrap();
    assert!(received);
}

// ── Error handler replacement ────────────────────────────────────────

#[test]
fn test_error_handler_can_be_replaced() {
    // BugSack pattern: addon replaces the default error handler
    let env = env();
    env.exec(
        r#"
        FirstHandlerCalls = 0
        SecondHandlerCalls = 0
        seterrorhandler(function() FirstHandlerCalls = FirstHandlerCalls + 1 end)
        seterrorhandler(function() SecondHandlerCalls = SecondHandlerCalls + 1 end)

        local f = CreateFrame("Frame")
        f:RegisterEvent("PLAYER_LOGIN")
        f:SetScript("OnEvent", function() error("boom") end)
        "#,
    )
    .unwrap();

    env.fire_event("PLAYER_LOGIN").ok();

    let (first, second): (i32, i32) = env
        .eval("return FirstHandlerCalls, SecondHandlerCalls")
        .unwrap();
    assert_eq!(first, 0, "first handler should not be called after replacement");
    assert_eq!(second, 1, "second handler should receive the error");
}
