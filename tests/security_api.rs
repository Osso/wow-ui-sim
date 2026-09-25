//! Tests for security API functions (security_api.rs) and Elune taint tracking.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SecureCmdOptionParse
// ============================================================================

#[test]
fn test_securecmdoptionparse_uses_fallback_when_condition_is_false() {
    let env = env();
    let result: String = env
        .eval("return SecureCmdOptionParse('[mod:shift] action1; action2')")
        .unwrap();
    assert_eq!(result, "action2");
}

#[test]
fn test_securecmdoptionparse_matches_modifier_and_comma_conditions() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetShiftKeyDown(true)
        A_Admin.SetTarget("Friendly", 70, 2, false)
    "#,
    )
    .unwrap();
    let result: String = env
        .eval("return SecureCmdOptionParse('[mod:shift,noharm] Flash Heal; [combat] Crusader Strike; fallback')")
        .unwrap();
    assert_eq!(result, "Flash Heal");
}

#[test]
fn test_securecmdoptionparse_uses_first_matching_combat_clause() {
    let env = env();
    env.exec("A_Admin.SetInCombat(true)").unwrap();
    let result: String = env
        .eval("return SecureCmdOptionParse('[mod:shift] shifted; [combat] combat_spell; fallback')")
        .unwrap();
    assert_eq!(result, "combat_spell");
}

#[test]
fn test_securecmdoptionparse_supports_harm_and_unit_override() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetTarget("Friendly", 70, 2, false)
        A_Admin.SetFocus("Enemy", 70, 1, true)
    "#,
    )
    .unwrap();
    let result: String = env
        .eval("return SecureCmdOptionParse('[@target,harm] target_harm; [@focus,harm] focus_harm; fallback')")
        .unwrap();
    assert_eq!(result, "focus_harm");
}

#[test]
fn test_securecmdoptionparse_returns_nil_when_no_clause_matches() {
    let env = env();
    let is_nil: bool = env
        .eval("return SecureCmdOptionParse('[combat] combat_only; [mod:alt] alt_only') == nil")
        .unwrap();
    assert!(is_nil);
}

#[test]
fn test_securecmdoptionparse_single_option() {
    let env = env();
    let result: String = env
        .eval("return SecureCmdOptionParse('just_this')")
        .unwrap();
    assert_eq!(result, "just_this");
}

// ============================================================================
// hooksecurefunc
// ============================================================================

#[test]
fn test_hooksecurefunc_global() {
    let env = env();
    let (orig_ran, hook_ran): (bool, bool) = env
        .eval(
            r#"
            HOOK_TEST_ORIG = false
            HOOK_TEST_HOOK = false
            function MyTestFunc() HOOK_TEST_ORIG = true end
            hooksecurefunc("MyTestFunc", function() HOOK_TEST_HOOK = true end)
            MyTestFunc()
            return HOOK_TEST_ORIG, HOOK_TEST_HOOK
            "#,
        )
        .unwrap();
    assert!(orig_ran);
    assert!(hook_ran);
}

#[test]
fn test_hooksecurefunc_table() {
    let env = env();
    let (orig_ran, hook_ran): (bool, bool) = env
        .eval(
            r#"
            local t = {}
            HOOK_TABLE_ORIG = false
            HOOK_TABLE_HOOK = false
            function t.Foo() HOOK_TABLE_ORIG = true end
            hooksecurefunc(t, "Foo", function() HOOK_TABLE_HOOK = true end)
            t.Foo()
            return HOOK_TABLE_ORIG, HOOK_TABLE_HOOK
            "#,
        )
        .unwrap();
    assert!(orig_ran);
    assert!(hook_ran);
}

// ============================================================================
// securecall / securecallfunction
// ============================================================================

#[test]
fn test_securecall() {
    let env = env();
    let result: i32 = env
        .eval("return securecall(function(a) return a * 2 end, 5)")
        .unwrap();
    assert_eq!(result, 10);
}

#[test]
fn test_securecall_accepts_global_function_name() {
    let env = env();
    let result: i32 = env
        .eval(
            r#"
            function SecureCallNameProbe(a, b)
                return a + b
            end
            return securecall("SecureCallNameProbe", 3, 4)
        "#,
        )
        .unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_securecallfunction() {
    let env = env();
    let result: i32 = env
        .eval("return securecallfunction(function(a, b) return a + b end, 3, 4)")
        .unwrap();
    assert_eq!(result, 7);
}

#[test]
fn test_securecallfunction_passes_table_self_to_unbound_method() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local mixin = {}
            function mixin:GetName()
                return self.name
            end
            local object = { name = "category" }
            return securecallfunction(mixin.GetName, object)
            "#,
        )
        .unwrap();
    assert_eq!(result, "category");
}

// ============================================================================
// secureexecuterange
// ============================================================================

#[test]
fn test_secureexecuterange() {
    let env = env();
    let total: i32 = env
        .eval(
            r#"
            SECURE_TOTAL = 0
            local t = {10, 20, 30}
            secureexecuterange(t, function(key, value) SECURE_TOTAL = SECURE_TOTAL + value end)
            return SECURE_TOTAL
            "#,
        )
        .unwrap();
    assert_eq!(total, 60);
}

// ============================================================================
// issecure / issecurevariable
// ============================================================================

#[test]
fn test_issecure_returns_true() {
    let env = env();
    let val: bool = env.eval("return issecure()").unwrap();
    assert!(val);
}

#[test]
fn test_issecurevariable_returns_true() {
    let env = env();
    // Elune's issecurevariable takes ("variable") or (table, "variable"), not nil
    let val: bool = env
        .eval("local s, t = issecurevariable('print'); return s")
        .unwrap();
    assert!(val);
}

#[test]
fn addon_tainted_global_helper_taints_blizzard_call_path() {
    let env = env();
    let (
        slot_secure,
        slot_owner,
        before_call_secure,
        callee_secure,
        callee_taint,
        after_call_secure,
    ): (bool, String, bool, bool, String, bool) = env
        .eval(
            r#"
            local function addonWriteGlobalHelper()
                debug.setstacktaint("TaintProbeAddon")
                CooldownFrame_Set = function()
                    return issecure(), debug.getstacktaint()
                end
                local secure, owner = issecurevariable(_G, "CooldownFrame_Set")
                debug.setstacktaint(nil)
                return secure, owner
            end

            local slotSecure, slotOwner = addonWriteGlobalHelper()
            local beforeCallSecure = issecure()
            local calleeSecure, calleeTaint = CooldownFrame_Set()
            local afterCallSecure = issecure()

            return slotSecure, slotOwner, beforeCallSecure, calleeSecure, calleeTaint or "", afterCallSecure
            "#,
        )
        .unwrap();

    assert!(
        !slot_secure,
        "addon writes to _G must taint the global slot"
    );
    assert_eq!(slot_owner, "TaintProbeAddon");
    assert!(
        before_call_secure,
        "Blizzard-style caller starts from a secure stack"
    );
    assert!(
        !callee_secure,
        "calling through the tainted global helper must run insecure"
    );
    assert_eq!(callee_taint, "TaintProbeAddon");
    assert!(
        !after_call_secure,
        "reading/calling a tainted global must taint the caller's execution path"
    );
}

// ============================================================================
// SecureHandler stubs
// ============================================================================

#[test]
fn test_secure_handler_stubs_exist() {
    let env = env();
    for func in &[
        "SecureHandlerSetFrameRef",
        "SecureHandlerExecute",
        "SecureHandlerWrapScript",
        "SecureHandlerUnwrapScript",
        "RegisterStateDriver",
        "UnregisterStateDriver",
        "RegisterAttributeDriver",
        "UnregisterAttributeDriver",
    ] {
        let is_func: bool = env
            .eval(&format!("return type({}) == 'function'", func))
            .unwrap();
        assert!(is_func, "{} should be a function", func);
    }
}

#[test]
fn test_secure_handlers_store_frame_refs_and_execute_snippets() {
    let env = env();
    let (ran_original, updated_attribute, stored_ref): (bool, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local ref = CreateFrame("Frame")
            local originalRan = false

            frame:SetAttribute("testAttr", "before")
            frame:SetScript("OnShow", function() originalRan = true end)
            frame:Hide()

            SecureHandlerSetFrameRef(frame, "target", ref)
            SecureHandlerExecute(frame, "self:SetAttribute('testAttr', 'after')")
            SecureHandlerWrapScript(frame, "OnShow", "self:SetAttribute('wrapped', true)")

            frame:Show()

            return originalRan,
                frame:GetAttribute("testAttr") == "after",
                SecureHandlerGetFrameRef(frame, "target") == ref
            "#,
        )
        .unwrap();
    assert!(ran_original, "original script should still run");
    assert!(
        updated_attribute,
        "SecureHandlerExecute should run the snippet against the frame"
    );
    assert!(
        stored_ref,
        "SecureHandlerSetFrameRef should make the ref retrievable"
    );
}

#[test]
fn test_secure_handler_unwrap_script_restores_original_handler() {
    let env = env();
    let order: String = env
        .eval(
            r#"
            local frame = CreateFrame("Button", "SecurityUnwrapScriptButton", UIParent)
            local header = CreateFrame("Frame", "SecurityUnwrapScriptHeader", UIParent)
            local log = {}

            frame:SetScript("OnClick", function()
                log[#log + 1] = "original"
            end)

            SecureHandlerWrapScript(frame, "OnClick", header,
                "log[#log + 1] = 'pre'",
                "log[#log + 1] = 'post'"
            )

            SecureHandlerUnwrapScript(frame, "OnClick")
            frame:GetScript("OnClick")(frame, "LeftButton", false)

            return table.concat(log, "|")
            "#,
        )
        .unwrap();
    assert_eq!(order, "original");
}

#[test]
fn test_protect_methods_are_not_public_addon_surface() {
    let env = env();
    let (protect_type, set_protected_type, protect_errors, set_protected_errors): (
        String,
        String,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "ProtectMethodFrame", UIParent)
            local protectOk = pcall(function()
                frame:Protect()
            end)
            local setProtectedOk = pcall(function()
                frame:SetProtected(true)
            end)
            return type(frame.Protect), type(frame.SetProtected), not protectOk, not setProtectedOk
            "#,
        )
        .unwrap();
    assert_eq!(protect_type, "nil");
    assert_eq!(set_protected_type, "nil");
    assert!(protect_errors);
    assert!(set_protected_errors);
}

#[test]
fn test_admin_can_mark_frame_protected_for_internal_setup() {
    let env = env();
    let secure_protected: bool = env
        .eval(
            r#"
            ProtectMethodFrame = CreateFrame("Frame", "ProtectMethodFrame", UIParent)
            A_Admin.SetFrameProtected("ProtectMethodFrame", true)
            return ProtectMethodFrame:IsProtected()
            "#,
        )
        .unwrap();
    assert!(secure_protected);
}

#[test]
fn test_frame_identity_slots_dispatch_surrogate_methods() {
    let env = env();
    let (has_identity, zero_name, zero_protected, legacy_success): (bool, String, bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "IdentitySlotFrame", UIParent)
            A_Admin.SetFrameProtected("IdentitySlotFrame", true)

            local zeroSlotSurrogate = { [0] = frame[0] }
            setmetatable(zeroSlotSurrogate, getmetatable(frame))

            local oneSlotSurrogate = { [1] = frame }
            setmetatable(oneSlotSurrogate, getmetatable(frame))
            local legacySuccess = pcall(function()
                return oneSlotSurrogate:GetName()
            end)

            return frame[0] ~= nil,
                zeroSlotSurrogate:GetName(),
                zeroSlotSurrogate:IsProtected(),
                legacySuccess
            "#,
        )
        .unwrap();

    assert!(has_identity, "frame[0] should expose a dispatch identity token");
    assert_eq!(zero_name, "IdentitySlotFrame");
    assert!(zero_protected);
    assert!(!legacy_success, "[1]-only surrogates should not dispatch");
}

#[test]
fn test_frame_identity_slot_can_redirect_method_receiver() {
    let env = env();
    let (a_before, b_protected, redirected): (bool, bool, bool) = env
        .eval(
            r#"
            local a = CreateFrame("Frame", "IdentityRedirectA", UIParent)
            local b = CreateFrame("Frame", "IdentityRedirectB", UIParent)
            A_Admin.SetFrameProtected("IdentityRedirectB", true)
            local aBefore = a:IsProtected()
            local bProtected = b:IsProtected()

            a[0] = b[0]

            return aBefore, bProtected, a:IsProtected()
            "#,
        )
        .unwrap();

    assert!(!a_before);
    assert!(b_protected);
    assert!(redirected, "methods should dispatch through the frame[0] token");
}

#[test]
fn test_execute_attribute_calls_function_attribute_and_returns_success_tuple() {
    let env = env();
    let (success, first, second, called): (bool, String, i64, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            local seen = false
            frame:SetAttribute("menu-function", function(selfArg, unit, button, isKeyPress)
                seen = selfArg == frame and unit == "player" and button == "LeftButton" and isKeyPress == true
                return "ok", 42
            end)

            local success, first, second = frame:ExecuteAttribute("menu-function", frame, "player", "LeftButton", true)
            return success, first, second, seen
            "#,
        )
        .unwrap();

    assert!(success);
    assert_eq!(first, "ok");
    assert_eq!(second, 42);
    assert!(called);
}

#[test]
fn test_execute_attribute_runs_protected_string_body_with_frame_refs() {
    let env = env();
    env.exec(
        r#"
        local header = CreateFrame("Frame", "ExecuteAttributeProtectedHeader", UIParent)
        local target = CreateFrame("Frame", "ExecuteAttributeProtectedTarget", UIParent)
        header:SetFrameRef("target", target)
        header:SetAttribute("snippet", [[
            local ref = self:GetFrameRef("target")
            ref:SetAttribute("fromSnippet", ...)
            return "snippet-ok"
        ]])
        "#,
    )
    .unwrap();

    let header_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ExecuteAttributeProtectedHeader")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.widgets.get_mut(header_id).unwrap().is_protected = true;
    }

    let (success, returned, target_value): (bool, String, String) = env
        .eval(
            r#"
            local header = ExecuteAttributeProtectedHeader
            local target = ExecuteAttributeProtectedTarget
            local success, returned = header:ExecuteAttribute("snippet", "payload")
            return success, returned, target:GetAttribute("fromSnippet")
            "#,
        )
        .unwrap();

    assert!(success);
    assert_eq!(returned, "snippet-ok");
    assert_eq!(target_value, "payload");
}

#[test]
fn test_execute_attribute_rejects_unprotected_string_body() {
    let env = env();
    let (success, reason): (bool, String) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame")
            frame:SetAttribute("snippet", "return 'nope'")
            local success, reason = frame:ExecuteAttribute("snippet")
            return success, reason
            "#,
        )
        .unwrap();

    assert!(!success);
    assert_eq!(reason, "unsupported-unprotected-snippet");
}

#[test]
fn test_can_change_protected_state_matches_lockdown_rules() {
    let env = env();
    let (plain, protected, parent, anchored): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "CanChangeParent", UIParent)
            local protected = CreateFrame("Frame", "CanChangeProtected", parent)
            local anchored = CreateFrame("Frame", "CanChangeAnchored", UIParent)
            local plain = CreateFrame("Frame", "CanChangePlain", UIParent)

            anchored:SetPoint("TOPLEFT", protected, "BOTTOMLEFT", 0, -4)

            A_Admin.SetFrameProtected("CanChangeProtected", true)
            A_Admin.SetInCombat(true)
            forceinsecure()

            return plain:CanChangeProtectedState(),
                   protected:CanChangeProtectedState(),
                   parent:CanChangeProtectedState(),
                   anchored:CanChangeProtectedState()
            "#,
        )
        .unwrap();

    assert!(plain, "plain frames should stay mutable");
    assert!(!protected, "protected frame should be blocked");
    assert!(!parent, "ancestor of protected frame should be blocked");
    assert!(
        !anchored,
        "frame anchored to protected relation should be blocked"
    );
}

#[test]
fn test_can_change_protected_state_allows_secure_and_out_of_combat_calls() {
    let env = env();
    let (secure_in_combat, insecure_out_of_combat): (bool, bool) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "CanChangeSecureProtected", UIParent)
            A_Admin.SetFrameProtected("CanChangeSecureProtected", true)

            A_Admin.SetInCombat(true)
            local secureInCombat = frame:CanChangeProtectedState()

            A_Admin.SetInCombat(false)
            forceinsecure()
            local insecureOutOfCombat = frame:CanChangeProtectedState()

            return secureInCombat, insecureOutOfCombat
            "#,
        )
        .unwrap();

    assert!(secure_in_combat, "secure callers should bypass lockdown");
    assert!(
        insecure_out_of_combat,
        "insecure callers should be allowed outside combat"
    );
}

// ============================================================================
// forceinsecure + taint tracking
// ============================================================================

#[test]
fn test_forceinsecure() {
    let env = env();
    let secure_after: bool = env.eval("forceinsecure(); return issecure()").unwrap();
    assert!(!secure_after, "forceinsecure() should taint execution");
}

#[test]
fn test_securecall_restores_taint() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            local function inner()
                forceinsecure()
            end
            securecall(inner)
            return issecure()
            "#,
        )
        .unwrap();
    assert!(result, "securecall should restore taint state after call");
}

#[test]
fn test_securecall_clears_taint_for_nested_closures() {
    let env = env();
    let (secure_inside, nested_secret): (bool, bool) = env
        .eval(
            r#"
            local secureInside
            local nestedSecret
            debug.setstacktaint("TestAddon")
            securecall(function()
                secureInside = issecure()
                nestedSecret = issecretvalue(function() end)
            end)
            debug.setstacktaint(nil)
            return secureInside, nestedSecret
            "#,
        )
        .unwrap();

    assert!(
        secure_inside,
        "securecall should execute the target with a secure stack even when the caller is tainted"
    );
    assert!(
        !nested_secret,
        "closures created inside securecall should not inherit caller taint"
    );
}

#[test]
fn test_loadstring_taints() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            forceinsecure()
            local f = loadstring("return issecure()")
            return f()
            "#,
        )
        .unwrap();
    assert!(!result, "code from loadstring should be tainted");
}

#[test]
fn test_issecurevariable_detects_taint() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            forceinsecure()
            loadstring("_G.TAINTED_VAR = 1")()
            local secure = issecurevariable("TAINTED_VAR")
            return secure
            "#,
        )
        .unwrap();
    assert!(!result, "variable set by tainted code should be insecure");
}

#[test]
fn addon_tainted_table_slot_does_not_make_table_secret() {
    let env = env();
    let (slot_secure, slot_owner, secret, access, all_access, deep_access, stored): (
        bool,
        String,
        bool,
        bool,
        bool,
        bool,
        String,
    ) = env
        .eval(
            r#"
            local key = {}
            local function addonWrite()
                debug.setstacktaint("TaintProbeAddon")
                key.field = "public"
            end
            addonWrite()
            local secure, owner = issecurevariable(key, "field")
            local secret = issecretvalue(key)
            local access = canaccessvalue(key)
            local allAccess = canaccessallvalues(key, 1)
            local deepAccess = canaccesstable(key)
            local map = SecureTypes.CreateSecureMap()
            map:SetValue(key, "stored")
            return secure, owner, secret, access, allAccess, deepAccess, map:GetValue(key)
            "#,
        )
        .unwrap();
    assert!(!slot_secure);
    assert_eq!(slot_owner, "TaintProbeAddon");
    assert!(!secret);
    assert!(access);
    assert!(all_access);
    assert!(deep_access);
    assert_eq!(stored, "stored");
}

#[test]
fn test_secure_map_rejects_secret_keys_and_values() {
    let env = env();
    let (key_error, value_error, stored_value): (String, String, String) = env
        .eval(
            r#"
            forceinsecure()
            local map = SecureTypes.CreateSecureMap()
            local secretKey = loadstring("return 1")
            local secretValue = loadstring("return 2")

            local keyOk, keyErr = pcall(function()
                map:SetValue(secretKey, "safe")
            end)
            local valueOk, valueErr = pcall(function()
                map:SetValue("safe", secretValue)
            end)

            map:SetValue("safe", "stored")
            return keyOk and "" or keyErr,
                valueOk and "" or valueErr,
                map:GetValue("safe")
            "#,
        )
        .unwrap();

    assert!(
        key_error.contains("attempted to store a secret key in a SecureMap"),
        "secret key should be rejected, got: {key_error}"
    );
    assert!(
        value_error.contains("attempted to store a secret value in a SecureMap"),
        "secret value should be rejected, got: {value_error}"
    );
    assert_eq!(stored_value, "stored");
}

// ============================================================================
// issecretvalue / canaccessvalue / canaccessallvalues / canaccesstable
// ============================================================================

#[test]
fn test_issecretvalue_untainted() {
    let env = env();
    let result: bool = env.eval("return issecretvalue(print)").unwrap();
    assert!(!result, "engine function should not be secret");
}

#[test]
fn test_issecretvalue_tainted() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            forceinsecure()
            local f = loadstring("return 1")
            return issecretvalue(f)
            "#,
        )
        .unwrap();
    assert!(result, "loadstring result should be secret");
}

#[test]
fn test_canaccessvalue_untainted() {
    let env = env();
    let result: bool = env.eval("return canaccessvalue(print)").unwrap();
    assert!(result, "engine function should be accessible");
}

#[test]
fn test_canaccessvalue_tainted() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            forceinsecure()
            local f = loadstring("return 1")
            return canaccessvalue(f)
            "#,
        )
        .unwrap();
    assert!(!result, "loadstring result should not be accessible");
}

#[test]
fn test_canaccessallvalues_all_clean() {
    let env = env();
    let result: bool = env
        .eval("return canaccessallvalues(print, type, tostring)")
        .unwrap();
    assert!(result, "all engine values should be accessible");
}

#[test]
fn test_canaccessallvalues_one_tainted() {
    let env = env();
    let result: bool = env
        .eval(
            r#"
            forceinsecure()
            local f = loadstring("return 1")
            return canaccessallvalues(print, f, type)
            "#,
        )
        .unwrap();
    assert!(!result, "mixed values should fail access check");
}

#[test]
fn test_canaccesstable_clean() {
    let env = env();
    let result: bool = env.eval("return canaccesstable({1, 2, 3})").unwrap();
    assert!(result, "engine-created table should be accessible");
}

#[test]
fn test_party_roster_name_is_secret_value() {
    let env = env();
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    let (secret, accessible): (bool, bool) = env
        .eval("local name = UnitName('party1'); return issecretvalue(name), canaccessvalue(name)")
        .unwrap();
    assert!(secret, "party roster identity should be secret");
    assert!(
        !accessible,
        "party roster identity should not be directly accessible"
    );
}

#[test]
fn test_party_full_name_marks_name_and_realm_secret() {
    let env = env();
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    let (name_secret, realm_secret, all_accessible): (bool, bool, bool) = env
        .eval(
            r#"
            local name, realm = UnitFullName('party1')
            return issecretvalue(name), issecretvalue(realm), canaccessallvalues(name, realm)
            "#,
        )
        .unwrap();
    assert!(name_secret, "party full-name identity should be secret");
    assert!(realm_secret, "party realm identity should be secret");
    assert!(
        !all_accessible,
        "secret full-name fields should block bulk access"
    );
}

#[test]
fn test_table_containing_party_identity_is_not_accessible() {
    let env = env();
    env.exec("A_Admin.SetPartySize(1)").unwrap();
    let accessible: bool = env
        .eval("local t = { name = UnitName('party1') }; return canaccesstable(t)")
        .unwrap();
    assert!(
        !accessible,
        "tables containing secret identities should be secret"
    );
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn native_secret_contents_remain_inaccessible_in_public_table() {
    let env = env();
    let (shallow_secret, shallow_access, deep_access, wrapper_secret, wrapper_access): (
        bool,
        bool,
        bool,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            local wrapped = secretwrap("private")
            local publicTable = { nested = { value = wrapped } }
            return issecretvalue(publicTable), canaccessvalue(publicTable),
                canaccesstable(publicTable), issecretvalue(wrapped), canaccessvalue(wrapped)
            "#,
        )
        .unwrap();
    assert!(!shallow_secret);
    assert!(shallow_access);
    assert!(!deep_access);
    assert!(wrapper_secret);
    assert!(!wrapper_access);
}

#[test]
fn test_scrub_helpers_are_passthrough() {
    let env = env();
    let (first, third, first_secret, third_secret): (i32, String, i32, String) = env
        .eval(
            r#"
            local t = { marker = true }
            local a, b, c = scrub(7, t, "ok")
            local x, y, z = scrubsecretvalues(7, t, "ok")
            return a, c, x, z
            "#,
        )
        .unwrap();
    assert_eq!(first, 7, "scrub should preserve the first argument");
    assert_eq!(third, "ok", "scrub should preserve later arguments");
    assert_eq!(
        first_secret, 7,
        "scrubsecretvalues should preserve the first argument"
    );
    assert_eq!(
        third_secret, "ok",
        "scrubsecretvalues should preserve later arguments"
    );
}
