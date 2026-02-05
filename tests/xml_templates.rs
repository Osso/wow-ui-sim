//! Tests for XML template registration and frame creation from XML.
//!
//! These tests cover the template registry, template inheritance,
//! and creating frames from XML definitions.

use wow_ui_sim::loader::create_frame_from_xml;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{clear_templates, get_template, parse_xml, register_template, XmlElement};

// ============================================================================
// XML Template Registry Tests
// ============================================================================

#[test]
fn test_register_xml_template() {
    clear_templates();

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

    if let XmlElement::Frame(frame) = &ui.elements[0] {
        assert!(frame.is_virtual == Some(true));
        register_template("MyCustomTemplate", "Frame", frame.clone());
    }

    let template = get_template("MyCustomTemplate");
    assert!(template.is_some(), "Template should be registered");

    let entry = template.unwrap();
    assert_eq!(entry.name, "MyCustomTemplate");
    assert_eq!(entry.widget_type, "Frame");
}

#[test]
fn test_xml_template_with_children() {
    clear_templates();

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

    let frames = template.frame.frames();
    assert!(
        frames.is_some(),
        "Template should have child frames defined"
    );
}

#[test]
fn test_xml_template_inheritance() {
    clear_templates();

    let base_xml = r#"
        <Ui>
            <Frame name="BaseTemplate" virtual="true">
                <Size x="100" y="100"/>
            </Frame>
        </Ui>
    "#;

    let derived_xml = r#"
        <Ui>
            <Frame name="DerivedTemplate" virtual="true" inherits="BaseTemplate">
                <Size x="200" y="200"/>
            </Frame>
        </Ui>
    "#;

    let base_ui = parse_xml(base_xml).unwrap();
    if let XmlElement::Frame(frame) = &base_ui.elements[0] {
        register_template("BaseTemplate", "Frame", frame.clone());
    }

    let derived_ui = parse_xml(derived_xml).unwrap();
    if let XmlElement::Frame(frame) = &derived_ui.elements[0] {
        register_template("DerivedTemplate", "Frame", frame.clone());
    }

    assert!(get_template("BaseTemplate").is_some());
    assert!(get_template("DerivedTemplate").is_some());

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

    env.exec(
        r#"
        local frame = CreateFrame("Frame", "TestWithTemplate", UIParent, "TestSizeTemplate")
    "#,
    )
    .unwrap();

    let exists: bool = env.eval("return TestWithTemplate ~= nil").unwrap();
    assert!(exists);
}

// ============================================================================
// Frame Creation from XML Tests
// ============================================================================

#[test]
fn test_create_frame_from_xml_basic() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

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

    let exists: bool = env.eval("return XmlTestFrame ~= nil").unwrap();
    let width: f32 = env.eval("return XmlTestFrame:GetWidth()").unwrap();
    let height: f32 = env.eval("return XmlTestFrame:GetHeight()").unwrap();

    assert!(exists);
    assert_eq!(width, 200.0);
    assert_eq!(height, 100.0);
}

#[test]
fn test_create_frame_from_xml_with_template() {
    let env = WowLuaEnv::new().unwrap();

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
        create_frame_from_xml(&env, frame, "Frame", None).unwrap();
    }

    assert!(
        get_template("TestPanelTemplateUnique").is_some(),
        "Template should be registered"
    );

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

    let exists: bool = env.eval("return TestPanelUnique ~= nil").unwrap();
    let width: f32 = env.eval("return TestPanelUnique:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestPanelUnique:GetHeight()").unwrap();

    assert!(exists);
    assert_eq!(width, 300.0, "Frame should inherit template's width");
    assert_eq!(height, 200.0, "Frame should inherit template's height");

    let has_title: bool = env.eval("return TestPanelUnique.TitleText ~= nil").unwrap();
    let has_close: bool = env.eval("return TestPanelUnique.CloseButton ~= nil").unwrap();

    assert!(has_title, "Frame should have TitleText from template");
    assert!(has_close, "Frame should have CloseButton from template");
}

#[test]
fn test_create_frame_from_xml_template_inheritance_chain() {
    let env = WowLuaEnv::new().unwrap();

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

    let width: f32 = env.eval("return TestFinalFrameChain:GetWidth()").unwrap();
    let height: f32 = env.eval("return TestFinalFrameChain:GetHeight()").unwrap();

    assert_eq!(width, 200.0, "Should have derived template's width");
    assert_eq!(height, 150.0, "Should have derived template's height");

    let has_bg: bool = env.eval("return TestFinalFrameChain.Bg ~= nil").unwrap();
    let has_title: bool = env.eval("return TestFinalFrameChain.Title ~= nil").unwrap();

    assert!(has_bg, "Should have Bg from base template");
    assert!(has_title, "Should have Title from derived template");
}

#[test]
fn test_create_frame_from_xml_parent_key() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

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

    let has_header: bool = env.eval("return ParentKeyTestFrame.Header ~= nil").unwrap();
    let has_content: bool = env.eval("return ParentKeyTestFrame.Content ~= nil").unwrap();
    let has_title: bool = env
        .eval("return ParentKeyTestFrame.Header.Title ~= nil")
        .unwrap();

    assert!(has_header, "Frame should have Header child via parentKey");
    assert!(has_content, "Frame should have Content child via parentKey");
    assert!(has_title, "Header should have Title child via parentKey");

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

    let my_string: String = env.eval("return KeyValueTestFrame.myString").unwrap();
    let my_number: i32 = env.eval("return KeyValueTestFrame.myNumber").unwrap();
    let my_bool: bool = env.eval("return KeyValueTestFrame.myBool").unwrap();

    assert_eq!(my_string, "hello");
    assert_eq!(my_number, 42);
    assert!(my_bool);
}
