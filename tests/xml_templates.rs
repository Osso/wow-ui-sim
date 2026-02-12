//! Tests for XML template registration and frame creation from XML.

use wow_ui_sim::loader::create_frame_from_xml;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{clear_templates, get_template, parse_xml, register_template, XmlElement};

/// Parse XML and create the first frame element via the loader.
fn create_first_frame(env: &WowLuaEnv, xml: &str, widget_type: &str) {
    let ui = parse_xml(xml).unwrap();
    match &ui.elements[0] {
        XmlElement::Frame(f) | XmlElement::Button(f) => {
            create_frame_from_xml(&env.loader_env(), f, widget_type, None, None).unwrap();
        }
        _ => panic!("Expected Frame or Button element"),
    }
}

/// Parse XML and register the first element as a template.
fn register_first_template(xml: &str, name: &str, widget_type: &str) {
    let ui = parse_xml(xml).unwrap();
    match &ui.elements[0] {
        XmlElement::Frame(f) | XmlElement::Button(f) => {
            register_template(name, widget_type, f.clone());
        }
        _ => panic!("Expected Frame or Button element"),
    }
}

// ============================================================================
// XML Template Registry Tests
// ============================================================================

#[test]
fn test_register_xml_template() {
    clear_templates();
    let xml = r#"<Ui><Frame name="MyCustomTemplate" virtual="true">
        <Size x="100" y="50"/>
        <Layers><Layer level="ARTWORK">
            <FontString parentKey="Title" inherits="GameFontNormal">
                <Anchors><Anchor point="TOP" y="-5"/></Anchors>
            </FontString>
        </Layer></Layers>
    </Frame></Ui>"#;

    register_first_template(xml, "MyCustomTemplate", "Frame");
    let entry = get_template("MyCustomTemplate").expect("Template should be registered");
    assert_eq!(entry.name, "MyCustomTemplate");
    assert_eq!(entry.widget_type, "Frame");
}

#[test]
fn test_xml_template_with_children() {
    clear_templates();
    let xml = r#"<Ui><Frame name="PanelTemplate" virtual="true">
        <Size x="300" y="200"/>
        <Frames>
            <Frame parentKey="TitleContainer"><Size x="280" y="24"/>
                <Anchors><Anchor point="TOP" y="-10"/></Anchors>
                <Layers><Layer level="ARTWORK">
                    <FontString parentKey="TitleText" inherits="GameFontNormal"/>
                </Layer></Layers>
            </Frame>
            <Button parentKey="CloseButton"><Size x="24" y="24"/>
                <Anchors><Anchor point="TOPRIGHT" x="-5" y="-5"/></Anchors>
            </Button>
        </Frames>
    </Frame></Ui>"#;

    register_first_template(xml, "PanelTemplate", "Frame");
    let template = get_template("PanelTemplate").unwrap();
    assert!(!template.frame.all_frame_elements().is_empty());
}

#[test]
fn test_xml_template_inheritance() {
    clear_templates();
    register_first_template(
        r#"<Ui><Frame name="BaseTemplate" virtual="true"><Size x="100" y="100"/></Frame></Ui>"#,
        "BaseTemplate", "Frame",
    );
    register_first_template(
        r#"<Ui><Frame name="DerivedTemplate" virtual="true" inherits="BaseTemplate">
            <Size x="200" y="200"/></Frame></Ui>"#,
        "DerivedTemplate", "Frame",
    );
    assert!(get_template("BaseTemplate").is_some());
    let derived = get_template("DerivedTemplate").unwrap();
    assert_eq!(derived.frame.inherits, Some("BaseTemplate".to_string()));
}

// ============================================================================
// CreateFrame with XML Template Tests
// ============================================================================

#[test]
fn test_create_frame_finds_xml_template() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    register_first_template(
        r#"<Ui><Frame name="TestSizeTemplate" virtual="true"><Size x="150" y="75"/></Frame></Ui>"#,
        "TestSizeTemplate", "Frame",
    );
    env.exec(r#"local f = CreateFrame("Frame", "TestWithTemplate", UIParent, "TestSizeTemplate")"#).unwrap();
    assert!(env.eval::<bool>("return TestWithTemplate ~= nil").unwrap());
}

// ============================================================================
// Frame Creation from XML Tests
// ============================================================================

#[test]
fn test_create_frame_from_xml_basic() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="XmlTestFrame" parent="UIParent">
        <Size x="200" y="100"/><Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#, "Frame");

    assert!(env.eval::<bool>("return XmlTestFrame ~= nil").unwrap());
    assert_eq!(env.eval::<f32>("return XmlTestFrame:GetWidth()").unwrap(), 200.0);
    assert_eq!(env.eval::<f32>("return XmlTestFrame:GetHeight()").unwrap(), 100.0);
}

#[test]
fn test_create_frame_from_xml_with_template() {
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="TestPanelTemplateUnique" virtual="true">
        <Size x="300" y="200"/>
        <Layers><Layer level="ARTWORK">
            <FontString parentKey="TitleText"><Size x="280" y="20"/>
                <Anchors><Anchor point="TOP" y="-10"/></Anchors>
            </FontString>
        </Layer></Layers>
        <Frames><Button parentKey="CloseButton"><Size x="24" y="24"/>
            <Anchors><Anchor point="TOPRIGHT" x="-5" y="-5"/></Anchors>
        </Button></Frames>
    </Frame></Ui>"#, "Frame");
    assert!(get_template("TestPanelTemplateUnique").is_some());

    create_first_frame(&env, r#"<Ui><Frame name="TestPanelUnique" parent="UIParent"
        inherits="TestPanelTemplateUnique">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#, "Frame");

    assert_eq!(env.eval::<f32>("return TestPanelUnique:GetWidth()").unwrap(), 300.0);
    assert_eq!(env.eval::<f32>("return TestPanelUnique:GetHeight()").unwrap(), 200.0);
    assert!(env.eval::<bool>("return TestPanelUnique.TitleText ~= nil").unwrap());
    assert!(env.eval::<bool>("return TestPanelUnique.CloseButton ~= nil").unwrap());
}

#[test]
fn test_create_frame_from_xml_template_inheritance_chain() {
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="TestBaseTemplateChain" virtual="true">
        <Size x="100" y="100"/>
        <Layers><Layer level="BACKGROUND">
            <Texture parentKey="Bg" setAllPoints="true"/>
        </Layer></Layers>
    </Frame></Ui>"#, "Frame");

    create_first_frame(&env, r#"<Ui><Frame name="TestDerivedTemplateChain" virtual="true"
        inherits="TestBaseTemplateChain"><Size x="200" y="150"/>
        <Layers><Layer level="ARTWORK">
            <FontString parentKey="Title"><Anchors><Anchor point="TOP" y="-5"/></Anchors></FontString>
        </Layer></Layers>
    </Frame></Ui>"#, "Frame");

    create_first_frame(&env, r#"<Ui><Frame name="TestFinalFrameChain" parent="UIParent"
        inherits="TestDerivedTemplateChain">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#, "Frame");

    assert_eq!(env.eval::<f32>("return TestFinalFrameChain:GetWidth()").unwrap(), 200.0);
    assert_eq!(env.eval::<f32>("return TestFinalFrameChain:GetHeight()").unwrap(), 150.0);
    assert!(env.eval::<bool>("return TestFinalFrameChain.Bg ~= nil").unwrap());
    assert!(env.eval::<bool>("return TestFinalFrameChain.Title ~= nil").unwrap());
}

#[test]
fn test_create_frame_from_xml_parent_key() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="ParentKeyTestFrame" parent="UIParent">
        <Size x="400" y="300"/>
        <Frames>
            <Frame parentKey="Header"><Size x="400" y="30"/>
                <Anchors><Anchor point="TOP"/></Anchors>
                <Layers><Layer level="ARTWORK">
                    <FontString parentKey="Title"><Anchors><Anchor point="CENTER"/></Anchors></FontString>
                </Layer></Layers>
            </Frame>
            <Frame parentKey="Content"><Size x="380" y="250"/>
                <Anchors><Anchor point="BOTTOM" y="10"/></Anchors>
            </Frame>
        </Frames>
    </Frame></Ui>"#, "Frame");

    assert!(env.eval::<bool>("return ParentKeyTestFrame.Header ~= nil").unwrap());
    assert!(env.eval::<bool>("return ParentKeyTestFrame.Content ~= nil").unwrap());
    assert!(env.eval::<bool>("return ParentKeyTestFrame.Header.Title ~= nil").unwrap());

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("ParentKeyTestFrame").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(frame.children_keys.contains_key("Header"));
    assert!(frame.children_keys.contains_key("Content"));
}

#[test]
fn test_create_button_from_xml() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Button name="XmlTestButton" parent="UIParent" text="Click Me">
        <Size x="120" y="30"/><Anchors><Anchor point="CENTER"/></Anchors>
    </Button></Ui>"#, "Button");

    assert!(env.eval::<bool>("return XmlTestButton ~= nil").unwrap());
    assert_eq!(env.eval::<String>("return XmlTestButton:GetObjectType()").unwrap(), "Button");
    assert_eq!(env.eval::<String>("return XmlTestButton:GetText() or ''").unwrap(), "Click Me");
}

#[test]
fn test_create_frame_from_xml_with_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="ScriptTestFrame" parent="UIParent">
        <Size x="100" y="100"/>
        <Scripts><OnLoad>self.loadedFlag = true</OnLoad></Scripts>
    </Frame></Ui>"#, "Frame");
    assert!(env.eval::<bool>("return ScriptTestFrame.loadedFlag == true").unwrap());
}

#[test]
fn test_create_frame_from_xml_with_keyvalues() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Frame name="KeyValueTestFrame" parent="UIParent">
        <Size x="100" y="100"/>
        <KeyValues>
            <KeyValue key="myString" value="hello" type="string"/>
            <KeyValue key="myNumber" value="42" type="number"/>
            <KeyValue key="myBool" value="true" type="boolean"/>
        </KeyValues>
    </Frame></Ui>"#, "Frame");

    assert_eq!(env.eval::<String>("return KeyValueTestFrame.myString").unwrap(), "hello");
    assert_eq!(env.eval::<i32>("return KeyValueTestFrame.myNumber").unwrap(), 42);
    assert!(env.eval::<bool>("return KeyValueTestFrame.myBool").unwrap());
}

/// Count children of a specific widget type under a named frame.
fn count_typed_children(env: &WowLuaEnv, name: &str, wt: wow_ui_sim::widget::WidgetType) -> usize {
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name(name).unwrap();
    let frame = state.widgets.get(id).unwrap();
    frame.children.iter()
        .filter(|&&cid| state.widgets.get(cid).is_some_and(|c| c.widget_type == wt))
        .count()
}

#[test]
fn test_template_children_not_duplicated() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(&env, r#"<Ui><Button name="TestCloseButtonBase" virtual="true">
        <Size x="24" y="24"/></Button></Ui>"#, "Button");
    create_first_frame(&env, r#"<Ui><Button name="TestCloseButtonAnchored" virtual="true"
        inherits="TestCloseButtonBase">
        <Anchors><Anchor point="TOPRIGHT" x="-2" y="-2"/></Anchors>
    </Button></Ui>"#, "Button");
    create_first_frame(&env, r#"<Ui><Frame name="TestPanelTemplate" virtual="true">
        <Size x="400" y="300"/>
        <Frames><Button name="$parentCloseButton" parentKey="CloseButton"
            inherits="TestCloseButtonAnchored"/></Frames>
    </Frame></Ui>"#, "Frame");
    create_first_frame(&env, r#"<Ui><Frame name="TestPanelInstance" parent="UIParent"
        inherits="TestPanelTemplate">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#, "Frame");

    assert!(env.eval::<bool>("return TestPanelInstance.CloseButton ~= nil").unwrap());
    let n = count_typed_children(&env, "TestPanelInstance", wow_ui_sim::widget::WidgetType::Button);
    assert_eq!(n, 1, "Template child Button should be created exactly once, found {n}");

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestPanelInstance").unwrap();
    let frame = state.widgets.get(id).unwrap();
    let btn_id = *frame.children_keys.get("CloseButton").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();
    assert!(!btn.anchors.is_empty(), "CloseButton should have anchors from template");
}

// ============================================================================
// Three-Slice Button Tests
// ============================================================================

const THREE_SLICE_TEMPLATE_XML: &str = r#"<Ui>
    <Button name="ThreeSliceButtonTemplate" mixin="ThreeSliceButtonMixin" virtual="true">
        <Size x="20" y="20"/>
        <Layers><Layer level="BACKGROUND">
            <Texture parentKey="Left"><Anchors><Anchor point="TOPLEFT"/></Anchors></Texture>
            <Texture parentKey="Right"><Anchors><Anchor point="TOPRIGHT"/></Anchors></Texture>
            <Texture parentKey="Center">
                <Anchors>
                    <Anchor point="TOPLEFT" relativeKey="$parent.Left" relativePoint="TOPRIGHT"/>
                    <Anchor point="BOTTOMRIGHT" relativeKey="$parent.Right" relativePoint="BOTTOMLEFT"/>
                </Anchors>
            </Texture>
        </Layer></Layers>
        <Frames><Frame parentKey="Controller" mixin="ButtonControllerMixin">
            <Scripts><OnLoad method="OnLoad"/></Scripts>
        </Frame></Frames>
    </Button>
    <Button name="BigRedThreeSliceButtonTemplate" inherits="ThreeSliceButtonTemplate" virtual="true">
        <Size x="441" y="128"/>
        <KeyValues><KeyValue key="atlasName" value="128-RedButton" type="string"/></KeyValues>
    </Button>
    <Button name="SharedButtonSmallTemplate" inherits="BigRedThreeSliceButtonTemplate" virtual="true">
        <Size x="138" y="28"/>
    </Button>
</Ui>"#;

/// Set up env with three-slice templates and mixins registered.
fn setup_three_slice_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let ui = parse_xml(THREE_SLICE_TEMPLATE_XML).unwrap();
    for element in &ui.elements {
        if let XmlElement::Button(frame) = element {
            if let Some(ref name) = frame.name {
                register_template(name, "Button", frame.clone());
            }
        }
    }
    env.exec(r#"
        ThreeSliceButtonMixin = {}
        function ThreeSliceButtonMixin:InitButton()
            self.leftAtlasInfo = C_Texture.GetAtlasInfo(self.atlasName .. "-Left")
            self.rightAtlasInfo = C_Texture.GetAtlasInfo(self.atlasName .. "-Right")
            self:SetHighlightAtlas(self.atlasName .. "-Highlight")
        end
        function ThreeSliceButtonMixin:UpdateButton(buttonState)
            buttonState = buttonState or "NORMAL"
            self.Left:SetAtlas(self.atlasName .. "-Left", true)
            self.Center:SetAtlas("_" .. self.atlasName .. "-Center")
            self.Right:SetAtlas(self.atlasName .. "-Right", true)
            self:UpdateScale()
        end
        function ThreeSliceButtonMixin:UpdateScale()
            local scale = self:GetHeight() / self.leftAtlasInfo.height
            self.Left:SetScale(scale)
            self.Right:SetScale(scale)
            self.Left:SetTexCoord(0, 1, 0, 1)
            self.Left:SetWidth(self.leftAtlasInfo.width)
            self.Right:SetTexCoord(0, 1, 0, 1)
            self.Right:SetWidth(self.rightAtlasInfo.width)
        end
        ButtonControllerMixin = {}
        function ButtonControllerMixin:OnLoad()
            self:GetParent():InitButton()
        end
    "#).unwrap();
    env
}

/// Three-slice InitButton runs via Controller:OnLoad after all templates applied.
#[test]
fn test_three_slice_button_texture_scaling() {
    let env = setup_three_slice_env();
    assert!(env.eval::<bool>("return C_Texture.GetAtlasInfo('128-RedButton-Left') ~= nil").unwrap());

    let result: String = env.eval(r#"
        local btn = CreateFrame("Button", "TestThreeSliceBtn", UIParent, "SharedButtonSmallTemplate")
        btn:SetSize(120, 22)
        if not btn.leftAtlasInfo then return "leftAtlasInfo nil" end
        if not btn.rightAtlasInfo then return "rightAtlasInfo nil" end
        return "ok"
    "#).unwrap();
    assert!(result.starts_with("ok"), "InitButton should have run: {result}");
}

/// Center texture gets non-zero width via cross-frame anchors to Left/Right siblings.
#[test]
fn test_three_slice_center_texture_layout() {
    let env = setup_three_slice_env();
    let result: String = env.eval(r#"
        local btn = CreateFrame("Button", "TestThreeSlice2", UIParent, "SharedButtonSmallTemplate")
        btn:SetSize(120, 22)
        if not btn.Center then return "Center child missing" end
        if btn.Center:GetNumPoints() ~= 2 then
            return "Center has " .. btn.Center:GetNumPoints() .. " anchors, expected 2"
        end
        btn:UpdateButton()
        local leftW = btn.Left:GetWidth()
        local rightW = btn.Right:GetWidth()
        if leftW == 0 then return "Left width 0" end
        if rightW == 0 then return "Right width 0" end
        local centerW = btn.Center:GetWidth()
        if centerW == 0 then return "Center width 0 (cross-frame anchors not resolving)" end
        return "ok:" .. string.format("L=%.1f R=%.1f C=%.1f", leftW, rightW, centerW)
    "#).unwrap();
    assert!(result.starts_with("ok"), "Center texture should have non-zero width: {result}");
}
