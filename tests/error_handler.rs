//! Tests for WoW error handler system and internal table visibility.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

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
fn test_error_handler_receives_script_errors() {
    let env = env();
    let (count, msg): (i32, String) = env
        .eval(
            r#"
            local errs = {}
            seterrorhandler(function(msg) table.insert(errs, msg) end)
            local f = CreateFrame("Frame")
            f:SetScript("OnShow", function() error("deliberate test error") end)
            f:Hide()
            f:Show()
            return #errs, errs[1] or ""
            "#,
        )
        .unwrap();
    assert_eq!(count, 1);
    assert!(
        msg.contains("deliberate test error"),
        "error message was: {msg}"
    );
}

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
