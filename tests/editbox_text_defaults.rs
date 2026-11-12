//! EditBox text defaults from XML templates: the box's `<FontString>`
//! element, its `<TextInsets>` and the LEFT justification of a new box, as the
//! WoW Forever 1.60.1 client reports them.
// Contributor-observed native defaults and font-object precedence await evidence.
#![cfg(feature = "optional-tests")]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

const TEMPLATES: &str = r#"<Ui xmlns="http://www.blizzard.com/wow/ui/">
    <Font name="EditBoxTextProbeFont" virtual="true" font="Fonts\ARIALN.TTF" height="14">
        <Color r="1.0" g="1.0" b="1.0"/>
    </Font>
    <EditBox name="EditBoxTextVisualTemplate" virtual="true">
        <FontString inherits="EditBoxTextProbeFont"/>
        <TextInsets left="16" right="20" bottom="0" top="0"/>
    </EditBox>
    <EditBox name="EditBoxTextSearchTemplate" inherits="EditBoxTextVisualTemplate" virtual="true"/>
    <EditBox name="EditBoxTextRightTemplate" virtual="true">
        <FontString inherits="EditBoxTextProbeFont" justifyH="RIGHT">
            <Color r="0.5" g="0.25" b="0.0"/>
        </FontString>
    </EditBox>
    <EditBox name="EditBoxTextXmlBox" parent="UIParent" inherits="EditBoxTextSearchTemplate" autoFocus="false"/>
</Ui>"#;

fn env_with_templates() -> (WowLuaEnv, tempfile::TempDir) {
    let env = WowLuaEnv::new().unwrap();
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(
        addon.path().join("EditBoxTextProbe.toc"),
        "## Title: EditBoxTextProbe\ntemplates.xml\n",
    )
    .unwrap();
    std::fs::write(addon.path().join("templates.xml"), TEMPLATES).unwrap();
    load_addon(
        &env.loader_env(),
        &addon.path().join("EditBoxTextProbe.toc"),
    )
    .unwrap();
    (env, addon)
}

#[test]
fn a_new_editbox_justifies_its_text_left() {
    let (env, _addon) = env_with_templates();
    let justify: String = env
        .eval(r#"return CreateFrame("EditBox", nil, UIParent):GetJustifyH()"#)
        .unwrap();
    assert_eq!(justify, "LEFT");
}

#[test]
fn an_editbox_takes_the_font_of_its_inherited_font_string_element() {
    let (env, _addon) = env_with_templates();
    let probe: (bool, bool, f64, f64, f64, String) = env
        .eval(
            r#"
            local runtime = CreateFrame("EditBox", nil, UIParent, "EditBoxTextSearchTemplate")
            local r, g, b = runtime:GetTextColor()
            return runtime:GetFontObject() == EditBoxTextProbeFont,
                EditBoxTextXmlBox:GetFontObject() == EditBoxTextProbeFont,
                r, g, b,
                runtime:GetJustifyH()
            "#,
        )
        .unwrap();
    assert_eq!(probe, (true, true, 1.0, 1.0, 1.0, "LEFT".to_string()));
}

#[test]
fn an_editbox_font_string_element_sets_justification_and_color() {
    let (env, _addon) = env_with_templates();
    let probe: (String, f64, f64, f64) = env
        .eval(
            r#"
            local box = CreateFrame("EditBox", nil, UIParent, "EditBoxTextRightTemplate")
            local r, g, b = box:GetTextColor()
            return box:GetJustifyH(), r, g, b
            "#,
        )
        .unwrap();
    assert_eq!(probe, ("RIGHT".to_string(), 0.5, 0.25, 0.0));
}

#[test]
fn an_editbox_takes_the_text_insets_of_its_template() {
    let (env, _addon) = env_with_templates();
    let probe: (f64, f64, f64, f64) = env
        .eval(
            r#"
            local runtime = CreateFrame("EditBox", nil, UIParent, "EditBoxTextSearchTemplate")
            local left, right = runtime:GetTextInsets()
            local xmlLeft, xmlRight = EditBoxTextXmlBox:GetTextInsets()
            return left, right, xmlLeft, xmlRight
            "#,
        )
        .unwrap();
    assert_eq!(probe, (16.0, 20.0, 16.0, 20.0));
}
