//! Tests for intrinsic frame types (ContainedAlertFrame, etc.).

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::xml::{parse_xml, register_template, XmlElement};

/// Register a ContainedAlertFrame intrinsic template and its mixin.
fn setup_contained_alert_frame(env: &WowLuaEnv) {
    let xml = r#"
        <Ui>
            <Button name="ContainedAlertFrame" mixin="ContainedAlertFrameMixin" intrinsic="true">
                <Size x="300" y="80"/>
            </Button>
        </Ui>
    "#;
    let ui = parse_xml(xml).unwrap();
    for element in &ui.elements {
        if let XmlElement::Button(frame) = element {
            if let Some(ref name) = frame.name {
                register_template(name, "Button", frame.clone());
            }
        }
    }

    env.exec(
        r#"
        ContainedAlertFrameMixin = {}
        function ContainedAlertFrameMixin:SetAlertText(text)
            self._alertText = text
        end
        "#,
    )
    .unwrap();
}

/// ContainedAlertFrame is a WoW intrinsic type that is a Button subtype.
/// CreateFrame("ContainedAlertFrame") should produce a Button with the
/// intrinsic mixin and size applied automatically.
#[test]
fn contained_alert_frame_is_button_with_mixin() {
    let env = WowLuaEnv::new().unwrap();
    setup_contained_alert_frame(&env);

    let obj_type: String = env
        .eval(
            r#"
            local f = CreateFrame("ContainedAlertFrame", "TestAlert", UIParent)
            return f:GetObjectType()
            "#,
        )
        .unwrap();
    assert_eq!(obj_type, "Button", "ContainedAlertFrame should be a Button");

    let has_method: bool = env
        .eval("return type(TestAlert.SetAlertText) == 'function'")
        .unwrap();
    assert!(has_method, "should have ContainedAlertFrameMixin methods");

    let width: f32 = env.eval("return TestAlert:GetWidth()").unwrap();
    assert!(
        (width - 300.0).abs() < 0.1,
        "should have intrinsic width 300, got {}",
        width
    );
}
