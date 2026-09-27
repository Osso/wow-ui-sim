use super::*;

const COLORSELECT_XML_TEXTURES: &str = r#"
    <ColorWheelTexture parentKey="Wheel"><Size x="128" y="128"/>
        <Color r="0.2" g="0.4" b="0.6" a="0.8"/></ColorWheelTexture>
    <ColorWheelThumbTexture parentKey="WheelThumb" file="Interface\Buttons\UI-ColorPicker-Buttons">
        <Size x="10" y="10"/><TexCoords left="0" right="0.15625" top="0" bottom="0.625"/>
    </ColorWheelThumbTexture>
    <ColorValueTexture parentKey="Value"><Size x="32" y="128"/></ColorValueTexture>
    <ColorValueThumbTexture parentKey="ValueThumb"><Size x="48" y="14"/></ColorValueThumbTexture>
    <ColorAlphaTexture parentKey="Alpha"><Size x="32" y="128"/></ColorAlphaTexture>
    <ColorAlphaThumbTexture parentKey="AlphaThumb" file="Interface\Buttons\UI-ColorPicker-Buttons">
        <Size x="48" y="14"/><TexCoords left="0.25" right="1" top="0" bottom="0.875"/>
    </ColorAlphaThumbTexture>
"#;

fn load_colorselect_xml_fixture(env: &WowLuaEnv) -> tempfile::TempDir {
    let xml = format!(
        r#"<Ui>
    <ColorSelect name="XmlInlineColorSelect" parent="UIParent">{COLORSELECT_XML_TEXTURES}</ColorSelect>
    <ColorSelect name="XmlColorSelectTextureTemplate" virtual="true">{COLORSELECT_XML_TEXTURES}</ColorSelect>
    <ColorSelect name="XmlInheritedColorSelect" parent="UIParent" inherits="XmlColorSelectTextureTemplate"/>
</Ui>"#
    );
    let dir = create_test_addon(&xml, "TestColorSelectXmlTextures");
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestColorSelectXmlTextures.toc"),
    )
    .expect("ColorSelect XML fixture should load");
    dir
}

fn assert_colorselect_xml_textures(env: &WowLuaEnv, frame: &str) {
    let region_count: i32 = env
        .eval(&format!("return {frame}:GetNumRegions()"))
        .unwrap();
    assert_eq!(region_count, 6, "{frame}: six declared textures only");

    for (role, key) in [
        ("ColorWheelTexture", "Wheel"),
        ("ColorWheelThumbTexture", "WheelThumb"),
        ("ColorValueTexture", "Value"),
        ("ColorValueThumbTexture", "ValueThumb"),
        ("ColorAlphaTexture", "Alpha"),
        ("ColorAlphaThumbTexture", "AlphaThumb"),
    ] {
        let bound: bool = env
            .eval(&format!(
                "local f = {frame}; local t = f:Get{role}(); return t ~= nil and t == f.{key} and t:GetObjectType() == 'Texture' and t:GetParent() == f"
            ))
            .unwrap();
        assert!(
            bound,
            "{frame}: {role} should bind the declared .{key} texture"
        );
    }

    assert_colorselect_xml_properties(env, frame);
}

fn assert_colorselect_xml_properties(env: &WowLuaEnv, frame: &str) {
    let (alpha_width, thumb_width, tlx, tly, blx, bly, trx, try_, brx, bry):
        (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) = env
        .eval(&format!(
            "local f = {frame}; return f.Alpha:GetWidth(), f.AlphaThumb:GetWidth(), f.AlphaThumb:GetTexCoord()"
        ))
        .unwrap();
    assert_eq!((alpha_width, thumb_width), (32.0, 48.0), "{frame}");
    assert_eq!(
        (tlx, tly, blx, bly, trx, try_, brx, bry),
        (0.25, 0.0, 0.25, 0.875, 1.0, 0.0, 1.0, 0.875),
        "{frame}"
    );
    let state = env.state().borrow();
    let parent_id = state.widgets.get_id_by_name(frame).unwrap();
    let wheel_id = state.widgets.get(parent_id).unwrap().children_keys["Wheel"];
    let fill = state.widgets.get(wheel_id).unwrap().color_texture.unwrap();
    assert_eq!(
        (fill.r, fill.g, fill.b, fill.a),
        (0.2, 0.4, 0.6, 0.8),
        "{frame}"
    );
}

#[test]
fn xml_colorselect_declared_textures_bind_to_getters() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let _dir = load_colorselect_xml_fixture(&env);
    assert_colorselect_xml_textures(&env, "XmlInlineColorSelect");
}

#[test]
fn xml_colorselect_inherited_textures_bind_to_getters() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let _dir = load_colorselect_xml_fixture(&env);
    assert_colorselect_xml_textures(&env, "XmlInheritedColorSelect");
}

#[test]
fn runtime_colorselect_xml_template_textures_bind_to_getters() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let _dir = load_colorselect_xml_fixture(&env);
    env.exec(
        r#"XmlRuntimeColorSelect = CreateFrame("ColorSelect", "XmlRuntimeColorSelect", UIParent, "XmlColorSelectTextureTemplate")"#,
    )
    .unwrap();
    assert_colorselect_xml_textures(&env, "XmlRuntimeColorSelect");
}
