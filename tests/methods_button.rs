//! Tests for button-specific methods in methods_button.rs.
//!
//! Covers: font objects, texture getters/setters, enable/disable state,
//! click handling, RegisterForClicks, button state, GetFontString, and
//! three-slice texture methods.

use wow_ui_sim::lua_api::WowLuaEnv;

// ============================================================================
// Font Object Methods
// ============================================================================

#[test]
fn test_set_and_get_normal_font_object() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestFontObjBtn", UIParent)
        local font = CreateFrame("Frame", "TestFontObj", UIParent)
        btn:SetNormalFontObject(font)
    "#,
    )
    .unwrap();

    let result: bool = env
        .eval("return TestFontObjBtn:GetNormalFontObject() == TestFontObj")
        .unwrap();
    assert!(
        result,
        "GetNormalFontObject should return the font set via SetNormalFontObject"
    );
}

#[test]
fn test_set_and_get_highlight_font_object() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestHlFontBtn", UIParent)
        local font = CreateFrame("Frame", "TestHlFont", UIParent)
        btn:SetHighlightFontObject(font)
    "#,
    )
    .unwrap();

    let result: bool = env
        .eval("return TestHlFontBtn:GetHighlightFontObject() == TestHlFont")
        .unwrap();
    assert!(
        result,
        "GetHighlightFontObject should return the font set via SetHighlightFontObject"
    );
}

#[test]
fn test_set_and_get_disabled_font_object() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestDisFontBtn", UIParent)
        local font = CreateFrame("Frame", "TestDisFont", UIParent)
        btn:SetDisabledFontObject(font)
    "#,
    )
    .unwrap();

    let result: bool = env
        .eval("return TestDisFontBtn:GetDisabledFontObject() == TestDisFont")
        .unwrap();
    assert!(
        result,
        "GetDisabledFontObject should return the font set via SetDisabledFontObject"
    );
}

#[test]
fn test_get_font_object_returns_nil_when_unset() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestNoFontBtn", UIParent)
    "#,
    )
    .unwrap();

    let normal_nil: bool = env
        .eval("return TestNoFontBtn:GetNormalFontObject() == nil")
        .unwrap();
    let highlight_nil: bool = env
        .eval("return TestNoFontBtn:GetHighlightFontObject() == nil")
        .unwrap();
    let disabled_nil: bool = env
        .eval("return TestNoFontBtn:GetDisabledFontObject() == nil")
        .unwrap();

    assert!(
        normal_nil,
        "GetNormalFontObject should return nil when unset"
    );
    assert!(
        highlight_nil,
        "GetHighlightFontObject should return nil when unset"
    );
    assert!(
        disabled_nil,
        "GetDisabledFontObject should return nil when unset"
    );
}

// ============================================================================
// Pushed Text Offset Methods
// ============================================================================

#[test]
fn test_pushed_text_offset() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestPushOffBtn", UIParent)
        btn:SetPushedTextOffset(2.5, -1.0)
    "#,
    )
    .unwrap();

    let (x, y): (f64, f64) = env
        .eval("return TestPushOffBtn:GetPushedTextOffset()")
        .unwrap();
    assert_eq!(x, 2.5);
    assert_eq!(y, -1.0);
}

#[test]
fn test_clear_button_texture_methods_clear_parent_fields_and_child_textures() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestClearBtnTex", UIParent)
        btn:SetNormalTexture("Interface/Buttons/UI-Panel-Button-Up")
        btn:SetHighlightTexture("Interface/Buttons/UI-Panel-Button-Highlight")
        btn:SetPushedTexture("Interface/Buttons/UI-Panel-Button-Down")
        btn:SetDisabledTexture("Interface/Buttons/UI-Panel-Button-Disabled")
        btn:ClearNormalTexture()
        btn:ClearHighlightTexture()
        btn:ClearPushedTexture()
        btn:ClearDisabledTexture()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let button_id = state.widgets.get_id_by_name("TestClearBtnTex").unwrap();
    let button = state.widgets.get(button_id).unwrap();
    assert_eq!(button.normal_texture, None);
    assert_eq!(button.highlight_texture, None);
    assert_eq!(button.pushed_texture, None);
    assert_eq!(button.disabled_texture, None);

    for key in [
        "NormalTexture",
        "HighlightTexture",
        "PushedTexture",
        "DisabledTexture",
    ] {
        let child_id = button
            .children_keys
            .get(key)
            .copied()
            .unwrap_or_else(|| panic!("Expected {key} child to exist"));
        let child = state.widgets.get(child_id).unwrap();
        assert_eq!(
            child.texture, None,
            "{key} child texture should be cleared on the child widget too"
        );
    }
}

// ============================================================================
// GetFontString Method
// ============================================================================

#[test]
fn test_get_font_string_returns_text_child() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestGetFontStr", UIParent)
        btn:SetText("Hello")
    "#,
    )
    .unwrap();

    // Verify GetFontString returns the Text child (a FontString)
    let not_nil: bool = env
        .eval("return TestGetFontStr:GetFontString() ~= nil")
        .unwrap();
    assert!(not_nil, "GetFontString should return a non-nil value");

    let obj_type: String = env
        .eval("return TestGetFontStr:GetFontString():GetObjectType()")
        .unwrap();
    assert_eq!(
        obj_type, "FontString",
        "GetFontString should return a FontString"
    );

    // Verify the Rust side has the Text child registered
    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestGetFontStr").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();
    assert!(
        btn.children_keys.contains_key("Text"),
        "Button should have a Text child in children_keys"
    );
}

#[test]
fn test_get_font_string_nil_when_no_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Button", "TestNoTextFrame", UIParent)
    "#,
    )
    .unwrap();

    let is_nil: bool = env
        .eval("return TestNoTextFrame:GetFontString() == nil")
        .unwrap();
    assert!(
        is_nil,
        "GetFontString should return nil for frame with no Text child"
    );
}

#[test]
fn test_get_font_string_exists_for_button_with_normal_font_but_no_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Button", "TestNoTextNormalFontFrame", UIParent)
        local font = CreateFrame("Frame", "TestNoTextNormalFontObject", UIParent)
        frame:SetNormalFontObject(font)
    "#,
    )
    .unwrap();

    let not_nil: bool = env
        .eval("return TestNoTextNormalFontFrame:GetFontString() ~= nil")
        .unwrap();
    assert!(
        not_nil,
        "GetFontString should synthesize a text region once a normal font object exists"
    );

    let obj_type: String = env
        .eval("return TestNoTextNormalFontFrame:GetFontString():GetObjectType()")
        .unwrap();
    assert_eq!(
        obj_type, "FontString",
        "buttons with only a normal font object should still expose a FontString"
    );

    let matches_named_global: bool = env
        .eval(
            "return TestNoTextNormalFontFrameText ~= nil and TestNoTextNormalFontFrame:GetFontString() == TestNoTextNormalFontFrameText",
        )
        .unwrap();
    assert!(
        matches_named_global,
        "synthetic button text child should bind the conventional $parentText global"
    );
}

#[test]
fn test_set_text_updates_button_text_child_without_synthesizing_text_key() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonButtonTextChild", UIParent)
        local fs = btn:CreateFontString(nil, "ARTWORK", "GameFontNormal")
        fs:SetPoint("CENTER")
        btn.ButtonText = fs
        btn:SetText("Category Header")
    "#,
    )
    .unwrap();

    let button_text: String = env
        .eval(
            r#"
            return TestButtonButtonTextChild.ButtonText
                and TestButtonButtonTextChild.ButtonText:GetText()
                or ""
            "#,
        )
        .unwrap();
    assert_eq!(
        button_text, "Category Header",
        "SetText should update an existing ButtonText region"
    );

    let synthesized_text_exists: bool = env
        .eval("return TestButtonButtonTextChild.Text ~= nil")
        .unwrap();
    assert!(
        !synthesized_text_exists,
        "SetText should reuse ButtonText instead of creating a synthetic Text child"
    );
}

#[test]
fn test_set_text_updates_lowercase_text_child_without_synthesizing_text_key() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonLowercaseTextChild", UIParent)
        local fs = btn:CreateFontString(nil, "ARTWORK", "GameFontNormal")
        fs:SetPoint("CENTER")
        btn.text = fs
        btn:SetText("World")
    "#,
    )
    .unwrap();

    let button_text: String = env
        .eval(
            r#"
            return TestButtonLowercaseTextChild.text
                and TestButtonLowercaseTextChild.text:GetText()
                or ""
            "#,
        )
        .unwrap();
    assert_eq!(
        button_text, "World",
        "SetText should update an existing lowercase text region"
    );

    let synthesized_text_exists: bool = env
        .eval("return TestButtonLowercaseTextChild.Text ~= nil")
        .unwrap();
    assert!(
        !synthesized_text_exists,
        "SetText should reuse lowercase text instead of creating a synthetic Text child"
    );
}

#[test]
fn test_set_text_finds_button_text_child_by_parent_key_when_children_keys_missing() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonTextParentKeyFallback", UIParent)
        local fs = btn:CreateFontString(nil, "ARTWORK", "GameFontNormal")
        fs:SetPoint("CENTER")
        btn.ButtonText = fs
    "#,
    )
    .unwrap();

    {
        let mut state = env.state().borrow_mut();
        let button_id = state
            .widgets
            .get_id_by_name("TestButtonTextParentKeyFallback")
            .expect("button should exist");
        let text_child_id = state
            .widgets
            .get(button_id)
            .and_then(|button| button.children_keys.get("ButtonText").copied())
            .expect("button should have a ButtonText child");

        let button = state
            .widgets
            .get_mut_visual(button_id)
            .expect("button should be mutable");
        button.children_keys.remove("ButtonText");
        button.children_keys.remove("Text");

        let text_child = state
            .widgets
            .get_mut_visual(text_child_id)
            .expect("button text child should be mutable");
        text_child.parent_key = Some("ButtonText".to_string());
    }

    env.exec(r#"TestButtonTextParentKeyFallback:SetText("Category Header")"#)
        .unwrap();

    let button_text: String = env
        .eval(
            r#"
            return TestButtonTextParentKeyFallback.ButtonText
                and TestButtonTextParentKeyFallback.ButtonText:GetText()
                or ""
            "#,
        )
        .unwrap();
    assert_eq!(
        button_text, "Category Header",
        "SetText should still update ButtonText when children_keys is stale"
    );

    let synthesized_text_exists: bool = env
        .eval("return TestButtonTextParentKeyFallback.Text ~= nil")
        .unwrap();
    assert!(
        !synthesized_text_exists,
        "SetText should not synthesize a Text child when an existing ButtonText child is discoverable"
    );
}

// ============================================================================
// Enable/Disable State Methods
// ============================================================================

#[test]
fn test_button_enabled_by_default() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local btn = CreateFrame("Button", "TestEnabledDef", UIParent)"#)
        .unwrap();

    let enabled: bool = env.eval("return TestEnabledDef:IsEnabled()").unwrap();
    assert!(enabled, "Buttons should be enabled by default");
}

#[test]
fn test_set_enabled_false() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetEnFalse", UIParent)
        btn:SetEnabled(false)
    "#,
    )
    .unwrap();

    let enabled: bool = env.eval("return TestSetEnFalse:IsEnabled()").unwrap();
    assert!(
        !enabled,
        "Button should be disabled after SetEnabled(false)"
    );
}

#[test]
fn test_set_enabled_true() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetEnTrue", UIParent)
        btn:SetEnabled(false)
        btn:SetEnabled(true)
    "#,
    )
    .unwrap();

    let enabled: bool = env.eval("return TestSetEnTrue:IsEnabled()").unwrap();
    assert!(enabled, "Button should be enabled after SetEnabled(true)");
}

#[test]
fn test_enable_method() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestEnMethod", UIParent)
        btn:SetEnabled(false)
        btn:Enable()
    "#,
    )
    .unwrap();

    let enabled: bool = env.eval("return TestEnMethod:IsEnabled()").unwrap();
    assert!(enabled, "Button should be enabled after Enable()");
}

#[test]
fn test_disable_method() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestDisMethod", UIParent)
        btn:Disable()
    "#,
    )
    .unwrap();

    let enabled: bool = env.eval("return TestDisMethod:IsEnabled()").unwrap();
    assert!(!enabled, "Button should be disabled after Disable()");
}

#[test]
fn test_motion_scripts_while_disabled_round_trip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestMotionScriptsWhileDisabled", UIParent)
        __motion_default = btn:GetMotionScriptsWhileDisabled()
        btn:SetMotionScriptsWhileDisabled(true)
        __motion_enabled = btn:GetMotionScriptsWhileDisabled()
        btn:SetMotionScriptsWhileDisabled(false)
        __motion_disabled = btn:GetMotionScriptsWhileDisabled()
    "#,
    )
    .unwrap();

    let (default_state, enabled_state, disabled_state): (bool, bool, bool) = env
        .eval("return __motion_default, __motion_enabled, __motion_disabled")
        .unwrap();

    assert!(
        !default_state,
        "motion scripts should default to disabled on new buttons"
    );
    assert!(
        enabled_state,
        "SetMotionScriptsWhileDisabled(true) should persist true"
    );
    assert!(
        !disabled_state,
        "SetMotionScriptsWhileDisabled(false) should clear the flag"
    );
}

// ============================================================================
// Click Method
// ============================================================================

#[test]
fn test_click_fires_onclick_handler() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestClickBtn", UIParent)
        __test_click_fired = false
        btn:SetScript("OnClick", function(self, button, down)
            __test_click_fired = true
            __test_click_button = button
            __test_click_down = down
        end)
        btn:Click()
    "#,
    )
    .unwrap();

    let fired: bool = env.eval("return __test_click_fired").unwrap();
    let button: String = env.eval("return __test_click_button").unwrap();
    let down: bool = env.eval("return __test_click_down").unwrap();

    assert!(fired, "Click() should fire OnClick handler");
    assert_eq!(button, "LeftButton", "Click() should pass 'LeftButton'");
    assert!(!down, "Click() should pass false for down");
}

#[test]
fn test_click_no_handler_does_not_error() {
    let env = WowLuaEnv::new().unwrap();

    // Click() on button with no OnClick handler should not error
    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestClickNoHandler", UIParent)
        btn:Click()
    "#,
    )
    .unwrap();
}

#[test]
fn test_click_dispatches_scripts_and_hooks_in_order_with_supplied_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        for _, script in ipairs({"PreClick", "OnClick", "PostClick"}) do
            btn:SetScript(script, function(self, button, down)
                assert(self == btn)
                calls[#calls + 1] = script .. ":script:" .. button .. ":" .. tostring(down)
            end)
            btn:HookScript(script, function(self, button, down)
                assert(self == btn)
                calls[#calls + 1] = script .. ":hook:" .. button .. ":" .. tostring(down)
            end)
        end
        btn:Click()
        btn:Click("RightButton", true)
        __button_click_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_click_calls").unwrap();
    let expected = ["LeftButton:false", "RightButton:true"]
        .into_iter()
        .flat_map(|args| {
            ["PreClick", "OnClick", "PostClick"]
                .into_iter()
                .flat_map(move |script| {
                    ["script", "hook"].map(move |binding| format!("{script}:{binding}:{args}"))
                })
        })
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(calls, expected);
}

#[test]
fn test_click_runs_pre_and_post_without_onclick() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        btn:SetScript("PreClick", function() calls[#calls + 1] = "pre" end)
        btn:SetScript("PostClick", function() calls[#calls + 1] = "post" end)
        btn:Click()
        __button_click_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_click_calls").unwrap();
    assert_eq!(calls, "pre,post");
}

#[test]
fn test_disabled_button_click_is_silent_until_enabled() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local calls = {}
        for _, script in ipairs({"PreClick", "OnClick", "PostClick"}) do
            btn:SetScript(script, function() calls[#calls + 1] = script end)
        end
        btn:Disable()
        btn:Click()
        __disabled_button_calls = table.concat(calls, ",")
        btn:Enable()
        btn:Click()
        __enabled_button_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let disabled: String = env.eval("return __disabled_button_calls").unwrap();
    let enabled: String = env.eval("return __enabled_button_calls").unwrap();
    assert_eq!(disabled, "");
    assert_eq!(enabled, "PreClick,OnClick,PostClick");
}

#[test]
fn test_disabled_checkbutton_click_toggles_without_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("CheckButton", nil, UIParent)
        local calls = 0
        btn:SetScript("OnClick", function() calls = calls + 1 end)
        btn:Disable()
        btn:Click()
        __disabled_check_checked = btn:GetChecked()
        __disabled_check_calls = calls
        btn:Enable()
        btn:Click()
        __enabled_check_checked = btn:GetChecked()
        __enabled_check_calls = calls
        "#,
    )
    .unwrap();
    let (disabled_checked, disabled_calls, enabled_checked, enabled_calls): (bool, i64, bool, i64) = env
        .eval("return __disabled_check_checked, __disabled_check_calls, __enabled_check_checked, __enabled_check_calls")
        .unwrap();
    assert!(disabled_checked);
    assert_eq!(disabled_calls, 0);
    assert!(!enabled_checked);
    assert_eq!(enabled_calls, 1);
}

#[test]
fn test_recursive_checkbutton_click_toggles_without_reentering_callbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("CheckButton", nil, UIParent)
        local calls = 0
        btn:SetScript("OnClick", function(self)
            calls = calls + 1
            self:Click()
            __recursive_check_checked_inside = self:GetChecked()
        end)
        btn:Click()
        __recursive_check_checked = btn:GetChecked()
        __recursive_check_calls = calls
        "#,
    )
    .unwrap();
    let (inside, checked, calls): (bool, bool, i64) = env
        .eval("return __recursive_check_checked_inside, __recursive_check_checked, __recursive_check_calls")
        .unwrap();
    assert!(!inside);
    assert!(!checked);
    assert_eq!(calls, 1);
}

#[test]
fn test_click_suppresses_same_button_recursion_but_allows_another_button() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local first = CreateFrame("Button", nil, UIParent)
        local second = CreateFrame("Button", nil, UIParent)
        local calls = {}
        first:SetScript("OnClick", function(self)
            calls[#calls + 1] = "first"
            self:Click()
            second:Click()
        end)
        second:SetScript("OnClick", function() calls[#calls + 1] = "second" end)
        first:Click()
        __button_recursion_calls = table.concat(calls, ",")
        "#,
    )
    .unwrap();
    let calls: String = env.eval("return __button_recursion_calls").unwrap();
    assert_eq!(calls, "first,second");
}

#[test]
fn test_click_onclick_error_is_reported_and_next_click_dispatches() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local errors = {}
        local calls = 0
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        btn:SetScript("OnClick", function()
            calls = calls + 1
            if calls == 1 then error("click handler sentinel") end
        end)
        btn:Click()
        btn:Click()
        __button_error_calls = calls
        __button_error_count = #errors
        __button_error_message = errors[1] or ""
        "#,
    )
    .unwrap();
    let (calls, count, message): (i64, i64, String) = env
        .eval("return __button_error_calls, __button_error_count, __button_error_message")
        .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(count, 1);
    assert!(message.contains("click handler sentinel"), "{message}");
}

#[test]
fn test_click_preclick_error_reports_and_later_phases_continue() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local btn = CreateFrame("Button", nil, UIParent)
        local errors, calls = {}, {}
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        btn:SetScript("PreClick", function()
            calls[#calls + 1] = "pre"
            if #calls == 1 then error("preclick sentinel") end
        end)
        btn:SetScript("OnClick", function() calls[#calls + 1] = "on" end)
        btn:SetScript("PostClick", function() calls[#calls + 1] = "post" end)
        btn:Click()
        btn:Click()
        __preclick_error_calls = table.concat(calls, ",")
        __preclick_error_count = #errors
        __preclick_error_message = errors[1] or ""
        "#,
    )
    .unwrap();
    let (calls, count, message): (String, i64, String) = env
        .eval("return __preclick_error_calls, __preclick_error_count, __preclick_error_message")
        .unwrap();
    assert_eq!(calls, "pre,on,post,pre,on,post");
    assert_eq!(count, 1);
    assert!(message.contains("preclick sentinel"), "{message}");
}

// ============================================================================
// RegisterForClicks Method
// ============================================================================

#[test]
fn test_register_for_clicks_no_error() {
    let env = WowLuaEnv::new().unwrap();

    // RegisterForClicks is a stub but should not error
    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestRegClicks", UIParent)
        btn:RegisterForClicks("AnyUp")
        btn:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    "#,
    )
    .unwrap();
}

// ============================================================================
// Button State Methods
// ============================================================================

#[test]
fn test_set_and_get_button_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestBtnState", UIParent)
        btn:SetButtonState("PUSHED", true)
    "#,
    )
    .unwrap();

    let state: String = env.eval("return TestBtnState:GetButtonState()").unwrap();
    assert_eq!(state, "PUSHED");
}

// ============================================================================
// SetFontString Method (stub)
// ============================================================================

#[test]
fn test_set_font_string_no_error() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetFontStr", UIParent)
        local fs = btn:GetFontString()
        btn:SetFontString(fs)
    "#,
    )
    .unwrap();
}
