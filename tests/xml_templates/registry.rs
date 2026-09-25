use super::*;
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
fn anonymous_wrapper_layer_texture_parent_key_reaches_named_ancestor() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlAnonymousWrapperParentKeyButton" parent="UIParent">
        <Frames>
            <Frame>
                <Layers><Layer level="OVERLAY">
                    <Texture name="$parentCreature" parentKey="creature"/>
                </Layer></Layers>
            </Frame>
        </Frames>
    </Button></Ui>"#,
        "Button",
    );

    let object_type: String = env
        .eval("return XmlAnonymousWrapperParentKeyButton.creature:GetObjectType()")
        .unwrap();
    assert_eq!(
        object_type, "Texture",
        "layer parentKey inside an anonymous wrapper should attach to the named ancestor used for $parent substitution"
    );

    let state = env.state().borrow();
    let parent_id = state
        .widgets
        .get_id_by_name("XmlAnonymousWrapperParentKeyButton")
        .expect("parent button should exist");
    let parent = state.widgets.get(parent_id).unwrap();
    assert!(
        !parent.children_keys.contains_key("creature"),
        "anonymous-wrapper parentKey aliases should not rewrite the Rust child hierarchy"
    );
}

#[test]
fn runtime_template_anonymous_wrapper_layer_texture_parent_key_reaches_instance() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    register_first_template(
        r#"<Ui><Button name="AnonymousWrapperParentKeyTemplate" virtual="true">
        <Frames>
            <Frame>
                <Layers><Layer level="OVERLAY">
                    <Texture name="$parentCreature" parentKey="creature"/>
                </Layer></Layers>
            </Frame>
        </Frames>
    </Button></Ui>"#,
        "AnonymousWrapperParentKeyTemplate",
        "Button",
    );

    env.exec(
        r#"
        RuntimeAnonymousWrapperButton = CreateFrame("Button", nil, UIParent, "AnonymousWrapperParentKeyTemplate")
        "#,
    )
    .unwrap();

    let object_type: String = env
        .eval("return RuntimeAnonymousWrapperButton.creature:GetObjectType()")
        .unwrap();
    assert_eq!(
        object_type, "Texture",
        "runtime templates should expose anonymous-wrapper layer parentKeys on the created instance"
    );
}

#[test]
fn runtime_template_frame_children_publish_parent_keys_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    register_first_template(
        r#"<Ui><Frame name="MoneyInputLikeTemplate" virtual="true">
        <Frames>
            <EditBox parentKey="gold" name="$parentGold"/>
            <EditBox parentKey="silver" name="$parentSilver"/>
            <EditBox parentKey="copper" name="$parentCopper"/>
        </Frames>
    </Frame></Ui>"#,
        "MoneyInputLikeTemplate",
        "Frame",
    );

    env.exec(
        r#"
        MoneyInputLikeFrame = CreateFrame("Frame", "MoneyInputLikeFrame", UIParent, "MoneyInputLikeTemplate")
        MONEY_INPUT_LIKE_ONLOAD_KEYS = {
            gold = MoneyInputLikeFrame.gold ~= nil,
            silver = MoneyInputLikeFrame.silver ~= nil,
            copper = MoneyInputLikeFrame.copper ~= nil,
        }
        "#,
    )
    .unwrap();

    let children_ready: (bool, bool, bool) = env
        .eval(
            r#"
            return MONEY_INPUT_LIKE_ONLOAD_KEYS.gold,
                MONEY_INPUT_LIKE_ONLOAD_KEYS.silver,
                MONEY_INPUT_LIKE_ONLOAD_KEYS.copper
            "#,
        )
        .unwrap();
    assert_eq!(
        children_ready,
        (true, true, true),
        "runtime template frame children must publish parentKey fields before instance OnLoad reads them"
    );
}

#[test]
fn xml_template_frame_children_publish_parent_keys_before_instance_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    register_first_template(
        r#"<Ui><Frame name="MoneyInputLikeXmlTemplate" virtual="true">
        <Frames>
            <EditBox parentKey="gold" name="$parentGold"/>
            <EditBox parentKey="silver" name="$parentSilver"/>
            <EditBox parentKey="copper" name="$parentCopper"/>
        </Frames>
    </Frame></Ui>"#,
        "MoneyInputLikeXmlTemplate",
        "Frame",
    );

    create_first_frame(
        &env,
        r#"<Ui><Frame name="MoneyInputLikeXmlFrame" parent="UIParent" inherits="MoneyInputLikeXmlTemplate">
        <Scripts>
            <OnLoad>
                MONEY_INPUT_XML_ONLOAD_KEYS = {
                    gold = self.gold ~= nil,
                    silver = self.silver ~= nil,
                    copper = self.copper ~= nil,
                }
            </OnLoad>
        </Scripts>
    </Frame></Ui>"#,
        "Frame",
    );

    let children_ready: (bool, bool, bool) = env
        .eval(
            r#"
            return MONEY_INPUT_XML_ONLOAD_KEYS.gold,
                MONEY_INPUT_XML_ONLOAD_KEYS.silver,
                MONEY_INPUT_XML_ONLOAD_KEYS.copper
            "#,
        )
        .unwrap();
    assert_eq!(
        children_ready,
        (true, true, true),
        "XML template frame children must publish parentKey fields before instance OnLoad reads them"
    );
}

#[test]
fn parent_keyed_wrapper_layer_texture_stays_under_wrapper() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlParentKeyedWrapperFrame" parent="UIParent">
        <Frames>
            <Frame parentKey="IconContainer">
                <Layers><Layer level="OVERLAY">
                    <Texture parentKey="Icon"/>
                </Layer></Layers>
            </Frame>
        </Frames>
    </Frame></Ui>"#,
        "Frame",
    );

    let wrapper_type: String = env
        .eval("return XmlParentKeyedWrapperFrame.IconContainer:GetObjectType()")
        .unwrap();
    let texture_type: String = env
        .eval("return XmlParentKeyedWrapperFrame.IconContainer.Icon:GetObjectType()")
        .unwrap();
    let outer_icon_is_nil: bool = env
        .eval("return XmlParentKeyedWrapperFrame.Icon == nil")
        .unwrap();
    assert_eq!(wrapper_type, "Frame");
    assert_eq!(texture_type, "Texture");
    assert!(
        outer_icon_is_nil,
        "layer parentKey under a parentKeyed anonymous wrapper should not overwrite the outer frame"
    );
}

#[test]
fn test_xml_template_inheritance() {
    clear_templates();
    register_first_template(
        r#"<Ui><Frame name="BaseTemplate" virtual="true"><Size x="100" y="100"/></Frame></Ui>"#,
        "BaseTemplate",
        "Frame",
    );
    register_first_template(
        r#"<Ui><Frame name="DerivedTemplate" virtual="true" inherits="BaseTemplate">
            <Size x="200" y="200"/></Frame></Ui>"#,
        "DerivedTemplate",
        "Frame",
    );
    assert!(get_template("BaseTemplate").is_some());
    let derived = get_template("DerivedTemplate").unwrap();
    assert_eq!(derived.frame.inherits, Some("BaseTemplate".to_string()));
}

#[test]
fn test_env_reinstalls_intrinsic_templates_after_clear() {
    clear_templates();
    let _env = WowLuaEnv::new().unwrap();
    assert!(
        get_template("WoWScrollBox").is_some(),
        "WowLuaEnv::new should restore intrinsic XML templates after a clear"
    );
}

#[test]
fn intrinsic_dropdown_scripts_dispatch_before_style_template_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        XmlIntrinsicDropdownCalls = {}

        XmlIntrinsicDropdownMixin = {}
        function XmlIntrinsicDropdownMixin:OnMouseDown_Intrinsic()
            table.insert(XmlIntrinsicDropdownCalls, "intrinsic")
        end

        XmlStyleDropdownMixin = {}
        function XmlStyleDropdownMixin:OnMouseDown()
            table.insert(XmlIntrinsicDropdownCalls, "style")
        end
    "#,
    )
    .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <DropdownButton name="DropdownButton" intrinsic="true" mixin="XmlIntrinsicDropdownMixin">
                <Scripts>
                    <OnMouseDown method="OnMouseDown_Intrinsic"/>
                </Scripts>
            </DropdownButton>
            <DropdownButton name="XmlStyleDropdownTemplate" virtual="true" mixin="XmlStyleDropdownMixin">
                <Scripts>
                    <OnMouseDown method="OnMouseDown"/>
                </Scripts>
            </DropdownButton>
            <DropdownButton name="XmlConcreteDropdown" parent="UIParent" inherits="XmlStyleDropdownTemplate"/>
        </Ui>"#,
        "TestIntrinsicDropdownScripts",
    );
    let toc_path = dir.path().join("TestIntrinsicDropdownScripts.toc");

    load_addon(&env.loader_env(), &toc_path).expect("addon load should succeed");
    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("XmlConcreteDropdown")
        .expect("concrete dropdown should exist");
    env.fire_script_handler(frame_id, "OnMouseDown", Vec::new())
        .unwrap();

    let calls: String = env
        .eval("return table.concat(XmlIntrinsicDropdownCalls, ',')")
        .unwrap();
    assert_eq!(
        calls, "intrinsic,style",
        "intrinsic dropdown handlers should dispatch before style-template handlers"
    );
}

#[test]
fn sibling_virtual_button_templates_do_not_share_onclick_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("SiblingTemplateCalls = {}").unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <Button name="FirstSiblingButtonTemplate" virtual="true">
                <Scripts>
                    <OnClick>table.insert(SiblingTemplateCalls, "first")</OnClick>
                </Scripts>
            </Button>
            <Button name="SecondSiblingButtonTemplate" virtual="true">
                <Scripts>
                    <OnClick>table.insert(SiblingTemplateCalls, "second")</OnClick>
                </Scripts>
            </Button>
            <Button name="ConcreteSiblingButton" parent="UIParent" inherits="SecondSiblingButtonTemplate"/>
        </Ui>"#,
        "TestSiblingButtonTemplateScripts",
    );
    let toc_path = dir.path().join("TestSiblingButtonTemplateScripts.toc");

    load_addon(&env.loader_env(), &toc_path).expect("addon load should succeed");
    env.exec("ConcreteSiblingButton:Click()").unwrap();

    let calls: String = env
        .eval("return table.concat(SiblingTemplateCalls, ',')")
        .unwrap();
    assert_eq!(
        calls, "second",
        "a concrete frame should inherit only its named template script"
    );
}

#[test]
fn trusted_template_scripts_keep_their_origin_when_addon_constructs_frame() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec("TrustedTemplateMixin = { OnShow = function(self) self.fastSecure = issecure() end }")
        .unwrap();
    register_first_template(
        r#"<Ui><Frame name="TrustedOriginFastTemplate" virtual="true" protected="true" hidden="true" mixin="TrustedTemplateMixin">
            <Scripts><OnShow method="OnShow"/></Scripts>
        </Frame></Ui>"#,
        "TrustedOriginFastTemplate",
        "Frame",
    );
    register_first_template(
        r#"<Ui><Frame name="TrustedOriginFallbackTemplate" virtual="true" protected="true" hidden="true">
            <Scripts><OnShow>self.fallbackSecure = issecure(); self.fallbackRan = true</OnShow></Scripts>
        </Frame></Ui>"#,
        "TrustedOriginFallbackTemplate",
        "Frame",
    );
    let (fast, fallback, caller_still_tainted): (bool, bool, bool) = env
        .eval(
            r#"local function addon()
                local fast = CreateFrame('Frame', nil, UIParent, 'TrustedOriginFastTemplate')
                local fallback = CreateFrame('Frame', nil, UIParent, 'TrustedOriginFallbackTemplate')
                fast:Show()
                fallback:Show()
                return fast.fastSecure == true, fallback.fallbackSecure == true and fallback.fallbackRan == true, not issecure()
            end
            debug.setobjecttaint(addon, 'TemplateConstructorProbe')
            return addon()"#,
        )
        .unwrap();
    assert!(fast, "trusted method-only XML script should remain secure");
    assert!(fallback, "trusted fallback XML script should remain secure");
    assert!(caller_still_tainted, "constructor must retain addon taint");
}

#[test]
fn addon_template_scripts_stay_tainted_when_clean_code_constructs_frame() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "AddonOriginMixin = { OnShow = function(self) self.addonMethodSecure = issecure() end }",
    )
    .unwrap();
    let dir = create_test_addon(
        r#"<Ui>
            <Frame name="AddonOriginFastTemplate" virtual="true" protected="true" hidden="true" mixin="AddonOriginMixin">
                <Scripts><OnShow method="OnShow"/></Scripts>
            </Frame>
            <Frame name="AddonOriginFallbackTemplate" virtual="true" protected="true" hidden="true">
                <Scripts><OnShow>self.addonFallbackSecure = issecure(); self.addonFallbackRan = true</OnShow></Scripts>
            </Frame>
        </Ui>"#,
        "TemplateOriginAddon",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TemplateOriginAddon.toc"),
    )
    .unwrap();
    let (fast, fallback): (bool, bool) = env
        .eval(
            r#"local fast = CreateFrame('Frame', nil, UIParent, 'AddonOriginFastTemplate')
            local fallback = CreateFrame('Frame', nil, UIParent, 'AddonOriginFallbackTemplate')
            fast:Show()
            fallback:Show()
            return fast.addonMethodSecure == false, fallback.addonFallbackSecure == false and fallback.addonFallbackRan == true"#,
        )
        .unwrap();
    assert!(fast, "addon XML method-only script must stay tainted");
    assert!(fallback, "addon XML fallback script must stay tainted");
}

#[test]
fn addon_method_override_stays_tainted_under_trusted_template_script() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        "OverrideOriginMixin = { OnShow = function(self) self.methodSecure = issecure() end }",
    )
    .unwrap();
    register_first_template(
        r#"<Ui><Frame name="OverrideOriginTemplate" virtual="true" protected="true" hidden="true" mixin="OverrideOriginMixin">
            <Scripts><OnShow method="OnShow"/></Scripts>
        </Frame></Ui>"#,
        "OverrideOriginTemplate",
        "Frame",
    );
    let (override_insecure, caller_insecure): (bool, bool) = env
        .eval(
            r#"local function addon()
                local frame = CreateFrame('Frame', nil, UIParent, 'OverrideOriginTemplate')
                frame.OnShow = function(self) self.methodSecure = issecure() end
                frame:Show()
                return frame.methodSecure == false, not issecure()
            end
            debug.setobjecttaint(addon, 'TemplateOverrideProbe')
            return addon()"#,
        )
        .unwrap();
    assert!(
        override_insecure,
        "addon method override must not become trusted"
    );
    assert!(caller_insecure, "constructor must retain addon taint");
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
        "TestSizeTemplate",
        "Frame",
    );
    env.exec(r#"local f = CreateFrame("Frame", "TestWithTemplate", UIParent, "TestSizeTemplate")"#)
        .unwrap();
    assert!(env.eval::<bool>("return TestWithTemplate ~= nil").unwrap());
}

#[test]
fn test_create_frame_method_only_template_script_runs() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        TestMethodOnlyTemplateMixin = {}
        function TestMethodOnlyTemplateMixin:OnLoad()
            self.methodOnlyLoaded = true
        end
    "#,
    )
    .unwrap();

    register_first_template(
        r#"<Ui><Frame name="TestMethodOnlyTemplate" virtual="true" mixin="TestMethodOnlyTemplateMixin">
            <Scripts><OnLoad method="OnLoad"/></Scripts>
        </Frame></Ui>"#,
        "TestMethodOnlyTemplate",
        "Frame",
    );

    env.exec(
        r#"local f = CreateFrame("Frame", "TestMethodOnlyFrame", UIParent, "TestMethodOnlyTemplate")"#,
    )
    .unwrap();

    let loaded: bool = env
        .eval("return TestMethodOnlyFrame.methodOnlyLoaded == true")
        .unwrap();
    assert!(loaded, "method-only template OnLoad should fire");
}
