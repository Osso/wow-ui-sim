//! Tests for globals_legacy.rs: print, ipairs, getmetatable overrides.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// print override
// ============================================================================

#[test]
fn test_print_nil() {
    let env = env();
    env.exec("print(nil)").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "nil");
}

#[test]
fn test_print_boolean() {
    let env = env();
    env.exec("print(true, false)").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "true\tfalse");
}

#[test]
fn test_print_numbers() {
    let env = env();
    env.exec("print(42, 3.14)").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "42\t3.14");
}

#[test]
fn test_print_string() {
    let env = env();
    env.exec("print('hello world')").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "hello world");
}

#[test]
fn test_print_table() {
    let env = env();
    env.exec("print({})").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "table");
}

#[test]
fn test_print_function() {
    let env = env();
    env.exec("print(print)").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "function");
}

#[test]
fn test_print_mixed_args_tab_separated() {
    let env = env();
    env.exec("print(1, 'two', true, nil)").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "1\ttwo\ttrue\tnil");
}

#[test]
fn test_print_no_args() {
    let env = env();
    env.exec("print()").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.last().unwrap(), "");
}

#[test]
fn test_print_accumulates_in_console_buffer() {
    let env = env();
    env.exec("print('first')").unwrap();
    env.exec("print('second')").unwrap();
    let output = &env.state().borrow().console_output;
    assert_eq!(output.len(), 2);
    assert_eq!(output[0], "first");
    assert_eq!(output[1], "second");
}

// ============================================================================
// ipairs override (with frame userdata support)
// ============================================================================

#[test]
fn test_ipairs_table_still_works() {
    let env = env();
    let total: i32 = env
        .eval(
            r#"
            local sum = 0
            for i, v in ipairs({10, 20, 30}) do
                sum = sum + v
            end
            return sum
            "#,
        )
        .unwrap();
    assert_eq!(total, 60);
}

#[test]
fn test_ipairs_empty_table() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local n = 0
            for _ in ipairs({}) do n = n + 1 end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_ipairs_frame_children() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "TestIpairsParent")
            CreateFrame("Frame", "TestIpairsChild1", parent)
            CreateFrame("Frame", "TestIpairsChild2", parent)
            local n = 0
            for i, child in ipairs(parent) do
                n = n + 1
            end
            return n
            "#,
        )
        .unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_ipairs_frame_children_index_starts_at_one() {
    let env = env();
    let first_idx: i32 = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "TestIpairsIdx")
            CreateFrame("Frame", "TestIpairsIdxChild", parent)
            local idx
            for i, child in ipairs(parent) do
                idx = i
                break
            end
            return idx
            "#,
        )
        .unwrap();
    assert_eq!(first_idx, 1);
}

// ============================================================================
// getmetatable override (frame userdata metatable)
// ============================================================================

#[test]
fn test_getmetatable_table_works() {
    let env = env();
    let has_mt: bool = env
        .eval(
            r#"
            local t = setmetatable({}, {__index = function() return 42 end})
            return getmetatable(t) ~= nil
            "#,
        )
        .unwrap();
    assert!(has_mt);
}

#[test]
fn test_getmetatable_frame_returns_table() {
    let env = env();
    let is_table: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestGetMTFrame")
            local mt = getmetatable(f)
            return type(mt) == "table"
            "#,
        )
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_getmetatable_frame_has_index_table() {
    let env = env();
    let is_table: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestGetMTIndex")
            local mt = getmetatable(f)
            return type(mt.__index) == "table"
            "#,
        )
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_getmetatable_frame_index_has_methods() {
    let env = env();
    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestGetMTMethods")
            local mt = getmetatable(f)
            local idx = mt.__index
            return type(idx.GetName) == "function",
                   type(idx.Show) == "function",
                   type(idx.SetPoint) == "function"
            "#,
        )
        .unwrap();
    assert!(result.0, "GetName should be a function");
    assert!(result.1, "Show should be a function");
    assert!(result.2, "SetPoint should be a function");
}

#[test]
fn test_getmetatable_frame_index_iterable_with_pairs() {
    let env = env();
    let count: i32 = env
        .eval(
            r#"
            local f = CreateFrame("Frame", "TestGetMTIterable")
            local mt = getmetatable(f)
            local n = 0
            for name, func in pairs(mt.__index) do
                n = n + 1
            end
            return n
            "#,
        )
        .unwrap();
    // Should have many methods
    assert!(count > 50, "Expected many methods in __index, got {}", count);
}

#[test]
fn test_getmetatable_nil_returns_nil() {
    let env = env();
    let is_nil: bool = env.eval("return getmetatable(nil) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_getmetatable_string_has_metatable() {
    let env = env();
    // Lua strings have a metatable with __index = string library
    let is_table: bool = env
        .eval("return type(getmetatable('')) == 'table'")
        .unwrap();
    assert!(is_table);
}

// ============================================================================
// CreateFrame exists and works (delegated to sub-module)
// ============================================================================

#[test]
fn test_create_frame_exists() {
    let env = env();
    let is_func: bool = env
        .eval("return type(CreateFrame) == 'function'")
        .unwrap();
    assert!(is_func);
}

// ============================================================================
// Sub-module registrations are in place
// ============================================================================

#[test]
fn test_submodule_apis_registered() {
    let env = env();
    // Spot-check a few functions/namespaces from sub-modules
    for name in &[
        "GetLocale",       // locale_api
        "GetNumAddOns",    // addon_api
        "UnitName",        // unit_api
        "Mixin",           // mixin_api
        "strsplit",        // utility_api
        "GetCVar",         // cvar_api
        "CreateFont",      // font_api
    ] {
        let is_func: bool = env
            .eval(&format!("return type({}) == 'function'", name))
            .unwrap();
        assert!(is_func, "{} should be a function", name);
    }
}

#[test]
fn test_submodule_namespaces_registered() {
    let env = env();
    for name in &[
        "C_Timer",
        "C_Map",
        "C_QuestLog",
        "C_MountJournal",
        "C_Item",
        "Enum",
        "Settings",
    ] {
        let is_table: bool = env
            .eval(&format!("return type({}) == 'table'", name))
            .unwrap();
        assert!(is_table, "{} should be a table", name);
    }
}

// ============================================================================
// UI strings registered
// ============================================================================

#[test]
fn test_ui_strings_registered() {
    let env = env();
    // Some well-known UI string constants
    let is_string: bool = env
        .eval("return type(OKAY) == 'string'")
        .unwrap();
    assert!(is_string);
}

// ============================================================================
// Standard font objects created
// ============================================================================

#[test]
fn test_standard_font_objects_created() {
    let env = env();
    let exists: bool = env
        .eval("return GameFontNormal ~= nil")
        .unwrap();
    assert!(exists, "GameFontNormal should exist");
}
