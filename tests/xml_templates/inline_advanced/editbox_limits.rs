use super::*;

#[test]
fn xml_editbox_letters_reach_public_getter_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><EditBox name="XmlLettersControl" parent="UIParent" letters="4">
            <Scripts><OnLoad>self.lettersAtLoad = self:GetMaxLetters()</OnLoad></Scripts>
        </EditBox></Ui>"#,
        "EditBox",
    );

    let values: (i32, i32) = env
        .eval("return XmlLettersControl:GetMaxLetters(), XmlLettersControl.lettersAtLoad")
        .unwrap();
    assert_eq!(values, (4, 4));
}

#[test]
fn xml_editbox_bytes_reach_public_getter_and_reject_overflowing_keyboard_input() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><EditBox name="XmlByteLimited" parent="UIParent" bytes="4" letters="3">
            <Scripts><OnLoad>self.bytesAtLoad = self:GetMaxBytes(); self.lettersAtLoad = self:GetMaxLetters()</OnLoad></Scripts>
        </EditBox></Ui>"#,
        "EditBox",
    );

    let limits: (i32, i32, i32, i32) = env
        .eval("return XmlByteLimited:GetMaxBytes(), XmlByteLimited.bytesAtLoad, XmlByteLimited:GetMaxLetters(), XmlByteLimited.lettersAtLoad")
        .unwrap();
    assert_eq!(limits, (4, 4, 3, 3));

    env.exec("XmlByteLimited:SetFocus()").unwrap();
    env.send_key_press("E", Some("é")).unwrap();
    env.send_key_press("M", Some("猫")).unwrap(); // 5 UTF-8 bytes exceed the XML limit.
    env.send_key_press("A", Some("a")).unwrap();
    assert_eq!(
        env.eval::<String>("return XmlByteLimited:GetText()")
            .unwrap(),
        "éa"
    );
}

#[test]
fn xml_editbox_instance_zero_and_derived_template_override_fields_independently() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    register_first_template(
        r#"<Ui><EditBox name="XmlLimitBase" virtual="true" bytes="8" letters="9"/></Ui>"#,
        "XmlLimitBase",
        "EditBox",
    );
    register_first_template(
        r#"<Ui><EditBox name="XmlLimitDerived" virtual="true" inherits="XmlLimitBase" bytes="5" letters="7"/></Ui>"#,
        "XmlLimitDerived",
        "EditBox",
    );
    create_first_frame(
        &env,
        r#"<Ui><EditBox name="XmlLimitDerivedInstance" parent="UIParent" inherits="XmlLimitDerived" letters="0">
            <Scripts><OnLoad>self.bytesAtLoad = self:GetMaxBytes(); self.lettersAtLoad = self:GetMaxLetters()</OnLoad></Scripts>
        </EditBox></Ui>"#,
        "EditBox",
    );
    let values: (i32, i32, i32, i32) = env
        .eval("return XmlLimitDerivedInstance:GetMaxBytes(), XmlLimitDerivedInstance:GetMaxLetters(), XmlLimitDerivedInstance.bytesAtLoad, XmlLimitDerivedInstance.lettersAtLoad")
        .unwrap();
    assert_eq!(values, (5, 0, 5, 0));

    create_first_frame(
        &env,
        r#"<Ui><EditBox name="XmlLimitZeroBytes" parent="UIParent" inherits="XmlLimitDerived" bytes="0"/></Ui>"#,
        "EditBox",
    );
    let zero_bytes: (i32, i32) = env
        .eval("return XmlLimitZeroBytes:GetMaxBytes(), XmlLimitZeroBytes:GetMaxLetters()")
        .unwrap();
    assert_eq!(zero_bytes, (0, 7));
}

fn create_runtime_limit_templates(env: &WowLuaEnv) {
    let dir = create_test_addon(
        r#"<Ui>
            <EditBox name="RuntimeLimitEditTemplate" virtual="true" bytes="6" letters="4">
                <Scripts><OnLoad>self.bytesAtLoad = self:GetMaxBytes(); self.lettersAtLoad = self:GetMaxLetters()</OnLoad></Scripts>
            </EditBox>
            <Frame name="RuntimeLimitParentTemplate" virtual="true">
                <Frames><EditBox name="$parentInput" parentKey="Input" bytes="7" letters="2">
                    <Scripts><OnLoad>self.bytesAtLoad = self:GetMaxBytes(); self.lettersAtLoad = self:GetMaxLetters()</OnLoad></Scripts>
                </EditBox></Frames>
            </Frame>
        </Ui>"#,
        "TestRuntimeEditboxXmlLimits",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestRuntimeEditboxXmlLimits.toc"),
    )
    .expect("runtime XML templates must load");
}

#[test]
fn runtime_create_frame_editbox_template_applies_xml_limits_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_runtime_limit_templates(&env);
    let values: (i32, i32, i32, i32) = env
        .eval(
            r#"local box = CreateFrame("EditBox", nil, UIParent, "RuntimeLimitEditTemplate")
               return box:GetMaxBytes(), box:GetMaxLetters(), box.bytesAtLoad, box.lettersAtLoad"#,
        )
        .unwrap();
    assert_eq!(values, (6, 4, 6, 4));
}

#[test]
fn runtime_frame_template_nested_editbox_applies_xml_limits_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_runtime_limit_templates(&env);
    let values: (i32, i32, i32, i32) = env
        .eval(
            r#"local parent = CreateFrame("Frame", nil, UIParent, "RuntimeLimitParentTemplate")
               local box = parent.Input
               return box:GetMaxBytes(), box:GetMaxLetters(), box.bytesAtLoad, box.lettersAtLoad"#,
        )
        .unwrap();
    assert_eq!(values, (7, 2, 7, 2));
}
