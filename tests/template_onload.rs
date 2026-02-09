//! Regression tests for template OnLoad firing.
//!
//! Verifies that fire_on_load does NOT fire the mixin's OnLoad on child frames
//! that share the parent's mixin but have no <Scripts> section.

use wow_ui_sim::loader::create_frame_from_xml;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{clear_templates, parse_xml, register_template, XmlElement};

const SPELL_BTN_MIXIN_LUA: &str = r#"
    TestSpellBtnMixin = {}
    __test_onload_calls = {}
    function TestSpellBtnMixin:OnLoad()
        table.insert(__test_onload_calls, self:GetName())
        local button = self.Button
        if not button then
            error("self.Button is nil on " .. self:GetName())
        end
        button:SetSize(33, 33)
    end
"#;

const SPELL_BTN_TEMPLATE_XML: &str = r#"
    <Ui>
        <Frame name="TestSpellBtnTpl" mixin="TestSpellBtnMixin" virtual="true">
            <Size x="66" y="33"/>
            <Frames>
                <Button parentKey="Button" mixin="TestSpellBtnMixin">
                    <Size x="33" y="33"/>
                    <Anchors><Anchor point="RIGHT"/></Anchors>
                </Button>
            </Frames>
            <Scripts>
                <OnLoad method="OnLoad"/>
            </Scripts>
        </Frame>
    </Ui>
"#;

fn setup_spell_btn_env() -> WowLuaEnv {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(SPELL_BTN_MIXIN_LUA).unwrap();
    let ui = parse_xml(SPELL_BTN_TEMPLATE_XML).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        register_template("TestSpellBtnTpl", "Frame", frame.clone());
    }
    env
}

fn check_onload_only_on_parent(env: &WowLuaEnv, parent_name: &str) -> String {
    env.eval(&format!(
        r#"
        local f = _G["{parent_name}"]
        if not f then return "frame is nil" end
        if not f.Button then return "f.Button is nil" end
        local calls = __test_onload_calls
        for _, name in ipairs(calls) do
            if name ~= "{parent_name}" then
                return "OnLoad called on wrong frame: " .. name
            end
        end
        if #calls == 0 then return "OnLoad never called" end
        return "ok"
        "#
    ))
    .unwrap()
}

/// CreateFrame path: child with shared mixin must not fire OnLoad.
#[test]
fn template_child_shared_mixin_no_onload_lua() {
    let env = setup_spell_btn_env();

    env.exec(
        r#"CreateFrame("Frame", "SpellBtnLua", UIParent, "TestSpellBtnTpl")"#,
    )
    .unwrap();

    let result = check_onload_only_on_parent(&env, "SpellBtnLua");
    assert_eq!(result, "ok", "Lua CreateFrame path: {}", result);
}

/// XML loading path: child with shared mixin must not fire OnLoad.
#[test]
fn template_child_shared_mixin_no_onload_xml() {
    let env = setup_spell_btn_env();

    let xml = r#"
        <Ui>
            <Frame name="SpellBtnXml" inherits="TestSpellBtnTpl" parent="UIParent">
                <Anchors><Anchor point="CENTER"/></Anchors>
            </Frame>
        </Ui>
    "#;
    let ui = parse_xml(xml).unwrap();
    if let XmlElement::Frame(frame) = &ui.elements[0] {
        create_frame_from_xml(&env.loader_env(), frame, "Frame", None, None).unwrap();
    }

    let result = check_onload_only_on_parent(&env, "SpellBtnXml");
    assert_eq!(result, "ok", "XML loading path: {}", result);
}
