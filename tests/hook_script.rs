//! Tests for HookScript return value and GetScript integration.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn hook_script_preserves_original_and_addon_hook_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        frame:Hide()
        local originalSecure, hookSecure
        frame:SetScript('OnShow', function() originalSecure = issecure() end)
        local function addon()
            frame:HookScript('OnShow', function() hookSecure = issecure() end)
            assert(not issecure(), 'hook construction cleared addon caller')
        end
        debug.setobjecttaint(addon, 'HookOriginAddon')
        addon()
        frame:Show()
        assert(originalSecure == true, 'hook wrapper tainted the trusted original')
        assert(hookSecure == false, 'addon hook became secure')
    "#,
    )
    .unwrap();
}

#[test]
fn secure_function_hook_preserves_original_origin_and_results() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local originalSecure, hookSecure
        HookOriginFunction = function(value)
            originalSecure = issecure()
            return value, nil, 3
        end
        local function addon()
            hooksecurefunc('HookOriginFunction', function() hookSecure = issecure() end)
            assert(not issecure(), 'hook construction cleared addon caller')
        end
        debug.setobjecttaint(addon, 'HookOriginAddon')
        addon()
        local first, gap, third = HookOriginFunction(7)
        assert(first == 7 and gap == nil and third == 3)
        assert(originalSecure == true and hookSecure == false)
        assert(not issecretvalue(HookOriginFunction))
        assert(issecurevariable('HookOriginFunction'))
        local function call_from_addon() HookOriginFunction(8) end
        debug.setobjecttaint(call_from_addon, 'HookCallerAddon')
        call_from_addon()
        assert(originalSecure == false and hookSecure == false,
            'calling a hooked function must not clear its caller taint')
    "#,
    )
    .unwrap();
}

#[test]
fn test_hook_script_returns_true() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        f:SetScript("OnShow", function() end)
        return f:HookScript("OnShow", function() end)
    "#,
        )
        .unwrap();
    assert!(result, "HookScript should return true");
}

#[test]
fn test_get_script_returns_different_function_after_hook() {
    let env = WowLuaEnv::new().unwrap();
    let changed: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        f:SetScript("OnShow", function() end)
        local original = f:GetScript("OnShow")
        f:HookScript("OnShow", function() end)
        return original ~= f:GetScript("OnShow")
    "#,
        )
        .unwrap();
    assert!(
        changed,
        "GetScript should return a different function after HookScript"
    );
}

#[test]
fn test_hook_script_chains_original_then_hook() {
    let env = WowLuaEnv::new().unwrap();
    let order: Vec<String> = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        local order = {}
        f:SetScript("OnShow", function() table.insert(order, "original") end)
        f:HookScript("OnShow", function() table.insert(order, "hook") end)
        local handler = f:GetScript("OnShow")
        handler(f)
        return order
    "#,
        )
        .unwrap();
    assert_eq!(order, vec!["original", "hook"]);
}

#[test]
fn test_hook_script_with_no_existing_handler() {
    let env = WowLuaEnv::new().unwrap();
    let called: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        local called = false
        f:HookScript("OnShow", function() called = true end)
        local handler = f:GetScript("OnShow")
        handler(f)
        return called
    "#,
        )
        .unwrap();
    assert!(
        called,
        "Hook should be callable via GetScript when no prior handler exists"
    );
}

#[test]
fn test_hook_script_rejects_pre_and_post_binding_slots() {
    let env = WowLuaEnv::new().unwrap();
    let (
        hook0_empty,
        hook2_empty,
        get0_nil,
        get2_nil,
        normal_hook,
        hook0_after_set,
        hook2_after_set,
        called,
    ): (bool, bool, bool, bool, bool, bool, bool, Vec<String>) = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        local hook0Empty = f:HookScript("OnShow", function() end, 0)
        local hook2Empty = f:HookScript("OnShow", function() end, 2)
        local get0Nil = f:GetScript("OnShow", 0) == nil
        local get2Nil = f:GetScript("OnShow", 2) == nil

        local normalHook = f:HookScript("OnShow", function() end)
        local called = {}
        f:SetScript("OnShow", function() table.insert(called, "normalSet") end)
        local hook0AfterSet = f:HookScript("OnShow", function() table.insert(called, "slot0") end, 0)
        local hook2AfterSet = f:HookScript("OnShow", function() table.insert(called, "slot2") end, 2)
        f:GetScript("OnShow")(f)
        return hook0Empty, hook2Empty, get0Nil, get2Nil, normalHook, hook0AfterSet, hook2AfterSet, called
    "#,
        )
        .unwrap();

    assert!(!hook0_empty);
    assert!(!hook2_empty);
    assert!(get0_nil);
    assert!(get2_nil);
    assert!(normal_hook);
    assert!(!hook0_after_set);
    assert!(!hook2_after_set);
    assert_eq!(called, ["normalSet"]);
}

#[test]
fn test_multiple_hook_scripts_chain_in_order() {
    let env = WowLuaEnv::new().unwrap();
    let order: Vec<String> = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        local order = {}
        f:SetScript("OnShow", function() table.insert(order, "original") end)
        f:HookScript("OnShow", function() table.insert(order, "hook1") end)
        f:HookScript("OnShow", function() table.insert(order, "hook2") end)
        local handler = f:GetScript("OnShow")
        handler(f)
        return order
    "#,
        )
        .unwrap();
    assert_eq!(order, vec!["original", "hook1", "hook2"]);
}

#[test]
fn test_scroll_frame_accepts_horizontal_scroll_script() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local frame = CreateFrame("ScrollFrame", "HorizontalScrollScriptFrame", UIParent)
        frame:SetScript("OnHorizontalScroll", function() end)
        frame:HookScript("OnHorizontalScroll", function() end)
        return frame:HasScript("OnHorizontalScroll")
    "#,
        )
        .unwrap();

    assert!(result, "ScrollFrame should support OnHorizontalScroll");
}

#[test]
fn test_set_script_invalid_handler_errors() {
    let env = WowLuaEnv::new().unwrap();
    let result = env.exec(
        r#"
        local f = CreateFrame("Frame")
        f:SetScript("OnNotARealScript", function() end)
    "#,
    );
    assert!(
        result.is_err(),
        "SetScript with unknown handler name should error"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("OnNotARealScript"),
        "Error message should name the invalid handler, got: {err}"
    );
}

#[test]
fn test_frame_supports_enable_disable_scripts() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
            local f = CreateFrame("Frame")
            f:SetScript("OnEnable", function() end)
            f:SetScript("OnDisable", function() end)
            return f:HasScript("OnEnable") and f:HasScript("OnDisable")
            "#,
        )
        .unwrap();
    assert!(result, "plain Frame should accept OnEnable and OnDisable");
}

#[test]
fn test_has_script_returns_false_for_onclick_on_plain_frame() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        return f:HasScript("OnClick")
    "#,
        )
        .unwrap();
    assert!(!result, "Plain Frame should not support OnClick");
}

#[test]
fn test_has_script_returns_true_for_onclick_on_button() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local b = CreateFrame("Button")
        return b:HasScript("OnClick")
    "#,
        )
        .unwrap();
    assert!(result, "Button should support OnClick");
}

#[test]
fn test_has_script_returns_false_for_bogus_name() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        return f:HasScript("OnNotARealScript")
    "#,
        )
        .unwrap();
    assert!(
        !result,
        "HasScript should return false for unknown handler names"
    );
}

#[test]
fn test_has_script_returns_true_for_base_handlers_on_frame() {
    let env = WowLuaEnv::new().unwrap();
    let result: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        return f:HasScript("OnShow") and f:HasScript("OnUpdate") and f:HasScript("OnEvent")
    "#,
        )
        .unwrap();
    assert!(
        result,
        "Frame should support base handlers OnShow, OnUpdate, OnEvent"
    );
}

#[test]
fn test_set_script_allows_ondoubleclick_on_plain_frame() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local f = CreateFrame("Frame")
        f:SetScript("OnDoubleClick", function() end)
    "#,
    )
    .unwrap();
    let has_script: bool = env
        .eval(
            r#"
        local f = CreateFrame("Frame")
        f:SetScript("OnDoubleClick", function() end)
        return f:HasScript("OnDoubleClick")
    "#,
        )
        .unwrap();
    assert!(has_script, "Plain Frame should accept OnDoubleClick");
}
