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
    assert!(result, "GetNormalFontObject should return the font set via SetNormalFontObject");
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

    assert!(normal_nil, "GetNormalFontObject should return nil when unset");
    assert!(highlight_nil, "GetHighlightFontObject should return nil when unset");
    assert!(disabled_nil, "GetDisabledFontObject should return nil when unset");
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

    // GetPushedTextOffset currently returns (0, 0) as a stub
    let (x, y): (f64, f64) = env
        .eval("return TestPushOffBtn:GetPushedTextOffset()")
        .unwrap();
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

// ============================================================================
// Texture Getter Methods (GetNormalTexture, etc.)
// ============================================================================

#[test]
fn test_get_normal_texture_not_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestGetNormTex", UIParent)
    "#,
    )
    .unwrap();

    let not_nil: bool = env
        .eval("return TestGetNormTex:GetNormalTexture() ~= nil")
        .unwrap();
    assert!(not_nil, "GetNormalTexture should return a non-nil value");

    let obj_type: String = env
        .eval("return TestGetNormTex:GetNormalTexture():GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "Texture", "GetNormalTexture should return a Texture");
}

#[test]
fn test_get_highlight_texture_not_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local btn = CreateFrame("Button", "TestGetHlTex", UIParent)"#)
        .unwrap();

    let not_nil: bool = env
        .eval("return TestGetHlTex:GetHighlightTexture() ~= nil")
        .unwrap();
    assert!(not_nil, "GetHighlightTexture should return a non-nil value");

    let obj_type: String = env
        .eval("return TestGetHlTex:GetHighlightTexture():GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "Texture", "GetHighlightTexture should return a Texture");
}

#[test]
fn test_get_pushed_texture_not_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local btn = CreateFrame("Button", "TestGetPushTex", UIParent)"#)
        .unwrap();

    let not_nil: bool = env
        .eval("return TestGetPushTex:GetPushedTexture() ~= nil")
        .unwrap();
    assert!(not_nil, "GetPushedTexture should return a non-nil value");

    let obj_type: String = env
        .eval("return TestGetPushTex:GetPushedTexture():GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "Texture", "GetPushedTexture should return a Texture");
}

#[test]
fn test_get_disabled_texture_not_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local btn = CreateFrame("Button", "TestGetDisTex", UIParent)"#)
        .unwrap();

    let not_nil: bool = env
        .eval("return TestGetDisTex:GetDisabledTexture() ~= nil")
        .unwrap();
    assert!(not_nil, "GetDisabledTexture should return a non-nil value");

    let obj_type: String = env
        .eval("return TestGetDisTex:GetDisabledTexture():GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "Texture", "GetDisabledTexture should return a Texture");
}

#[test]
fn test_get_texture_returns_child_of_button() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local btn = CreateFrame("Button", "TestTexChild", UIParent)"#)
        .unwrap();

    let parent_name: String = env
        .eval("return TestTexChild:GetNormalTexture():GetParent():GetName()")
        .unwrap();
    assert_eq!(parent_name, "TestTexChild", "Texture child should have button as parent");
}

// ============================================================================
// Texture Setter Methods (SetNormalTexture, etc.)
// ============================================================================

#[test]
fn test_set_normal_texture_with_path() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetNormTex", UIParent)
        btn:SetNormalTexture("Interface\\Buttons\\UI-Panel-Button-Up")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSetNormTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(
        btn.normal_texture.is_some(),
        "Button normal_texture should be set after SetNormalTexture with path"
    );
    assert!(
        btn.normal_texture.as_ref().unwrap().contains("UI-Panel-Button-Up"),
        "normal_texture should contain the texture path"
    );
}

#[test]
fn test_set_pushed_texture_with_path() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetPushTex", UIParent)
        btn:SetPushedTexture("Interface\\Buttons\\UI-Panel-Button-Down")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSetPushTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(
        btn.pushed_texture.is_some(),
        "Button pushed_texture should be set after SetPushedTexture with path"
    );
}

#[test]
fn test_set_highlight_texture_with_path() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetHlTex", UIParent)
        btn:SetHighlightTexture("Interface\\Buttons\\UI-Panel-Button-Highlight")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSetHlTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(
        btn.highlight_texture.is_some(),
        "Button highlight_texture should be set after SetHighlightTexture with path"
    );
}

#[test]
fn test_set_disabled_texture_with_path() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetDisTex", UIParent)
        btn:SetDisabledTexture("Interface\\Buttons\\UI-Panel-Button-Disabled")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSetDisTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(
        btn.disabled_texture.is_some(),
        "Button disabled_texture should be set after SetDisabledTexture with path"
    );
}

#[test]
fn test_set_texture_with_userdata_does_not_overwrite_path() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSetTexUD", UIParent)
        btn:SetNormalTexture("Interface\\Buttons\\UI-Panel-Button-Up")
        -- Now set with userdata (texture object) - should NOT overwrite path
        local tex = btn:GetNormalTexture()
        btn:SetNormalTexture(tex)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSetTexUD").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(
        btn.normal_texture.is_some(),
        "normal_texture should still be set after SetNormalTexture with userdata"
    );
    assert!(
        btn.normal_texture.as_ref().unwrap().contains("UI-Panel-Button-Up"),
        "normal_texture should still contain original path after userdata set"
    );
}

// ============================================================================
// Checked Texture Methods (CheckButton)
// ============================================================================

#[test]
fn test_set_checked_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestSetChkTex", UIParent)
        cb:SetCheckedTexture("Interface\\Buttons\\CheckButtonCheck")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let cb_id = state.widgets.get_id_by_name("TestSetChkTex").unwrap();
    let cb = state.widgets.get(cb_id).unwrap();

    assert!(
        cb.checked_texture.is_some(),
        "checked_texture should be set after SetCheckedTexture"
    );

    // CheckedTexture child should start hidden
    let tex_id = cb.children_keys.get("CheckedTexture").unwrap();
    let tex = state.widgets.get(*tex_id).unwrap();
    assert!(!tex.visible, "CheckedTexture child should start hidden");
}

#[test]
fn test_set_disabled_checked_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestSetDisChkTex", UIParent)
        cb:SetDisabledCheckedTexture("Interface\\Buttons\\CheckButtonCheckDisabled")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let cb_id = state.widgets.get_id_by_name("TestSetDisChkTex").unwrap();
    let cb = state.widgets.get(cb_id).unwrap();

    assert!(
        cb.disabled_checked_texture.is_some(),
        "disabled_checked_texture should be set after SetDisabledCheckedTexture"
    );

    let tex_id = cb.children_keys.get("DisabledCheckedTexture").unwrap();
    let tex = state.widgets.get(*tex_id).unwrap();
    assert!(!tex.visible, "DisabledCheckedTexture child should start hidden");
}

// ============================================================================
// Three-Slice Texture Methods
// ============================================================================

#[test]
fn test_set_left_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestLeftTex", UIParent)
        btn:SetLeftTexture("Interface\\Buttons\\Left")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestLeftTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(btn.left_texture.is_some(), "left_texture should be set");
    assert!(btn.left_texture.as_ref().unwrap().contains("Left"));
}

#[test]
fn test_set_middle_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestMidTex", UIParent)
        btn:SetMiddleTexture("Interface\\Buttons\\Middle")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestMidTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(btn.middle_texture.is_some(), "middle_texture should be set");
}

#[test]
fn test_set_right_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestRightTex", UIParent)
        btn:SetRightTexture("Interface\\Buttons\\Right")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestRightTex").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(btn.right_texture.is_some(), "right_texture should be set");
}

#[test]
fn test_set_three_slice_nil_clears() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestSliceNil", UIParent)
        btn:SetLeftTexture("Interface\\Buttons\\Left")
        btn:SetLeftTexture(nil)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestSliceNil").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    assert!(btn.left_texture.is_none(), "left_texture should be nil after setting nil");
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
    assert_eq!(obj_type, "FontString", "GetFontString should return a FontString");

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
        local frame = CreateFrame("Frame", "TestNoTextFrame", UIParent)
    "#,
    )
    .unwrap();

    let is_nil: bool = env
        .eval("return TestNoTextFrame:GetFontString() == nil")
        .unwrap();
    assert!(is_nil, "GetFontString should return nil for frame with no Text child");
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
    assert!(!enabled, "Button should be disabled after SetEnabled(false)");
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

    // GetButtonState is a stub returning "NORMAL"
    let state: String = env.eval("return TestBtnState:GetButtonState()").unwrap();
    assert_eq!(state, "NORMAL");
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

// ============================================================================
// SetHighlightAtlas Method
// ============================================================================

#[test]
fn test_set_highlight_atlas_creates_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestHlAtlas", UIParent)
        btn:SetHighlightAtlas("checkbox-minimal")
    "#,
    )
    .unwrap();

    // Verify the HighlightTexture child exists
    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestHlAtlas").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();
    assert!(
        btn.children_keys.contains_key("HighlightTexture"),
        "SetHighlightAtlas should create HighlightTexture child"
    );
}

// ============================================================================
// Texture child anchors
// ============================================================================

#[test]
fn test_texture_children_have_fill_parent_anchors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestTexAnchors", UIParent)
        btn:SetSize(100, 30)
    "#,
    )
    .unwrap();

    // Getting textures should create them with fill-parent anchors
    env.exec(
        r#"
        local _ = TestTexAnchors:GetNormalTexture()
        local _ = TestTexAnchors:GetPushedTexture()
        local _ = TestTexAnchors:GetHighlightTexture()
        local _ = TestTexAnchors:GetDisabledTexture()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let btn_id = state.widgets.get_id_by_name("TestTexAnchors").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();

    for key in &["NormalTexture", "PushedTexture", "HighlightTexture", "DisabledTexture"] {
        let tex_id = btn.children_keys.get(*key).expect(&format!("{} should exist", key));
        let tex = state.widgets.get(*tex_id).unwrap();
        assert!(
            !tex.anchors.is_empty(),
            "{} should have fill-parent anchors",
            key
        );
    }
}
