//! Tests for system_api.rs: type(), rawget(), xpcall(), BN*, build type stubs.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// type() override (frame userdata awareness)
// ============================================================================

#[test]
fn test_type_number() {
    let env = env();
    let t: String = env.eval("return type(42)").unwrap();
    assert_eq!(t, "number");
}

#[test]
fn test_type_string() {
    let env = env();
    let t: String = env.eval("return type('hello')").unwrap();
    assert_eq!(t, "string");
}

#[test]
fn test_type_table() {
    let env = env();
    let t: String = env.eval("return type({})").unwrap();
    assert_eq!(t, "table");
}

#[test]
fn test_type_nil() {
    let env = env();
    let t: String = env.eval("return type(nil)").unwrap();
    assert_eq!(t, "nil");
}

#[test]
fn test_type_boolean() {
    let env = env();
    let t: String = env.eval("return type(true)").unwrap();
    assert_eq!(t, "boolean");
}

#[test]
fn test_type_function() {
    let env = env();
    let t: String = env.eval("return type(print)").unwrap();
    assert_eq!(t, "function");
}

#[test]
fn test_type_frame_returns_table() {
    let env = env();
    let t: String = env.eval(r#"
        local f = CreateFrame("Frame", "TestTypeFrame", UIParent)
        return type(f)
    "#).unwrap();
    assert_eq!(t, "table", "type() on frame userdata should return 'table'");
}

// ============================================================================
// rawget()
// ============================================================================

#[test]
fn test_rawget_table() {
    let env = env();
    let val: i32 = env.eval(r#"
        local t = {a = 42}
        return rawget(t, "a")
    "#).unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_rawget_bypasses_metatable() {
    let env = env();
    let is_nil: bool = env.eval(r#"
        local t = setmetatable({}, {__index = function() return 99 end})
        return rawget(t, "missing") == nil
    "#).unwrap();
    assert!(is_nil, "rawget should bypass metatable __index");
}

// ============================================================================
// xpcall()
// ============================================================================

#[test]
fn test_xpcall_success() {
    let env = env();
    let (ok, result): (bool, i32) = env.eval(r#"
        return xpcall(function() return 42 end, function(err) return err end)
    "#).unwrap();
    assert!(ok);
    assert_eq!(result, 42);
}

#[test]
fn test_xpcall_error() {
    let env = env();
    let (ok, msg): (bool, String) = env.eval(r#"
        return xpcall(function() error("boom") end, function(err) return "handled: " .. err end)
    "#).unwrap();
    assert!(!ok);
    assert!(msg.contains("handled:"), "Error handler should be called");
}

#[test]
fn test_xpcall_passes_args() {
    let env = env();
    let (ok, result): (bool, i32) = env.eval(r#"
        return xpcall(function(a, b) return a + b end, function(err) return err end, 10, 20)
    "#).unwrap();
    assert!(ok);
    assert_eq!(result, 30);
}

// ============================================================================
// SlashCmdList
// ============================================================================

#[test]
fn test_slash_cmd_list_is_table() {
    let env = env();
    let is_table: bool = env.eval("return type(SlashCmdList) == 'table'").unwrap();
    assert!(is_table);
}

// ============================================================================
// Build type stubs
// ============================================================================

#[test]
fn test_is_public_test_client() {
    let env = env();
    let val: bool = env.eval("return IsPublicTestClient()").unwrap();
    assert!(!val);
}

#[test]
fn test_is_beta_build() {
    let env = env();
    let val: bool = env.eval("return IsBetaBuild()").unwrap();
    assert!(!val);
}

#[test]
fn test_is_public_build() {
    let env = env();
    let val: bool = env.eval("return IsPublicBuild()").unwrap();
    assert!(val);
}

// ============================================================================
// Battle.net stubs
// ============================================================================

#[test]
fn test_bn_features_enabled() {
    let env = env();
    let val: bool = env.eval("return BNFeaturesEnabled()").unwrap();
    assert!(!val);
}

#[test]
fn test_bn_features_enabled_and_connected() {
    let env = env();
    let val: bool = env.eval("return BNFeaturesEnabledAndConnected()").unwrap();
    assert!(!val);
}

#[test]
fn test_bn_connected() {
    let env = env();
    let val: bool = env.eval("return BNConnected()").unwrap();
    assert!(!val);
}

#[test]
fn test_bn_get_num_friends() {
    let env = env();
    let (total, online): (i32, i32) = env.eval("return BNGetNumFriends()").unwrap();
    assert_eq!(total, 0);
    assert_eq!(online, 0);
}

#[test]
fn test_bn_get_friend_info_nil() {
    let env = env();
    let is_nil: bool = env.eval("return BNGetFriendInfo(1) == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn test_bn_get_info() {
    let env = env();
    // BNGetInfo returns mock data, just verify it doesn't error
    env.exec("local info = BNGetInfo()").unwrap();
}

// ============================================================================
// FireEvent / ReloadUI
// ============================================================================

#[test]
fn test_fire_event_no_error() {
    let env = env();
    env.exec(r#"FireEvent("PLAYER_ENTERING_WORLD")"#).unwrap();
}

#[test]
fn test_reload_ui_exists() {
    let env = env();
    let is_func: bool = env.eval("return type(ReloadUI) == 'function'").unwrap();
    assert!(is_func);
}
