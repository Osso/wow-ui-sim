//! Tests for CreateFrame and Lua frame creation.

use std::path::Path;
use wow_ui_sim::loader::{create_frame_from_xml, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{clear_templates, get_template, parse_xml, register_template, XmlElement};

// Blizzard SharedXML paths for loading templates
const BLIZZARD_SHARED_XML_BASE_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXMLBase/Blizzard_SharedXMLBase.toc";
const BLIZZARD_SHARED_XML_TOC: &str =
    "/home/osso/Projects/wow/reference-addons/wow-ui-source/Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML_Mainline.toc";

/// Helper to load Blizzard_SharedXML templates for tests that need them.
/// Returns the environment with templates loaded.
fn env_with_shared_xml() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");

    // Load SharedXMLBase first (dependency)
    let base_path = Path::new(BLIZZARD_SHARED_XML_BASE_TOC);
    if base_path.exists() {
        if let Err(e) = load_addon(&env, base_path) {
            eprintln!("Warning: Failed to load SharedXMLBase: {}", e);
        }
    }

    // Load SharedXML (contains scroll templates)
    let shared_path = Path::new(BLIZZARD_SHARED_XML_TOC);
    if shared_path.exists() {
        if let Err(e) = load_addon(&env, shared_path) {
            eprintln!("Warning: Failed to load SharedXML: {}", e);
        }
    }

    env
}

// ============================================================================
// Basic CreateFrame Tests
// ============================================================================

#[test]
fn test_create_frame_basic() {
    let env = WowLuaEnv::new().unwrap();

    // Create a simple frame
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestBasicFrame", UIParent)
        frame:SetSize(100, 50)
    "#,
    )
    .unwrap();

    // Verify frame exists and has correct properties
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

    // Test creating different frame types
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

    // Verify each type
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

    // Create anonymous frame (nil name)
    env.exec(
        r#"
        local frame = CreateFrame("Frame", nil, UIParent)
        frame:SetSize(50, 50)
        TestAnonymousFrame = frame  -- Store reference manually
    "#,
    )
    .unwrap();

    // Verify frame exists but wasn't registered as global by CreateFrame
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

    // Verify parent-child relationship
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

    // When no parent is specified (nil), defaults to UIParent
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

    // Verify $parent was substituted
    let exists: bool = env.eval("return MyAddonFrameChild ~= nil").unwrap();
    let child_name: String = env.eval("return MyAddonFrameChild:GetName()").unwrap();

    assert!(exists, "Frame with substituted name should exist");
    assert_eq!(child_name, "MyAddonFrameChild");
}

#[test]
fn test_create_frame_parent_case_insensitive() {
    let env = WowLuaEnv::new().unwrap();

    // Test both $parent and $Parent work
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

    // Child should inherit strata from parent
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

    // Buttons should have NormalTexture, PushedTexture, etc. created automatically
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

    // Button should have Text FontString as child
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

    // Sliders should have Low, High, Text fontstrings and ThumbTexture
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
// Template Tests (Hardcoded Templates)
// ============================================================================

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

    // CheckButton should have button textures
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

    // Initially unchecked
    let initially_checked: bool = env.eval("return TestCheckButtonState:GetChecked()").unwrap();
    assert!(!initially_checked, "CheckButton should start unchecked");

    // Set checked
    env.exec("TestCheckButtonState:SetChecked(true)").unwrap();
    let now_checked: bool = env.eval("return TestCheckButtonState:GetChecked()").unwrap();
    assert!(now_checked, "CheckButton should be checked after SetChecked(true)");

    // Toggle off
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

    // UICheckButtonTemplate should create Text FontString
    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestCheckBoxTemplate", UIParent, "UICheckButtonTemplate")
        cb:SetSize(24, 24)
    "#,
    )
    .unwrap();

    // Should have Text child from template
    let has_text: bool = env.eval("return TestCheckBoxTemplate.Text ~= nil").unwrap();
    assert!(has_text, "CheckButton with UICheckButtonTemplate should have Text");

    // Text should be a FontString
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
// ScrollBar and ScrollFrame Tests
// ============================================================================

#[test]
fn test_create_scrollframe_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameBasic", UIParent)
        sf:SetSize(200, 300)
        sf:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestScrollFrameBasic:GetObjectType()").unwrap();
    assert_eq!(obj_type, "ScrollFrame");
}

#[test]
fn test_scrollframe_template_creates_scrollbar() {
    // Load SharedXML which contains FauxScrollFrameTemplate
    let env = env_with_shared_xml();

    // FauxScrollFrameTemplate creates ScrollBar with buttons
    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollFrameTemplate", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    // Should have ScrollBar child
    let has_scrollbar: bool = env
        .eval("return TestScrollFrameTemplate.ScrollBar ~= nil")
        .unwrap();
    assert!(
        has_scrollbar,
        "ScrollFrame with FauxScrollFrameTemplate should have ScrollBar"
    );
}

#[test]
fn test_scrollbar_has_buttons() {
    // Load SharedXML which contains FauxScrollFrameTemplate
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarButtons", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    // ScrollBar should have up/down buttons
    let has_up: bool = env
        .eval("return TestScrollBarButtons.ScrollBar.ScrollUpButton ~= nil")
        .unwrap();
    let has_down: bool = env
        .eval("return TestScrollBarButtons.ScrollBar.ScrollDownButton ~= nil")
        .unwrap();

    assert!(has_up, "ScrollBar should have ScrollUpButton");
    assert!(has_down, "ScrollBar should have ScrollDownButton");
}

#[test]
fn test_scrollbar_has_thumb_texture() {
    // Load SharedXML which contains FauxScrollFrameTemplate
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarThumb", UIParent, "FauxScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    // ScrollBar should have ThumbTexture
    let has_thumb: bool = env
        .eval("return TestScrollBarThumb.ScrollBar.ThumbTexture ~= nil")
        .unwrap();
    assert!(has_thumb, "ScrollBar should have ThumbTexture");
}

#[test]
fn test_scrollbar_track_textures() {
    // Load SharedXML which contains ListScrollFrameTemplate
    let env = env_with_shared_xml();

    // ListScrollFrameTemplate (inherits FauxScrollFrameTemplate) adds track textures
    // Note: FauxScrollFrameTemplate itself does NOT have track textures
    env.exec(
        r#"
        local sf = CreateFrame("ScrollFrame", "TestScrollBarTrack", UIParent, "ListScrollFrameTemplate")
        sf:SetSize(200, 300)
    "#,
    )
    .unwrap();

    // ListScrollFrameTemplate should have track textures on the frame itself
    let has_top: bool = env
        .eval("return TestScrollBarTrack.ScrollBarTop ~= nil")
        .unwrap();
    let has_bot: bool = env
        .eval("return TestScrollBarTrack.ScrollBarBottom ~= nil")
        .unwrap();

    assert!(has_top, "ListScrollFrame should have ScrollBarTop texture");
    assert!(has_bot, "ListScrollFrame should have ScrollBarBottom texture");
}

#[test]
fn test_slider_basic() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderBasic", UIParent)
        slider:SetSize(200, 20)
        slider:SetPoint("CENTER")
        slider:SetMinMaxValues(0, 100)
        slider:SetValue(50)
    "#,
    )
    .unwrap();

    let obj_type: String = env.eval("return TestSliderBasic:GetObjectType()").unwrap();
    let min_val: f32 = env.eval("return select(1, TestSliderBasic:GetMinMaxValues())").unwrap();
    let max_val: f32 = env.eval("return select(2, TestSliderBasic:GetMinMaxValues())").unwrap();

    assert_eq!(obj_type, "Slider");
    assert_eq!(min_val, 0.0);
    assert_eq!(max_val, 100.0);

    // Note: SetValue/GetValue are stub implementations that don't persist the value yet
    // Once implemented, uncomment:
    // let cur_val: f32 = env.eval("return TestSliderBasic:GetValue()").unwrap();
    // assert_eq!(cur_val, 50.0);
}

#[test]
fn test_slider_has_fontstrings() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local slider = CreateFrame("Slider", "TestSliderFontStrings", UIParent)
        slider:SetSize(200, 20)
    "#,
    )
    .unwrap();

    // Slider should have Low, High, and Text fontstrings
    let has_low: bool = env.eval("return TestSliderFontStrings.Low ~= nil").unwrap();
    let has_high: bool = env.eval("return TestSliderFontStrings.High ~= nil").unwrap();
    let has_text: bool = env.eval("return TestSliderFontStrings.Text ~= nil").unwrap();

    assert!(has_low, "Slider should have Low FontString");
    assert!(has_high, "Slider should have High FontString");
    assert!(has_text, "Slider should have Text FontString");
}

#[test]
fn test_hybrid_scroll_template() {
    // Load SharedXML which contains HybridScrollBarTemplate
    let env = env_with_shared_xml();

    // HybridScrollBarTemplate creates track textures and buttons
    env.exec(
        r#"
        local hsb = CreateFrame("Slider", "TestHybridScrollBar", UIParent, "HybridScrollBarTemplate")
        hsb:SetSize(16, 200)
    "#,
    )
    .unwrap();

    // Should have track textures
    let has_thumb: bool = env.eval("return TestHybridScrollBar.ThumbTexture ~= nil").unwrap();
    let has_top: bool = env.eval("return TestHybridScrollBar.ScrollBarTop ~= nil").unwrap();
    let has_mid: bool = env.eval("return TestHybridScrollBar.ScrollBarMiddle ~= nil").unwrap();
    let has_bot: bool = env.eval("return TestHybridScrollBar.ScrollBarBottom ~= nil").unwrap();

    assert!(has_thumb, "HybridScrollBar should have ThumbTexture");
    assert!(has_top, "HybridScrollBar should have ScrollBarTop");
    assert!(has_mid, "HybridScrollBar should have ScrollBarMiddle");
    assert!(has_bot, "HybridScrollBar should have ScrollBarBottom");

    // Should have scroll buttons
    let has_up: bool = env
        .eval("return TestHybridScrollBar.ScrollUpButton ~= nil")
        .unwrap();
    let has_down: bool = env
        .eval("return TestHybridScrollBar.ScrollDownButton ~= nil")
        .unwrap();

    assert!(has_up, "HybridScrollBar should have ScrollUpButton");
    assert!(has_down, "HybridScrollBar should have ScrollDownButton");
}

// ============================================================================
// XML Template Registry Tests
// ============================================================================

#[test]
fn test_register_xml_template() {
    // Clear any previous templates
    clear_templates();

    // Parse and register a virtual frame template
    let xml = r#"
        <Ui>
            <Frame name="MyCustomTemplate" virtual="true">
                <Size x="100" y="50"/>
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Title" inherits="GameFontNormal">
                            <Anchors>
                                <Anchor point="TOP" y="-5"/>
                            </Anchors>
                        </FontString>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    assert_eq!(ui.elements.len(), 1);

    // Register the template
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        assert!(frame.is_virtual == Some(true));
        register_template("MyCustomTemplate", "Frame", frame.clone());
    }

    // Verify template is registered
    let template = get_template("MyCustomTemplate");
    assert!(template.is_some(), "Template should be registered");

    let entry = template.unwrap();
    assert_eq!(entry.name, "MyCustomTemplate");
    assert_eq!(entry.widget_type, "Frame");
}

#[test]
fn test_xml_template_with_children() {
    clear_templates();

    // Template with child frames
    let xml = r#"
        <Ui>
            <Frame name="PanelTemplate" virtual="true">
                <Size x="300" y="200"/>
                <Frames>
                    <Frame parentKey="TitleContainer">
                        <Size x="280" y="24"/>
                        <Anchors>
                            <Anchor point="TOP" y="-10"/>
                        </Anchors>
                        <Layers>
                            <Layer level="ARTWORK">
                                <FontString parentKey="TitleText" inherits="GameFontNormal"/>
                            </Layer>
                        </Layers>
                    </Frame>
                    <Button parentKey="CloseButton">
                        <Size x="24" y="24"/>
                        <Anchors>
                            <Anchor point="TOPRIGHT" x="-5" y="-5"/>
                        </Anchors>
                    </Button>
                </Frames>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();

    if let XmlElement::Frame(frame) = &ui.elements[0] {
        register_template("PanelTemplate", "Frame", frame.clone());
    }

    let template = get_template("PanelTemplate").unwrap();

    // Verify template has frames
    let frames = template.frame.frames();
    assert!(
        frames.is_some(),
        "Template should have child frames defined"
    );
}

#[test]
fn test_xml_template_inheritance() {
    clear_templates();

    // Base template
    let base_xml = r#"
        <Ui>
            <Frame name="BaseTemplate" virtual="true">
                <Size x="100" y="100"/>
            </Frame>
        </Ui>
    "#;

    // Derived template that inherits from base
    let derived_xml = r#"
        <Ui>
            <Frame name="DerivedTemplate" virtual="true" inherits="BaseTemplate">
                <Size x="200" y="200"/>
            </Frame>
        </Ui>
    "#;

    // Register both
    let base_ui = parse_xml(base_xml).unwrap();
    if let XmlElement::Frame(frame) = &base_ui.elements[0] {
        register_template("BaseTemplate", "Frame", frame.clone());
    }

    let derived_ui = parse_xml(derived_xml).unwrap();
    if let XmlElement::Frame(frame) = &derived_ui.elements[0] {
        register_template("DerivedTemplate", "Frame", frame.clone());
    }

    // Verify both exist
    assert!(get_template("BaseTemplate").is_some());
    assert!(get_template("DerivedTemplate").is_some());

    // Derived should have inherits field pointing to base
    let derived = get_template("DerivedTemplate").unwrap();
    assert_eq!(
        derived.frame.inherits,
        Some("BaseTemplate".to_string())
    );
}

// ============================================================================
// CreateFrame with XML Template Tests
// ============================================================================

#[test]
fn test_create_frame_finds_xml_template() {
    clear_templates();

    let env = WowLuaEnv::new().unwrap();

    // Register a template with specific size
    let xml = r#"
        <Ui>
            <Frame name="TestSizeTemplate" virtual="true">
                <Size x="150" y="75"/>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        register_template("TestSizeTemplate", "Frame", frame.clone());
    }

    // Currently CreateFrame doesn't apply XML template sizes - this test documents current behavior
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestWithTemplate", UIParent, "TestSizeTemplate")
        -- Frame won't have size from template until we implement template application
    "#,
    )
    .unwrap();

    // Frame exists but won't have template's size (current limitation)
    let exists: bool = env.eval("return TestWithTemplate ~= nil").unwrap();
    assert!(exists);

    // Note: Template size application would be tested here once implemented:
    // let width: f32 = env.eval("return TestWithTemplate:GetWidth()").unwrap();
    // assert_eq!(width, 150.0, "Frame should have template's width");
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
        tex:SetColorTexture(1, 0, 0, 1)  -- Red
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
// Integration: Creating frames like real addons do
// ============================================================================

#[test]
fn test_addon_style_frame_creation() {
    let env = WowLuaEnv::new().unwrap();

    // Simulate typical addon frame creation pattern
    env.exec(
        r#"
        -- Main addon frame
        local AddonFrame = CreateFrame("Frame", "MyAddon", UIParent)
        AddonFrame:SetSize(400, 300)
        AddonFrame:SetPoint("CENTER")
        AddonFrame:SetFrameStrata("HIGH")

        -- Title bar
        local TitleBar = CreateFrame("Frame", "$parentTitleBar", AddonFrame)
        TitleBar:SetSize(400, 30)
        TitleBar:SetPoint("TOP")

        -- Title text
        local Title = TitleBar:CreateFontString("$parentTitle", "OVERLAY")
        Title:SetPoint("CENTER")
        Title:SetText("My Addon")

        -- Close button
        local CloseBtn = CreateFrame("Button", "$parentCloseButton", TitleBar)
        CloseBtn:SetSize(24, 24)
        CloseBtn:SetPoint("RIGHT", -5, 0)

        -- Content area
        local Content = CreateFrame("ScrollFrame", "$parentContent", AddonFrame)
        Content:SetSize(380, 250)
        Content:SetPoint("BOTTOM", 0, 10)
    "#,
    )
    .unwrap();

    // Verify frame hierarchy
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

    // Verify parent relationships
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

    // Verify strata inheritance
    let titlebar_strata: String = env.eval("return MyAddonTitleBar:GetFrameStrata()").unwrap();
    assert_eq!(titlebar_strata, "HIGH");
}

// ============================================================================
// Frame Creation from XML (via loader)
// ============================================================================

#[test]
fn test_create_frame_from_xml_basic() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    // Parse a simple frame XML
    let xml = r#"
        <Ui>
            <Frame name="XmlTestFrame" parent="UIParent">
                <Size x="200" y="100"/>
                <Anchors>
                    <Anchor point="CENTER"/>
                </Anchors>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify frame was created
    let exists: bool = env.eval("return XmlTestFrame ~= nil").unwrap();
    let width: f32 = env.eval("return XmlTestFrame:GetWidth()").unwrap();
    let height: f32 = env.eval("return XmlTestFrame:GetHeight()").unwrap();

    assert!(exists);
    assert_eq!(width, 200.0);
    assert_eq!(height, 100.0);
}

#[test]
fn test_create_frame_from_xml_with_template() {
    // Use unique names to avoid conflicts with other tests
    let env = WowLuaEnv::new().unwrap();

    // First, register a template with unique name
    let template_xml = r#"
        <Ui>
            <Frame name="TestPanelTemplateUnique" virtual="true">
                <Size x="300" y="200"/>
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="TitleText">
                            <Size x="280" y="20"/>
                            <Anchors>
                                <Anchor point="TOP" y="-10"/>
                            </Anchors>
                        </FontString>
                    </Layer>
                </Layers>
                <Frames>
                    <Button parentKey="CloseButton">
                        <Size x="24" y="24"/>
                        <Anchors>
                            <Anchor point="TOPRIGHT" x="-5" y="-5"/>
                        </Anchors>
                    </Button>
                </Frames>
            </Frame>
        </Ui>
    "#;

    let template_ui = parse_xml(template_xml).unwrap();
    if let XmlElement::Frame(frame) = &template_ui.elements[0] {
        // Virtual frame gets registered but not created
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify template was registered
    assert!(
        get_template("TestPanelTemplateUnique").is_some(),
        "Template should be registered"
    );

    // Now create a frame that inherits from the template
    let frame_xml = r#"
        <Ui>
            <Frame name="TestPanelUnique" parent="UIParent" inherits="TestPanelTemplateUnique">
                <Anchors>
                    <Anchor point="CENTER"/>
                </Anchors>
            </Frame>
        </Ui>
    "#;

    let frame_ui = parse_xml(frame_xml).unwrap();
    if let XmlElement::Frame(frame) = &frame_ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify frame was created with template's size
    let exists: bool = env.eval("return TestPanelUnique ~= nil").unwrap();
    let width: f32 = env.eval("return TestPanelUnique:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestPanelUnique:GetHeight()").unwrap();

    assert!(exists);
    assert_eq!(width, 300.0, "Frame should inherit template's width");
    assert_eq!(height, 200.0, "Frame should inherit template's height");

    // Verify template children were instantiated
    let has_title: bool = env.eval("return TestPanelUnique.TitleText ~= nil").unwrap();
    let has_close: bool = env.eval("return TestPanelUnique.CloseButton ~= nil").unwrap();

    assert!(has_title, "Frame should have TitleText from template");
    assert!(has_close, "Frame should have CloseButton from template");
}

#[test]
fn test_create_frame_from_xml_template_inheritance_chain() {
    // Use unique names to avoid conflicts with parallel tests and global template state
    let env = WowLuaEnv::new().unwrap();

    // Register base template with unique name
    let base_xml = r#"
        <Ui>
            <Frame name="TestBaseTemplateChain" virtual="true">
                <Size x="100" y="100"/>
                <Layers>
                    <Layer level="BACKGROUND">
                        <Texture parentKey="Bg" setAllPoints="true"/>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
    "#;

    let base_ui = parse_xml(base_xml).unwrap();
    if let XmlElement::Frame(frame) = &base_ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Register derived template with unique name
    let derived_xml = r#"
        <Ui>
            <Frame name="TestDerivedTemplateChain" virtual="true" inherits="TestBaseTemplateChain">
                <Size x="200" y="150"/>
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Title">
                            <Anchors>
                                <Anchor point="TOP" y="-5"/>
                            </Anchors>
                        </FontString>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
    "#;

    let derived_ui = parse_xml(derived_xml).unwrap();
    if let XmlElement::Frame(frame) = &derived_ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Create frame using derived template
    let frame_xml = r#"
        <Ui>
            <Frame name="TestFinalFrameChain" parent="UIParent" inherits="TestDerivedTemplateChain">
                <Anchors>
                    <Anchor point="CENTER"/>
                </Anchors>
            </Frame>
        </Ui>
    "#;

    let frame_ui = parse_xml(frame_xml).unwrap();
    if let XmlElement::Frame(frame) = &frame_ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify derived template's size is used (overrides base)
    let width: f32 = env.eval("return TestFinalFrameChain:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestFinalFrameChain:GetHeight()").unwrap();

    assert_eq!(width, 200.0, "Should have derived template's width");
    assert_eq!(height, 150.0, "Should have derived template's height");

    // Verify children from both base and derived templates
    let has_bg: bool = env.eval("return TestFinalFrameChain.Bg ~= nil").unwrap();
    let has_title: bool = env.eval("return TestFinalFrameChain.Title ~= nil").unwrap();

    assert!(has_bg, "Should have Bg from base template");
    assert!(has_title, "Should have Title from derived template");
}

#[test]
fn test_create_frame_from_xml_parent_key() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    // Frame with nested children using parentKey
    let xml = r#"
        <Ui>
            <Frame name="ParentKeyTestFrame" parent="UIParent">
                <Size x="400" y="300"/>
                <Frames>
                    <Frame parentKey="Header">
                        <Size x="400" y="30"/>
                        <Anchors>
                            <Anchor point="TOP"/>
                        </Anchors>
                        <Layers>
                            <Layer level="ARTWORK">
                                <FontString parentKey="Title">
                                    <Anchors>
                                        <Anchor point="CENTER"/>
                                    </Anchors>
                                </FontString>
                            </Layer>
                        </Layers>
                    </Frame>
                    <Frame parentKey="Content">
                        <Size x="380" y="250"/>
                        <Anchors>
                            <Anchor point="BOTTOM" y="10"/>
                        </Anchors>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify parentKey children are accessible
    let has_header: bool = env.eval("return ParentKeyTestFrame.Header ~= nil").unwrap();
    let has_content: bool = env.eval("return ParentKeyTestFrame.Content ~= nil").unwrap();
    let has_title: bool = env
        .eval("return ParentKeyTestFrame.Header.Title ~= nil")
        .unwrap();

    assert!(has_header, "Frame should have Header child via parentKey");
    assert!(has_content, "Frame should have Content child via parentKey");
    assert!(has_title, "Header should have Title child via parentKey");

    // Verify children_keys in Rust
    let state = env.state().borrow();
    let frame_id = state
        .widgets
        .get_id_by_name("ParentKeyTestFrame")
        .expect("Frame should exist");
    let frame = state.widgets.get(frame_id).unwrap();

    assert!(
        frame.children_keys.contains_key("Header"),
        "Rust children_keys should have Header"
    );
    assert!(
        frame.children_keys.contains_key("Content"),
        "Rust children_keys should have Content"
    );
}

#[test]
fn test_create_button_from_xml() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Button name="XmlTestButton" parent="UIParent" text="Click Me">
                <Size x="120" y="30"/>
                <Anchors>
                    <Anchor point="CENTER"/>
                </Anchors>
            </Button>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Button(frame) = &ui.elements[0] {
        create_frame_from_xml(&env, frame, "Button", None).unwrap();
    }

    // Verify button was created
    let exists: bool = env.eval("return XmlTestButton ~= nil").unwrap();
    let obj_type: String = env.eval("return XmlTestButton:GetObjectType()").unwrap();
    let text: String = env.eval("return XmlTestButton:GetText() or ''").unwrap();

    assert!(exists);
    assert_eq!(obj_type, "Button");
    assert_eq!(text, "Click Me");
}

#[test]
fn test_create_frame_from_xml_with_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="ScriptTestFrame" parent="UIParent">
                <Size x="100" y="100"/>
                <Scripts>
                    <OnLoad>
                        self.loadedFlag = true
                    </OnLoad>
                </Scripts>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // OnLoad should have been fired during creation
    let loaded: bool = env
        .eval("return ScriptTestFrame.loadedFlag == true")
        .unwrap();
    assert!(loaded, "OnLoad script should have set loadedFlag");
}

#[test]
fn test_create_frame_from_xml_with_keyvalues() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="KeyValueTestFrame" parent="UIParent">
                <Size x="100" y="100"/>
                <KeyValues>
                    <KeyValue key="myString" value="hello" type="string"/>
                    <KeyValue key="myNumber" value="42" type="number"/>
                    <KeyValue key="myBool" value="true" type="boolean"/>
                </KeyValues>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    // Verify KeyValues were applied
    let my_string: String = env.eval("return KeyValueTestFrame.myString").unwrap();
    let my_number: i32 = env.eval("return KeyValueTestFrame.myNumber").unwrap();
    let my_bool: bool = env.eval("return KeyValueTestFrame.myBool").unwrap();

    assert_eq!(my_string, "hello");
    assert_eq!(my_number, 42);
    assert!(my_bool);
}
