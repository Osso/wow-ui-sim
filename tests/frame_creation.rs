//! Tests for basic CreateFrame functionality.
//!
//! These tests cover frame creation, parent-child relationships, strata inheritance,
//! and widget-type defaults (button textures, slider fontstrings).

use wow_ui_sim::lua_api::WowLuaEnv;

// ============================================================================
// Basic CreateFrame Tests
// ============================================================================

#[test]
fn test_create_frame_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestBasicFrame", UIParent)
        frame:SetSize(100, 50)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestBasicFrame ~= nil").unwrap();
    let width: f32 = env.eval("return TestBasicFrame:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestBasicFrame:GetHeight()").unwrap();
    let obj_type: String = env.eval("return TestBasicFrame:GetObjectType()").unwrap();

    assert!(exists);
    assert_eq!(width, 100.0);
    assert_eq!(height, 50.0);
    assert_eq!(obj_type, "Frame");
}

#[test]
fn test_create_frame_types() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestFrame", UIParent)
        local button = CreateFrame("Button", "TestButton", UIParent)
        local checkbutton = CreateFrame("CheckButton", "TestCheckButton", UIParent)
        local slider = CreateFrame("Slider", "TestSlider", UIParent)
        local editbox = CreateFrame("EditBox", "TestEditBox", UIParent)
        local scrollframe = CreateFrame("ScrollFrame", "TestScrollFrame", UIParent)
        local statusbar = CreateFrame("StatusBar", "TestStatusBar", UIParent)
    "#,
    )
    .unwrap();

    let frame_type: String = env.eval("return TestFrame:GetObjectType()").unwrap();
    let button_type: String = env.eval("return TestButton:GetObjectType()").unwrap();
    let checkbutton_type: String = env.eval("return TestCheckButton:GetObjectType()").unwrap();
    let slider_type: String = env.eval("return TestSlider:GetObjectType()").unwrap();
    let editbox_type: String = env.eval("return TestEditBox:GetObjectType()").unwrap();
    let scrollframe_type: String = env.eval("return TestScrollFrame:GetObjectType()").unwrap();
    let statusbar_type: String = env.eval("return TestStatusBar:GetObjectType()").unwrap();

    assert_eq!(frame_type, "Frame");
    assert_eq!(button_type, "Button");
    assert_eq!(checkbutton_type, "CheckButton");
    assert_eq!(slider_type, "Slider");
    assert_eq!(editbox_type, "EditBox");
    assert_eq!(scrollframe_type, "ScrollFrame");
    assert_eq!(statusbar_type, "StatusBar");
}

#[test]
fn test_create_frame_anonymous() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", nil, UIParent)
        frame:SetSize(50, 50)
        TestAnonymousFrame = frame
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestAnonymousFrame:GetObjectType()").unwrap();
    let width: f32 = env.eval("return TestAnonymousFrame:GetWidth()").unwrap();

    assert_eq!(obj_type, "Frame");
    assert_eq!(width, 50.0);
}

// ============================================================================
// Parent-Child Relationship Tests
// ============================================================================

#[test]
fn test_create_frame_with_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "TestParentFrame", UIParent)
        parent:SetSize(200, 200)

        local child = CreateFrame("Frame", "TestChildFrame", parent)
        child:SetSize(50, 50)
    "#,
    )
    .unwrap();

    let parent_name: String = env.eval("return TestChildFrame:GetParent():GetName()").unwrap();
    assert_eq!(parent_name, "TestParentFrame");

    // Verify child is registered with parent in Rust
    let state = env.state().borrow();
    let parent_id = state
        .widgets
        .get_id_by_name("TestParentFrame")
        .expect("Parent should exist");
    let child_id = state
        .widgets
        .get_id_by_name("TestChildFrame")
        .expect("Child should exist");

    let parent_frame = state.widgets.get(parent_id).unwrap();
    assert!(
        parent_frame.children.contains(&child_id),
        "Parent should have child in children list"
    );
}

#[test]
fn test_create_frame_default_parent() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestDefaultParent", nil)
        frame:SetSize(50, 50)
    "#,
    )
    .unwrap();

    let parent_name: String = env.eval("return TestDefaultParent:GetParent():GetName()").unwrap();
    assert_eq!(parent_name, "UIParent");
}

// ============================================================================
// $parent Name Substitution Tests
// ============================================================================

#[test]
fn test_create_frame_parent_substitution() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "MyAddonFrame", UIParent)
        local child = CreateFrame("Frame", "$parentChild", parent)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return MyAddonFrameChild ~= nil").unwrap();
    let child_name: String = env.eval("return MyAddonFrameChild:GetName()").unwrap();

    assert!(exists, "Frame with substituted name should exist");
    assert_eq!(child_name, "MyAddonFrameChild");
}

#[test]
fn test_create_frame_parent_case_insensitive() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "ParentFrame", UIParent)
        local child1 = CreateFrame("Frame", "$parentButton", parent)
        local child2 = CreateFrame("Frame", "$ParentText", parent)
    "#,
    )
    .unwrap();

    let exists1: bool = env.eval("return ParentFrameButton ~= nil").unwrap();
    let exists2: bool = env.eval("return ParentFrameText ~= nil").unwrap();

    assert!(exists1, "$parent substitution should work");
    assert!(exists2, "$Parent substitution should work");
}

// ============================================================================
// Strata and Level Inheritance Tests
// ============================================================================

#[test]
fn test_create_frame_strata_inheritance() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "HighStrataParent", UIParent)
        parent:SetFrameStrata("DIALOG")
        parent:SetFrameLevel(10)

        local child = CreateFrame("Frame", "HighStrataChild", parent)
    "#,
    )
    .unwrap();

    let child_strata: String = env.eval("return HighStrataChild:GetFrameStrata()").unwrap();
    let child_level: i32 = env.eval("return HighStrataChild:GetFrameLevel()").unwrap();

    assert_eq!(child_strata, "DIALOG", "Child should inherit parent's strata");
    assert_eq!(child_level, 11, "Child level should be parent level + 1");
}

// ============================================================================
// Button Child Element Tests
// ============================================================================

#[test]
fn test_create_button_has_textures() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonTextures", UIParent)
        btn:SetSize(100, 30)
    "#,
    )
    .unwrap();

    let has_normal: bool = env
        .eval("return TestButtonTextures:GetNormalTexture() ~= nil")
        .unwrap();
    let has_pushed: bool = env
        .eval("return TestButtonTextures:GetPushedTexture() ~= nil")
        .unwrap();
    let has_highlight: bool = env
        .eval("return TestButtonTextures:GetHighlightTexture() ~= nil")
        .unwrap();

    assert!(has_normal, "Button should have NormalTexture");
    assert!(has_pushed, "Button should have PushedTexture");
    assert!(has_highlight, "Button should have HighlightTexture");
}

#[test]
fn test_create_button_has_text_fontstring() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestButtonText", UIParent)
        btn:SetText("Click Me")
    "#,
    )
    .unwrap();

    let has_text: bool = env.eval("return TestButtonText.Text ~= nil").unwrap();
    let text_content: String = env.eval("return TestButtonText:GetText()").unwrap();

    assert!(has_text, "Button should have Text FontString");
    assert_eq!(text_content, "Click Me");
}

// ============================================================================
// Slider Child Element Tests
// ============================================================================

#[test]
fn test_create_slider_has_children() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderChildren", UIParent)
        slider:SetSize(200, 20)
    "#,
    )
    .unwrap();

    let has_low: bool = env.eval("return TestSliderChildren.Low ~= nil").unwrap();
    let has_high: bool = env.eval("return TestSliderChildren.High ~= nil").unwrap();
    let has_text: bool = env.eval("return TestSliderChildren.Text ~= nil").unwrap();
    let has_thumb: bool = env
        .eval("return TestSliderChildren.ThumbTexture ~= nil")
        .unwrap();

    assert!(has_low, "Slider should have Low fontstring");
    assert!(has_high, "Slider should have High fontstring");
    assert!(has_text, "Slider should have Text fontstring");
    assert!(has_thumb, "Slider should have ThumbTexture");
}

// ============================================================================
// CheckButton Tests
// ============================================================================

#[test]
fn test_create_checkbutton_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCheckButton", UIParent)
        cb:SetSize(24, 24)
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestCheckButton:GetObjectType()").unwrap();
    assert_eq!(obj_type, "CheckButton");

    let has_normal: bool = env
        .eval("return TestCheckButton:GetNormalTexture() ~= nil")
        .unwrap();
    let has_pushed: bool = env
        .eval("return TestCheckButton:GetPushedTexture() ~= nil")
        .unwrap();
    let has_highlight: bool = env
        .eval("return TestCheckButton:GetHighlightTexture() ~= nil")
        .unwrap();

    assert!(has_normal, "CheckButton should have NormalTexture");
    assert!(has_pushed, "CheckButton should have PushedTexture");
    assert!(has_highlight, "CheckButton should have HighlightTexture");
}

#[test]
fn test_checkbutton_checked_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCheckButtonState", UIParent)
        cb:SetSize(24, 24)
    "#,
    )
    .unwrap();

    let initially_checked: bool = env.eval("return TestCheckButtonState:GetChecked()").unwrap();
    assert!(!initially_checked, "CheckButton should start unchecked");

    env.exec("TestCheckButtonState:SetChecked(true)").unwrap();
    let now_checked: bool = env.eval("return TestCheckButtonState:GetChecked()").unwrap();
    assert!(now_checked, "CheckButton should be checked after SetChecked(true)");

    env.exec("TestCheckButtonState:SetChecked(false)").unwrap();
    let now_unchecked: bool = env.eval("return TestCheckButtonState:GetChecked()").unwrap();
    assert!(
        !now_unchecked,
        "CheckButton should be unchecked after SetChecked(false)"
    );
}

#[test]
fn test_checkbutton_template_creates_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCheckBoxTemplate", UIParent, "UICheckButtonTemplate")
        cb:SetSize(24, 24)
    "#,
    )
    .unwrap();

    let has_text: bool = env.eval("return TestCheckBoxTemplate.Text ~= nil").unwrap();
    assert!(has_text, "CheckButton with UICheckButtonTemplate should have Text");

    let text_type: String = env
        .eval("return TestCheckBoxTemplate.Text:GetObjectType()")
        .unwrap();
    assert_eq!(text_type, "FontString");
}

#[test]
fn test_checkbutton_with_label() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCheckBoxWithLabel", UIParent, "UICheckButtonTemplate")
        cb:SetSize(24, 24)
        cb.Text:SetText("Enable Feature")
    "#,
    )
    .unwrap();

    let label_text: String = env.eval("return TestCheckBoxWithLabel.Text:GetText()").unwrap();
    assert_eq!(label_text, "Enable Feature");
}

// ============================================================================
// CreateTexture and CreateFontString Tests
// ============================================================================

#[test]
fn test_create_texture() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestTextureFrame", UIParent)
        frame:SetSize(100, 100)

        local tex = frame:CreateTexture("TestTexture", "BACKGROUND")
        tex:SetAllPoints()
        tex:SetColorTexture(1, 0, 0, 1)
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestTexture ~= nil").unwrap();
    let obj_type: String = env.eval("return TestTexture:GetObjectType()").unwrap();
    let parent: String = env.eval("return TestTexture:GetParent():GetName()").unwrap();

    assert!(exists);
    assert_eq!(obj_type, "Texture");
    assert_eq!(parent, "TestTextureFrame");
}

#[test]
fn test_create_fontstring() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestFontFrame", UIParent)
        frame:SetSize(200, 50)

        local fs = frame:CreateFontString("TestFS", "OVERLAY")
        fs:SetPoint("CENTER")
        fs:SetText("Hello World")
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestFS ~= nil").unwrap();
    let obj_type: String = env.eval("return TestFS:GetObjectType()").unwrap();
    let text: String = env.eval("return TestFS:GetText()").unwrap();

    assert!(exists);
    assert_eq!(obj_type, "FontString");
    assert_eq!(text, "Hello World");
}

// ============================================================================
// Integration: Addon-style frame creation
// ============================================================================

#[test]
fn test_addon_style_frame_creation() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local AddonFrame = CreateFrame("Frame", "MyAddon", UIParent)
        AddonFrame:SetSize(400, 300)
        AddonFrame:SetPoint("CENTER")
        AddonFrame:SetFrameStrata("HIGH")

        local TitleBar = CreateFrame("Frame", "$parentTitleBar", AddonFrame)
        TitleBar:SetSize(400, 30)
        TitleBar:SetPoint("TOP")

        local Title = TitleBar:CreateFontString("$parentTitle", "OVERLAY")
        Title:SetPoint("CENTER")
        Title:SetText("My Addon")

        local CloseBtn = CreateFrame("Button", "$parentCloseButton", TitleBar)
        CloseBtn:SetSize(24, 24)
        CloseBtn:SetPoint("RIGHT", -5, 0)

        local Content = CreateFrame("ScrollFrame", "$parentContent", AddonFrame)
        Content:SetSize(380, 250)
        Content:SetPoint("BOTTOM", 0, 10)
    "#,
    )
    .unwrap();

    let main_exists: bool = env.eval("return MyAddon ~= nil").unwrap();
    let titlebar_exists: bool = env.eval("return MyAddonTitleBar ~= nil").unwrap();
    let title_exists: bool = env.eval("return MyAddonTitleBarTitle ~= nil").unwrap();
    let close_exists: bool = env.eval("return MyAddonTitleBarCloseButton ~= nil").unwrap();
    let content_exists: bool = env.eval("return MyAddonContent ~= nil").unwrap();

    assert!(main_exists);
    assert!(titlebar_exists);
    assert!(title_exists);
    assert!(close_exists);
    assert!(content_exists);

    let titlebar_parent: String = env.eval("return MyAddonTitleBar:GetParent():GetName()").unwrap();
    let title_parent: String = env
        .eval("return MyAddonTitleBarTitle:GetParent():GetName()")
        .unwrap();
    let close_parent: String = env
        .eval("return MyAddonTitleBarCloseButton:GetParent():GetName()")
        .unwrap();

    assert_eq!(titlebar_parent, "MyAddon");
    assert_eq!(title_parent, "MyAddonTitleBar");
    assert_eq!(close_parent, "MyAddonTitleBar");

    let titlebar_strata: String = env.eval("return MyAddonTitleBar:GetFrameStrata()").unwrap();
    assert_eq!(titlebar_strata, "HIGH");
}

#[test]
fn test_checkbutton_text_from_global_string() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCbGlobalStr", UIParent, "UICheckButtonTemplate")
        cb:SetSize(24, 24)
        cb.Text:SetText(ADDON_FORCE_LOAD)
    "#,
    )
    .unwrap();

    let label: String = env.eval("return TestCbGlobalStr.Text:GetText()").unwrap();
    assert_eq!(label, "Load out of date AddOns");
}

