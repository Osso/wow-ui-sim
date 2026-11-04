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
    let t: String = env
        .eval(
            r#"
        local f = CreateFrame("Frame", "TestTypeFrame", UIParent)
        return type(f)
    "#,
        )
        .unwrap();
    assert_eq!(t, "table", "type() on frame userdata should return 'table'");
}

// ============================================================================
// rawget()
// ============================================================================

#[test]
fn test_rawget_table() {
    let env = env();
    let val: i32 = env
        .eval(
            r#"
        local t = {a = 42}
        return rawget(t, "a")
    "#,
        )
        .unwrap();
    assert_eq!(val, 42);
}

#[test]
fn test_rawget_bypasses_metatable() {
    let env = env();
    let is_nil: bool = env
        .eval(
            r#"
        local t = setmetatable({}, {__index = function() return 99 end})
        return rawget(t, "missing") == nil
    "#,
        )
        .unwrap();
    assert!(is_nil, "rawget should bypass metatable __index");
}

// ============================================================================
// xpcall()
// ============================================================================

#[test]
fn test_xpcall_success() {
    let env = env();
    let (ok, result): (bool, i32) = env
        .eval(
            r#"
        return xpcall(function() return 42 end, function(err) return err end)
    "#,
        )
        .unwrap();
    assert!(ok);
    assert_eq!(result, 42);
}

#[test]
fn test_xpcall_error() {
    let env = env();
    let (ok, msg): (bool, String) = env
        .eval(
            r#"
        return xpcall(function() error("boom") end, function(err) return "handled: " .. err end)
    "#,
        )
        .unwrap();
    assert!(!ok);
    assert!(msg.contains("handled:"), "Error handler should be called");
}

#[test]
fn test_xpcall_passes_args() {
    let env = env();
    let (ok, result): (bool, i32) = env
        .eval(
            r#"
        return xpcall(function(a, b) return a + b end, function(err) return err end, 10, 20)
    "#,
        )
        .unwrap();
    assert!(ok);
    assert_eq!(result, 30);
}

#[test]
fn test_xpcall_error_handler_sees_failing_lua_call_path() {
    let env = env();
    let (ok, error): (bool, String) = env
        .eval(
            r#"
            return xpcall(function()
                local function inner()
                    error("traceback boom")
                end
                inner()
            end, debug.traceback)
            "#,
        )
        .unwrap();

    assert!(!ok);
    assert!(
        error.contains("inner"),
        "traceback should include the failing Lua call path, got: {error}"
    );
}

#[test]
fn test_protected_calls_do_not_record_handled_errors() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local p_ok, p_err = pcall(function() error("repeat me") end)
        local x_ok, x_err = xpcall(
            function() error("repeat me") end,
            function(err) return "handled: " .. err end
        )
        if p_ok or x_ok then
            return "unexpected_success"
        end
        return p_err .. " | " .. x_err
    "#,
        )
        .unwrap();

    assert!(
        result.contains("repeat me"),
        "protected calls should still surface the original error to Lua"
    );

    let state = env.state().borrow();
    assert!(
        state.lua_error_counts.is_empty(),
        "handled protected-call errors should not enter lua_error_counts: {:?}",
        state.lua_error_counts
    );
}

#[test]
fn test_xpcall_handler_failure_returns_original_error_without_recording() {
    let env = env();
    let (_ok, msg): (bool, String) = env
        .eval(
            r#"
        return xpcall(
            function() error("root failure") end,
            function(err) error("handler failure: " .. err) end
        )
    "#,
        )
        .unwrap();

    assert!(
        msg.contains("root failure"),
        "xpcall should return original error when handler also fails: {msg}"
    );

    let state = env.state().borrow();
    assert!(
        state.lua_error_counts.is_empty(),
        "handled xpcall failures should not enter lua_error_counts: {:?}",
        state.lua_error_counts
    );
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

#[test]
fn test_get_net_stats_returns_seeded_latency_values() {
    let env = env();
    env.exec("A_Admin.SetNetStats(128, 64, 47, 89)").unwrap();
    let (bandwidth_in, bandwidth_out, latency_home, latency_world): (f64, f64, f64, f64) =
        env.eval("return GetNetStats()").unwrap();
    assert!(
        bandwidth_in > 0.0,
        "GetNetStats should seed incoming bandwidth for network stats UI"
    );
    assert!(
        bandwidth_out > 0.0,
        "GetNetStats should seed outgoing bandwidth for network stats UI"
    );
    assert!(
        latency_home > 0.0,
        "GetNetStats should seed home latency for network stats UI"
    );
    assert!(
        latency_world > 0.0,
        "GetNetStats should seed world latency for network stats UI"
    );
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

#[test]
fn test_reload_ui_defers_entering_world_until_request_is_serviced() {
    let env = env();
    let received: bool = env
        .eval(
            r#"
            local received = false
            local frame = CreateFrame("Frame")
            frame:RegisterEvent("PLAYER_ENTERING_WORLD")
            frame:SetScript("OnEvent", function(_, event)
                if event == "PLAYER_ENTERING_WORLD" then received = true end
            end)
            ReloadUI()
            return received
            "#,
        )
        .unwrap();

    assert!(
        !received,
        "the old VM must not receive a synchronous reload event"
    );
    assert!(env.state().borrow().ui_reload_requested);
}

// ============================================================================
// Generated stub overrides
// ============================================================================

#[test]
fn test_ambiguate_preserves_plain_names() {
    let env = env();
    let value: String = env.eval(r#"return Ambiguate("Thrall", "all")"#).unwrap();
    assert_eq!(value, "Thrall");
}

#[test]
fn test_ambiguate_strips_realm_for_common_contexts() {
    let env = env();
    let (all_name, short_name, guild_name): (String, String, String) = env
        .eval(
            r#"
        return Ambiguate("Thrall-Area52", "all"),
               Ambiguate("Thrall-Area52", "short"),
               Ambiguate("Thrall-Area52", "guild")
    "#,
        )
        .unwrap();
    assert_eq!(all_name, "Thrall");
    assert_eq!(short_name, "Thrall");
    assert_eq!(guild_name, "Thrall");
}

#[test]
fn test_ambiguate_none_keeps_full_name() {
    let env = env();
    let value: String = env
        .eval(r#"return Ambiguate("Thrall-Area52", "none")"#)
        .unwrap();
    assert_eq!(value, "Thrall-Area52");
}

#[test]
fn test_are_talents_locked_returns_false() {
    let env = env();
    let value: bool = env.eval("return AreTalentsLocked()").unwrap();
    assert!(!value);
}
