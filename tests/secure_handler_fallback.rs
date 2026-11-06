//! `SecureHandlerSetFrameRef` / `SecureHandlerExecute` / `SecureHandlerWrapScript`
//! Lua fallback contract — see `register_secure_handler_stubs` in
//! `src/lua_api/globals/security.rs`.
//!
//! The fallback is the sim's minimal stand-in before
//! `Blizzard_RestrictedAddOnEnvironment` loads the full retail
//! implementation. These tests run against a fresh `WowLuaEnv`, so
//! Blizzard_RestrictedAddOnEnvironment is *not* loaded and the
//! fallback is active.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn set_frame_ref_round_trips_via_get_frame_ref() {
    let env = env();
    let lookup: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local ref = CreateFrame("Frame", nil, UIParent)
            SecureHandlerSetFrameRef(frame, "MyRef", ref)
            return SecureHandlerGetFrameRef(frame, "MyRef") == ref
            "#,
        )
        .unwrap();
    assert!(
        lookup,
        "SetFrameRef must stash the ref so GetFrameRef retrieves it"
    );
}

#[test]
fn set_frame_ref_scopes_per_frame() {
    let env = env();
    let isolated: bool = env
        .eval(
            r#"
            local a = CreateFrame("Frame", nil, UIParent)
            local b = CreateFrame("Frame", nil, UIParent)
            local ref = CreateFrame("Frame", nil, UIParent)
            SecureHandlerSetFrameRef(a, "Shared", ref)
            -- b never had Shared set -> nil
            return SecureHandlerGetFrameRef(b, "Shared") == nil
                and SecureHandlerGetFrameRef(a, "Shared") == ref
            "#,
        )
        .unwrap();
    assert!(isolated);
}

#[test]
fn set_frame_ref_overwrites_same_label() {
    let env = env();
    let overwrote: bool = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            local first = CreateFrame("Frame", nil, UIParent)
            local second = CreateFrame("Frame", nil, UIParent)
            SecureHandlerSetFrameRef(owner, "Label", first)
            SecureHandlerSetFrameRef(owner, "Label", second)
            return SecureHandlerGetFrameRef(owner, "Label") == second
            "#,
        )
        .unwrap();
    assert!(overwrote);
}

#[test]
fn set_frame_ref_is_noop_with_nil_args() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            -- Must not throw even when every arg is nil / wrong-typed.
            SecureHandlerSetFrameRef(nil, "MyRef", nil)
            SecureHandlerSetFrameRef(CreateFrame("Frame", nil, UIParent), 42, nil)
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn execute_runs_body_with_self_bound_to_frame() {
    let env = env();
    let captured: String = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", "SecureExecuteOwner", UIParent)
            SecureHandlerExecute(owner, "self:SetAttribute('capturedName', self:GetName())")
            return owner:GetAttribute('capturedName') or ""
            "#,
        )
        .unwrap();
    assert_eq!(captured, "SecureExecuteOwner");
}

#[test]
fn execute_passes_varargs_to_body() {
    let env = env();
    let sum: f64 = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            SecureHandlerExecute(owner, "local a, b = ...; self:SetAttribute('sum', (a or 0) + (b or 0))", 3, 4)
            return owner:GetAttribute('sum')
            "#,
        )
        .unwrap();
    assert_eq!(sum, 7.0);
}

#[test]
fn execute_uses_restricted_environment_without_global_table_access() {
    let env = env();
    let leaked: String = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            _G.__secure_leak = "clean"
            SecureHandlerExecute(owner, "_G.__secure_leak = 'leaked'; self:SetAttribute('afterLeak', true)")
            return _G.__secure_leak .. ":" .. tostring(owner:GetAttribute('afterLeak'))
            "#,
        )
        .unwrap();
    assert_eq!(leaked, "clean:nil");
}

#[test]
fn execute_restricted_environment_allows_math_string_and_print() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            SecureHandlerExecute(owner, "local text = string.upper('ok') .. ':' .. math.max(2, 5); print(text); self:SetAttribute('allowed', text)")
            return owner:GetAttribute('allowed') or ""
            "#,
        )
        .unwrap();
    assert_eq!(result, "OK:5");
}

#[test]
fn execute_restricted_environment_allows_table_maxn_for_click_cast_loops() {
    let env = env();
    let applied: String = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            SecureHandlerExecute(owner, [[
                local keybinds = {}
                local count = 0
                for i = 1, table.maxn(keybinds) do
                    count = count + 1
                end
                self:SetAttribute("loopCount", count)
            ]])
            return tostring(owner:GetAttribute("loopCount"))
            "#,
        )
        .unwrap();
    assert_eq!(applied, "0");
}

classic_secret_values_fixture! {
#[test]
fn execute_callmethod_propagates_tainted_args_to_insecure_method() {
    let env = env();
    let (called, returned_is_secret): (bool, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            function owner:Echo(value)
                return value
            end

            local payload = "secret-unit-identity"
            debug.settaintmode("rw")
            debug.setstacktaint("evil-addon")
            SecureHandlerExecute(owner, [[
                local returned = self:CallMethod('Echo', ...)
                self:SetAttribute('callmethodReturned', returned)
            ]], payload)
            debug.setstacktaint(nil)

            local returned = owner:GetAttribute('callmethodReturned')
            return returned == payload, issecretvalue(returned)
            "#,
        )
        .unwrap();
    assert!(called);
    assert!(returned_is_secret);
}
}

#[test]
fn execute_callmethod_does_not_poison_table_args_or_results() {
    let env = env();
    let (called, returned_is_secret): (bool, bool) = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            function owner:Echo(value)
                return value
            end

            local payload = {}
            debug.settaintmode("rw")
            debug.setstacktaint("evil-addon")
            SecureHandlerExecute(owner, [[
                local returned = self:CallMethod('Echo', ...)
                self:SetAttribute('callmethodReturned', returned)
            ]], payload)
            debug.setstacktaint(nil)

            local returned = owner:GetAttribute('callmethodReturned')
            return returned == payload, issecretvalue(returned)
            "#,
        )
        .unwrap();
    assert!(called);
    assert!(!returned_is_secret);
}

#[test]
fn execute_swallows_compile_and_runtime_errors() {
    let env = env();
    let still_true: bool = env
        .eval(
            r#"
            local owner = CreateFrame("Frame", nil, UIParent)
            -- Syntax error: should not bubble.
            SecureHandlerExecute(owner, "return @@@@@")
            -- Runtime error: should not bubble.
            SecureHandlerExecute(owner, "error('nope')")
            return true
            "#,
        )
        .unwrap();
    assert!(still_true);
}

#[test]
fn wrap_script_installs_pre_and_post_around_existing_handler() {
    let env = env();
    let order: String = env
        .eval(
            r#"
            local frame = CreateFrame("Button", "WrapScriptBtn", UIParent)
            local header = CreateFrame("Frame", "WrapScriptHeader", UIParent)
            local function record(tag)
                header:SetAttribute('log', (header:GetAttribute('log') or '') .. tag .. '|')
            end
            -- Original: records "mid"
            frame:SetScript("OnClick", function() record("mid") end)
            SecureHandlerWrapScript(frame, "OnClick", header,
                "self:SetAttribute('log', (self:GetAttribute('log') or '') .. 'pre:' .. self:GetName() .. '|')",
                "self:SetAttribute('log', (self:GetAttribute('log') or '') .. 'post:' .. self:GetName() .. '|')"
            )
            frame:GetScript("OnClick")(frame)
            return header:GetAttribute('log') or ''
            "#,
        )
        .unwrap();
    assert_eq!(
        order, "pre:WrapScriptHeader|mid|post:WrapScriptHeader|",
        "wrapped handler should run pre(header)/original(frame)/post(header)"
    );
}

#[test]
fn wrap_script_runs_even_without_existing_handler() {
    let env = env();
    let fired: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Button", nil, UIParent)
            local header = CreateFrame("Frame", nil, UIParent)
            SecureHandlerWrapScript(frame, "OnClick", header,
                "self:SetAttribute('fired', true)",
                nil
            )
            frame:GetScript("OnClick")(frame)
            return header:GetAttribute('fired') == true
            "#,
        )
        .unwrap();
    assert!(fired);
}

#[test]
fn wrap_script_post_body_is_optional() {
    let env = env();
    let pre_only: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Button", nil, UIParent)
            local header = CreateFrame("Frame", nil, UIParent)
            SecureHandlerWrapScript(frame, "OnClick", header,
                "self:SetAttribute('preCalled', true)"
                -- no postBody
            )
            frame:GetScript("OnClick")(frame)
            return header:GetAttribute('preCalled') == true
            "#,
        )
        .unwrap();
    assert!(pre_only);
}

#[test]
fn wrap_script_isolates_bad_snippets_from_original() {
    let env = env();
    let original_ran: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Button", nil, UIParent)
            local header = CreateFrame("Frame", nil, UIParent)
            _G.__original_ran = false
            frame:SetScript("OnClick", function() _G.__original_ran = true end)
            SecureHandlerWrapScript(frame, "OnClick", header,
                "error('bad pre')",
                "error('bad post')"
            )
            frame:GetScript("OnClick")(frame)
            return _G.__original_ran
            "#,
        )
        .unwrap();
    assert!(
        original_ran,
        "A failing pre or post snippet must not prevent the original handler from firing"
    );
}
