//! Tests for XML frameStrata and frameLevel attribute parsing.

use wow_ui_sim::loader::create_frame_from_xml;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{clear_templates, parse_xml, XmlElement};

#[test]
fn test_create_frame_from_xml_frame_strata() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let xml = r#"
        <Ui>
            <Frame name="DialogStrataFrame" parent="UIParent" frameStrata="DIALOG">
                <Size x="200" y="100"/>
            </Frame>
        </Ui>
    "#;

    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env.loader_env(), frame, "Frame", None, None).unwrap();
    }

    let strata: String = env.eval("return DialogStrataFrame:GetFrameStrata()").unwrap();
    assert_eq!(strata, "DIALOG");

    // Children should inherit the parent's strata
    let child_strata: String = env
        .eval(
            r#"
            local child = CreateFrame("Frame", "DialogChild", DialogStrataFrame)
            return child:GetFrameStrata()
            "#,
        )
        .unwrap();
    assert_eq!(child_strata, "DIALOG");
}

#[test]
fn test_frame_strata_inherited_from_template() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    let template_xml = r#"
        <Ui>
            <Frame name="HighStrataTemplate" virtual="true" frameStrata="HIGH">
                <Size x="100" y="100"/>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(template_xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env.loader_env(), frame, "Frame", None, None).unwrap();
    }

    let frame_xml = r#"
        <Ui>
            <Frame name="InheritsHighStrata" parent="UIParent" inherits="HighStrataTemplate">
                <Anchors><Anchor point="CENTER"/></Anchors>
            </Frame>
        </Ui>
    "#;
    let ui2 = parse_xml(frame_xml).unwrap();
    if let XmlElement::Frame(frame) = &ui2.elements[0] {
        create_frame_from_xml(&env.loader_env(), frame, "Frame", None, None).unwrap();
    }

    let strata: String = env.eval("return InheritsHighStrata:GetFrameStrata()").unwrap();
    assert_eq!(strata, "HIGH");
}
