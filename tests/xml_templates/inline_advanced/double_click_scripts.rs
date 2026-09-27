use super::*;

#[test]
fn ordinary_xml_method_only_double_click_keeps_mixin_and_button_payload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlDoubleClickMixin = {}
        function XmlDoubleClickMixin:OnEnter()
            self.enterCount = (self.enterCount or 0) + 1
        end
        function XmlDoubleClickMixin:OnDoubleClick(button, down)
            self.doubleClicks = (self.doubleClicks or 0) + 1
            self.lastButton = button
            self.lastDown = down
        end
    "#,
    )
    .unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlMethodDoubleClick" parent="UIParent" mixin="XmlDoubleClickMixin">
            <Scripts><OnEnter method="OnEnter"/><OnDoubleClick method="OnDoubleClick"/></Scripts>
        </Button></Ui>"#,
        "Button",
    );

    let values: (bool, i32, bool, i32, String, bool) = env
        .eval(
            r#"local frame = XmlMethodDoubleClick
               local control = frame:GetScript("OnEnter")
               control(frame)
               local handler = frame:GetScript("OnDoubleClick")
               if handler then handler(frame, "LeftButton", true) end
               return type(control) == "function", frame.enterCount,
                      type(handler) == "function", frame.doubleClicks or 0,
                      frame.lastButton or "", frame.lastDown == true"#,
        )
        .unwrap();
    assert_eq!(values, (true, 1, true, 1, "LeftButton".into(), true));
}

#[test]
fn ordinary_xml_inline_only_double_click_runs_with_button_payload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineDoubleClick" parent="UIParent">
            <Scripts><OnDoubleClick>
                self.doubleClicks = (self.doubleClicks or 0) + 1
                self.lastButton = button
                self.lastDown = down
            </OnDoubleClick></Scripts>
        </Button></Ui>"#,
        "Button",
    );

    let values: (bool, i32, String, bool) = env
        .eval(
            r#"local frame = XmlInlineDoubleClick
               local handler = frame:GetScript("OnDoubleClick")
               if handler then handler(frame, "LeftButton", false) end
               return type(handler) == "function", frame.doubleClicks or 0,
                      frame.lastButton or "", frame.lastDown == false and frame.doubleClicks == 1"#,
        )
        .unwrap();
    assert_eq!(values, (true, 1, "LeftButton".into(), true));
}

#[test]
fn runtime_template_double_click_method_and_inline_handlers_run() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        RuntimeDoubleClickMixin = {}
        function RuntimeDoubleClickMixin:OnDoubleClick(button, down)
            self.methodCount = (self.methodCount or 0) + 1
            self.lastButton = button
            self.lastDown = down
        end
    "#,
    )
    .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <Button name="RuntimeMethodDoubleClickTemplate" virtual="true" mixin="RuntimeDoubleClickMixin">
                <Scripts><OnDoubleClick method="OnDoubleClick"/></Scripts>
            </Button>
            <Button name="RuntimeInlineDoubleClickTemplate" virtual="true">
                <Scripts><OnDoubleClick>
                    self.inlineCount = (self.inlineCount or 0) + 1
                    self.lastButton = button
                    self.lastDown = down
                </OnDoubleClick></Scripts>
            </Button>
        </Ui>"#,
        "TestRuntimeDoubleClickXml",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestRuntimeDoubleClickXml.toc"),
    )
    .unwrap();

    let values: (bool, i32, String, bool, bool, i32, String, bool) = env
        .eval(
            r#"local method = CreateFrame("Button", nil, UIParent, "RuntimeMethodDoubleClickTemplate")
               local inline = CreateFrame("Button", nil, UIParent, "RuntimeInlineDoubleClickTemplate")
               local methodHandler = method:GetScript("OnDoubleClick")
               local inlineHandler = inline:GetScript("OnDoubleClick")
               if methodHandler then methodHandler(method, "LeftButton", true) end
               if inlineHandler then inlineHandler(inline, "LeftButton", false) end
               return type(methodHandler) == "function", method.methodCount or 0,
                      method.lastButton or "", method.lastDown == true,
                      type(inlineHandler) == "function", inline.inlineCount or 0,
                      inline.lastButton or "", inline.lastDown == false and inline.inlineCount == 1"#,
        )
        .unwrap();
    assert_eq!(
        values,
        (
            true,
            1,
            "LeftButton".into(),
            true,
            true,
            1,
            "LeftButton".into(),
            true
        )
    );
}
