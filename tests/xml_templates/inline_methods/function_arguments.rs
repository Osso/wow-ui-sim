use super::*;
#[test]
fn test_create_frame_from_xml_inline_function_with_self_and_parent_field_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        function XmlInlineParentFieldSetup(self, value)
            self.capturedPartyBackfill = value
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlInlineParentFieldFrame" parent="UIParent">
        <KeyValues>
            <KeyValue key="PartyBackfill" value="true" type="boolean"/>
        </KeyValues>
        <Frames>
            <Button parentKey="Child">
                <Scripts><OnLoad>XmlInlineParentFieldSetup(self, self:GetParent().PartyBackfill)</OnLoad></Scripts>
            </Button>
        </Frames>
    </Frame></Ui>"#,
        "Frame",
    );

    let captured: bool = env
        .eval("return XmlInlineParentFieldFrame.Child.capturedPartyBackfill == true")
        .unwrap();
    assert!(
        captured,
        "inline self+parent-field function call should fire"
    );
}

#[test]
fn test_create_frame_from_xml_inline_function_with_number_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlinePageChange = 0
        function XmlInlineSetPage(delta)
            XmlInlinePageChange = delta
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineNumberArgFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineSetPage(-1)</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec("XmlInlineNumberArgFrame:GetScript('OnClick')(XmlInlineNumberArgFrame)")
        .unwrap();

    let delta: i32 = env.eval("return XmlInlinePageChange").unwrap();
    assert_eq!(delta, -1, "inline numeric function arg should fire");
}

#[test]
fn test_create_frame_from_xml_inline_function_with_self_number_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        function XmlInlineSelectMailTab(self, tab_index)
            self.selectedTab = tab_index
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineSelfNumberArgFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineSelectMailTab(self, 2)</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec("XmlInlineSelfNumberArgFrame:GetScript('OnClick')(XmlInlineSelfNumberArgFrame)")
        .unwrap();

    let selected: i32 = env
        .eval("return XmlInlineSelfNumberArgFrame.selectedTab")
        .unwrap();
    assert_eq!(selected, 2, "inline self+number function arg should fire");
}

#[test]
fn test_create_frame_from_xml_inline_function_with_string_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineDebugMessage = nil
        function XmlInlineDebug(message)
            XmlInlineDebugMessage = message
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineStringArgFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineDebug("debug line")</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec("XmlInlineStringArgFrame:GetScript('OnClick')(XmlInlineStringArgFrame)")
        .unwrap();

    let message: String = env.eval("return XmlInlineDebugMessage").unwrap();
    assert_eq!(message, "debug line");
}

#[test]
fn test_create_frame_from_xml_inline_function_with_noarg_function_result_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineTrackedQuestId = nil
        function XmlInlineGetQuestId()
            return 42
        end
        function XmlInlineTrackQuest(quest_id)
            XmlInlineTrackedQuestId = quest_id
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineNoArgFunctionResultFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineTrackQuest(XmlInlineGetQuestId())</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec(
        "XmlInlineNoArgFunctionResultFrame:GetScript('OnClick')(XmlInlineNoArgFunctionResultFrame)",
    )
    .unwrap();

    let tracked: i32 = env.eval("return XmlInlineTrackedQuestId").unwrap();
    assert_eq!(tracked, 42);
}

#[test]
fn test_create_frame_from_xml_inline_function_with_parent_field_arg_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineTrackedQuestId = nil
        function XmlInlineTrackQuest(quest_id)
            XmlInlineTrackedQuestId = quest_id
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlInlineParentFieldOnlyFrame" parent="UIParent">
        <KeyValues>
            <KeyValue key="questID" value="42" type="number"/>
        </KeyValues>
        <Frames>
            <Button parentKey="Child">
                <Scripts><OnClick>XmlInlineTrackQuest(self:GetParent().questID)</OnClick></Scripts>
            </Button>
        </Frames>
    </Frame></Ui>"#,
        "Frame",
    );

    env.exec("XmlInlineParentFieldOnlyFrame.Child:GetScript('OnClick')(XmlInlineParentFieldOnlyFrame.Child)")
        .unwrap();

    let tracked: i32 = env.eval("return XmlInlineTrackedQuestId").unwrap();
    assert_eq!(tracked, 42);
}

#[test]
fn test_create_frame_from_xml_inline_function_with_global_method_result_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        _G.XmlInlineEditBox = {}
        function _G.XmlInlineEditBox:GetText()
            return "filter name"
        end
        XmlInlineCapturedFilterName = nil
        function XmlInlineSetFilterName(name)
            XmlInlineCapturedFilterName = name
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineGlobalMethodResultFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineSetFilterName(XmlInlineEditBox:GetText())</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec(
        "XmlInlineGlobalMethodResultFrame:GetScript('OnClick')(XmlInlineGlobalMethodResultFrame)",
    )
    .unwrap();

    let captured: String = env.eval("return XmlInlineCapturedFilterName").unwrap();
    assert_eq!(captured, "filter name");
}

#[test]
fn test_create_frame_from_xml_inline_function_with_string_number_args_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlInlineCVarKey = nil
        XmlInlineCVarValue = nil
        function XmlInlineSetCVar(key, value)
            XmlInlineCVarKey = key
            XmlInlineCVarValue = value
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlineStringNumberArgsFrame" parent="UIParent">
        <Scripts><OnClick>XmlInlineSetCVar("addFriendInfoShown", 1)</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec("XmlInlineStringNumberArgsFrame:GetScript('OnClick')(XmlInlineStringNumberArgsFrame)")
        .unwrap();

    let result: (String, i32) = env
        .eval("return XmlInlineCVarKey, XmlInlineCVarValue")
        .unwrap();
    assert_eq!(result.0, "addFriendInfoShown");
    assert_eq!(result.1, 1);
}

#[test]
fn test_create_frame_from_xml_inline_function_with_string_nil_nil_global_args_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        CHATCONFIG_SELECTED_FILTER = "SelectedFilter"
        XmlInlinePopupName = nil
        XmlInlinePopupData = nil
        function StaticPopup_Show(name, _, _, data)
            XmlInlinePopupName = name
            XmlInlinePopupData = data
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlInlinePopupShowFrame" parent="UIParent">
        <Scripts><OnClick>StaticPopup_Show("COPY_COMBAT_FILTER", nil, nil, CHATCONFIG_SELECTED_FILTER)</OnClick></Scripts>
    </Button></Ui>"#,
        "Button",
    );

    env.exec("XmlInlinePopupShowFrame:GetScript('OnClick')(XmlInlinePopupShowFrame)")
        .unwrap();

    let result: (String, String) = env
        .eval("return XmlInlinePopupName, XmlInlinePopupData")
        .unwrap();
    assert_eq!(result.0, "COPY_COMBAT_FILTER");
    assert_eq!(result.1, "SelectedFilter");
}

#[test]
fn test_create_frame_from_xml_inline_parent_field_method_with_self_result_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlInlineParentFieldMethodFrame" parent="UIParent">
        <Button parentKey="BuyButton"/>
        <CheckButton name="XmlInlineParentFieldMethodToggle" parent="XmlInlineParentFieldMethodFrame">
            <Scripts><OnClick>self:GetParent().BuyButton:SetEnabled(self:GetChecked())</OnClick></Scripts>
        </CheckButton>
    </Frame></Ui>"#,
        "Frame",
    );

    env.exec(
        r#"
        XmlInlineParentFieldMethodToggle:SetChecked(false)
        XmlInlineParentFieldMethodToggle:GetScript("OnClick")(XmlInlineParentFieldMethodToggle)
    "#,
    )
    .unwrap();
    let disabled: bool = env
        .eval(
            "local v = XmlInlineParentFieldMethodFrame.BuyButton:IsEnabled(); return v == true or v == 1",
        )
        .unwrap();
    assert!(!disabled);

    env.exec(
        r#"
        XmlInlineParentFieldMethodToggle:SetChecked(true)
        XmlInlineParentFieldMethodToggle:GetScript("OnClick")(XmlInlineParentFieldMethodToggle)
    "#,
    )
    .unwrap();
    let enabled: bool = env
        .eval(
            "local v = XmlInlineParentFieldMethodFrame.BuyButton:IsEnabled(); return v == true or v == 1",
        )
        .unwrap();
    assert!(enabled);
}
