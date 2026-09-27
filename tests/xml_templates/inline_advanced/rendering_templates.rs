use super::*;
#[test]
fn test_create_scrollframe_from_xml_registers_scroll_child() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let ui = parse_xml(
        r#"<Ui><ScrollFrame name="XmlScrollFrame" parent="UIParent">
        <Size x="200" y="100"/>
        <Anchors><Anchor point="CENTER"/></Anchors>
        <ScrollChild>
            <Frame parentKey="Child">
                <Size x="320" y="180"/>
                <Anchors><Anchor point="TOPLEFT"/></Anchors>
            </Frame>
        </ScrollChild>
    </ScrollFrame></Ui>"#,
    )
    .unwrap();
    match &ui.elements[0] {
        XmlElement::ScrollFrame(f) => {
            create_frame_from_xml(
                &env.loader_env(),
                f,
                "ScrollFrame",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
        _ => panic!("Expected ScrollFrame element"),
    }

    let matches_child: bool = env
        .eval("return XmlScrollFrame:GetScrollChild() == XmlScrollFrame.Child")
        .unwrap();
    assert!(
        matches_child,
        "XML ScrollChild should be registered as the ScrollFrame's scroll child"
    );
}

#[test]
fn test_button_text_without_parent_key_registers_as_text_fontstring() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let ui = parse_xml(
        r#"<Ui><Button name="XmlButtonText" parent="UIParent">
        <ButtonText name="$parentText"/>
    </Button></Ui>"#,
    )
    .unwrap();
    match &ui.elements[0] {
        XmlElement::Button(f) => {
            create_frame_from_xml(
                &env.loader_env(),
                f,
                "Button",
                None,
                None,
                None,
                &mut LoadTiming::default(),
            )
            .unwrap();
        }
        _ => panic!("Expected Button element"),
    }

    let same_text_region: bool = env
        .eval("return XmlButtonText:GetFontString() == XmlButtonTextText")
        .unwrap();
    assert!(
        same_text_region,
        "ButtonText without an explicit parentKey should still back GetFontString()"
    );
}

#[test]
fn item_button_xml_uses_item_button_intrinsic_template() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <Button name="ItemButton" intrinsic="true">
        <NormalTexture name="$parentIcon" parentKey="icon"/>
        <Layers>
            <Layer level="OVERLAY">
                <Texture parentKey="IconBorder"/>
            </Layer>
        </Layers>
    </Button>
    <ItemButton name="XmlIntrinsicItemButton" parent="UIParent"/>
</Ui>"#,
        "TestItemButtonIntrinsic",
    );
    let toc_path = dir.path().join("TestItemButtonIntrinsic.toc");

    load_addon(&env.loader_env(), &toc_path).expect("addon load should succeed");

    let (has_icon, has_border): (bool, bool) = env
        .eval(
            r#"
            return XmlIntrinsicItemButton.icon ~= nil,
                   XmlIntrinsicItemButton.IconBorder ~= nil
            "#,
        )
        .unwrap();
    assert!(
        has_icon,
        "top-level <ItemButton> should inherit the intrinsic ItemButton icon child"
    );
    assert!(
        has_border,
        "top-level <ItemButton> should inherit the intrinsic ItemButton border child"
    );
}

#[test]
fn inherited_statusbar_bar_texture_creates_live_bar_child() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <StatusBar name="SharedStatusBarTemplate" virtual="true">
        <BarTexture parentKey="Bar" file="Interface\Buttons\WHITE8X8"/>
    </StatusBar>
    <StatusBar name="XmlInheritedStatusBar" parent="UIParent" inherits="SharedStatusBarTemplate"/>
</Ui>"#,
        "TestInheritedStatusBarBarTexture",
    );
    let toc_path = dir.path().join("TestInheritedStatusBarBarTexture.toc");

    load_addon(&env.loader_env(), &toc_path).expect("addon load should succeed");

    let (has_status_bar_texture, has_bar_field): (bool, bool) = env
        .eval(
            r#"
            return XmlInheritedStatusBar:GetStatusBarTexture() ~= nil,
                   XmlInheritedStatusBar.Bar ~= nil
            "#,
        )
        .unwrap();
    assert!(
        has_status_bar_texture,
        "StatusBar should create a live bar texture from inherited <BarTexture>"
    );
    assert!(
        has_bar_field,
        "StatusBar should expose inherited <BarTexture parentKey='Bar'> as .Bar"
    );
}

#[test]
fn xml_statusbar_rotates_texture_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <StatusBar name="XmlRotatedStatusBar" parent="UIParent" rotatesTexture="true">
        <BarTexture file="Interface\Buttons\WHITE8X8"/>
        <Scripts><OnLoad>self.rotationAtLoad = self:GetRotatesTexture(); self.coordsAtLoad = table.concat({self:GetStatusBarTexture():GetTexCoord()}, ",")</OnLoad></Scripts>
    </StatusBar>
</Ui>"#,
        "TestXmlStatusBarRotation",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestXmlStatusBarRotation.toc"),
    )
    .expect("addon load should succeed");

    let (rotates, at_load, coords): (bool, bool, String) = env
        .eval(
            "return XmlRotatedStatusBar:GetRotatesTexture(), XmlRotatedStatusBar.rotationAtLoad, XmlRotatedStatusBar.coordsAtLoad",
        )
        .unwrap();
    assert!(rotates && at_load);
    assert_eq!(coords, "0,1,1,1,0,0,1,0");
}

#[test]
fn xml_statusbar_inherits_rotation_with_explicit_false_override() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <StatusBar name="XmlRotatedTemplate" virtual="true" rotatesTexture="true">
        <BarTexture file="Interface\Buttons\WHITE8X8"/>
        <Scripts><OnLoad>self.rotationAtLoad = self:GetRotatesTexture()</OnLoad></Scripts>
    </StatusBar>
    <StatusBar name="XmlInheritedRotation" parent="UIParent" inherits="XmlRotatedTemplate"/>
    <StatusBar name="XmlDisabledRotation" parent="UIParent" inherits="XmlRotatedTemplate" rotatesTexture="false"/>
</Ui>"#,
        "TestXmlInheritedStatusBarRotation",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestXmlInheritedStatusBarRotation.toc"),
    )
    .expect("addon load should succeed");

    let (inherited, inherited_at_load, inherited_coords, overridden, overridden_at_load, override_coords):
        (bool, bool, String, bool, bool, String) = env
        .eval(
            r#"return XmlInheritedRotation:GetRotatesTexture(), XmlInheritedRotation.rotationAtLoad,
                      table.concat({XmlInheritedRotation:GetStatusBarTexture():GetTexCoord()}, ","),
                      XmlDisabledRotation:GetRotatesTexture(), XmlDisabledRotation.rotationAtLoad,
                      table.concat({XmlDisabledRotation:GetStatusBarTexture():GetTexCoord()}, ",")"#,
        )
        .unwrap();
    assert!(inherited && inherited_at_load);
    assert_eq!(inherited_coords, "0,1,1,1,0,0,1,0");
    assert!(!overridden && !overridden_at_load);
    assert_ne!(override_coords, inherited_coords);
}

#[test]
fn create_frame_statusbar_template_applies_rotation_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <StatusBar name="RuntimeRotatedStatusBarTemplate" virtual="true" rotatesTexture="true">
        <BarTexture file="Interface\Buttons\WHITE8X8"/>
        <Scripts><OnLoad>self.rotationAtLoad = self:GetRotatesTexture(); self.coordsAtLoad = table.concat({self:GetStatusBarTexture():GetTexCoord()}, ",")</OnLoad></Scripts>
    </StatusBar>
</Ui>"#,
        "TestRuntimeStatusBarRotation",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestRuntimeStatusBarRotation.toc"),
    )
    .expect("addon load should succeed");

    let (rotates, at_load, coords): (bool, bool, String) = env
        .eval(
            r#"local bar = CreateFrame("StatusBar", nil, UIParent, "RuntimeRotatedStatusBarTemplate")
               return bar:GetRotatesTexture(), bar.rotationAtLoad, bar.coordsAtLoad"#,
        )
        .unwrap();
    assert!(rotates && at_load);
    assert_eq!(coords, "0,1,1,1,0,0,1,0");
}

#[test]
fn create_frame_statusbar_template_applies_orientation_before_onload() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let dir = create_test_addon(
        r#"<Ui>
    <StatusBar name="RuntimeVerticalBarTemplate" virtual="true" orientation="VERTICAL">
        <Scripts><OnLoad>self.orientationAtLoad = self:GetOrientation()</OnLoad></Scripts>
    </StatusBar>
    <StatusBar name="RuntimeInheritedVerticalBarTemplate" virtual="true" inherits="RuntimeVerticalBarTemplate"/>
    <StatusBar name="RuntimeHorizontalBarTemplate" virtual="true" inherits="RuntimeVerticalBarTemplate" orientation="HORIZONTAL"/>
</Ui>"#,
        "TestRuntimeStatusBarOrientation",
    );
    load_addon(
        &env.loader_env(),
        &dir.path().join("TestRuntimeStatusBarOrientation.toc"),
    )
    .expect("addon load should succeed");

    let (direct, direct_at_load, inherited, inherited_at_load, overridden, overridden_at_load):
        (String, String, String, String, String, String) = env
        .eval(
            r#"local direct = CreateFrame("StatusBar", nil, UIParent, "RuntimeVerticalBarTemplate")
               local inherited = CreateFrame("StatusBar", nil, UIParent, "RuntimeInheritedVerticalBarTemplate")
               local overridden = CreateFrame("StatusBar", nil, UIParent, "RuntimeHorizontalBarTemplate")
               return direct:GetOrientation(), direct.orientationAtLoad,
                      inherited:GetOrientation(), inherited.orientationAtLoad,
                      overridden:GetOrientation(), overridden.orientationAtLoad"#,
        )
        .unwrap();
    assert_eq!(
        (direct.as_str(), direct_at_load.as_str()),
        ("VERTICAL", "VERTICAL")
    );
    assert_eq!(
        (inherited.as_str(), inherited_at_load.as_str()),
        ("VERTICAL", "VERTICAL")
    );
    assert_eq!(
        (overridden.as_str(), overridden_at_load.as_str()),
        ("HORIZONTAL", "HORIZONTAL")
    );
}

#[test]
fn test_create_frame_from_xml_hidden_starts_hidden() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlHiddenFrame" parent="UIParent" hidden="true">
        <Size x="200" y="100"/><Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    let shown: bool = env.eval("return XmlHiddenFrame:IsShown()").unwrap();
    let visible: bool = env.eval("return XmlHiddenFrame:IsVisible()").unwrap();
    let effective_alpha: f32 = env
        .eval("return XmlHiddenFrame:GetEffectiveAlpha()")
        .unwrap();

    assert!(!shown, "hidden XML frame should start with shown=false");
    assert!(!visible, "hidden XML frame should not be visible");
    assert_eq!(
        effective_alpha, 0.0,
        "hidden XML frame should start with effective alpha 0"
    );
}

#[test]
fn test_create_frame_from_xml_hidden_not_in_render_buckets() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="XmlHiddenBucketFrame" parent="UIParent" hidden="true">
        <Size x="200" y="100"/><Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("XmlHiddenBucketFrame")
        .expect("hidden XML frame should exist");
    let buckets = build_strata_buckets(&env);
    let in_buckets = buckets.iter().any(|bucket| bucket.contains(&frame_id));

    assert!(
        !in_buckets,
        "hidden XML frame should never enter visible strata buckets"
    );
}

#[test]
fn test_create_frame_from_xml_with_template() {
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestPanelTemplateUnique" virtual="true">
        <Size x="300" y="200"/>
        <Layers><Layer level="ARTWORK">
            <FontString parentKey="TitleText"><Size x="280" y="20"/>
                <Anchors><Anchor point="TOP" y="-10"/></Anchors>
            </FontString>
        </Layer></Layers>
        <Frames><Button parentKey="CloseButton"><Size x="24" y="24"/>
            <Anchors><Anchor point="TOPRIGHT" x="-5" y="-5"/></Anchors>
        </Button></Frames>
    </Frame></Ui>"#,
        "Frame",
    );
    assert!(get_template("TestPanelTemplateUnique").is_some());

    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestPanelUnique" parent="UIParent"
        inherits="TestPanelTemplateUnique">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    assert_eq!(
        env.eval::<f32>("return TestPanelUnique:GetWidth()")
            .unwrap(),
        300.0
    );
    assert_eq!(
        env.eval::<f32>("return TestPanelUnique:GetHeight()")
            .unwrap(),
        200.0
    );
    assert!(
        env.eval::<bool>("return TestPanelUnique.TitleText ~= nil")
            .unwrap()
    );
    assert!(
        env.eval::<bool>("return TestPanelUnique.CloseButton ~= nil")
            .unwrap()
    );
}

#[test]
fn test_create_frame_from_xml_template_inheritance_chain() {
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestBaseTemplateChain" virtual="true">
        <Size x="100" y="100"/>
        <Layers><Layer level="BACKGROUND">
            <Texture parentKey="Bg" setAllPoints="true"/>
        </Layer></Layers>
    </Frame></Ui>"#,
        "Frame",
    );

    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestDerivedTemplateChain" virtual="true"
        inherits="TestBaseTemplateChain"><Size x="200" y="150"/>
        <Layers><Layer level="ARTWORK">
            <FontString parentKey="Title"><Anchors><Anchor point="TOP" y="-5"/></Anchors></FontString>
        </Layer></Layers>
    </Frame></Ui>"#,
        "Frame",
    );

    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestFinalFrameChain" parent="UIParent"
        inherits="TestDerivedTemplateChain">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    assert_eq!(
        env.eval::<f32>("return TestFinalFrameChain:GetWidth()")
            .unwrap(),
        200.0
    );
    assert_eq!(
        env.eval::<f32>("return TestFinalFrameChain:GetHeight()")
            .unwrap(),
        150.0
    );
    assert!(
        env.eval::<bool>("return TestFinalFrameChain.Bg ~= nil")
            .unwrap()
    );
    assert!(
        env.eval::<bool>("return TestFinalFrameChain.Title ~= nil")
            .unwrap()
    );
}

#[test]
fn test_create_frame_from_xml_inherited_template_mixin_available() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        TestTemplateMixin = {}
        function TestTemplateMixin:GetProbeValue()
            return 42
        end
    "#,
    )
    .unwrap();

    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestMixinTemplate" virtual="true" mixin="TestTemplateMixin">
        <Size x="100" y="50"/>
    </Frame></Ui>"#,
        "Frame",
    );

    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestMixinFrame" parent="UIParent" inherits="TestMixinTemplate">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    let probe_value: i32 = env.eval("return TestMixinFrame:GetProbeValue()").unwrap();
    assert_eq!(probe_value, 42, "template mixin method should be available");
}

#[test]
fn test_create_frame_from_xml_parent_key() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="ParentKeyTestFrame" parent="UIParent">
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
    </Frame></Ui>"#,
        "Frame",
    );

    assert!(
        env.eval::<bool>("return ParentKeyTestFrame.Header ~= nil")
            .unwrap()
    );
    assert!(
        env.eval::<bool>("return ParentKeyTestFrame.Content ~= nil")
            .unwrap()
    );
    assert!(
        env.eval::<bool>("return ParentKeyTestFrame.Header.Title ~= nil")
            .unwrap()
    );

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
    create_first_frame(
        &env,
        r#"<Ui><Button name="XmlTestButton" parent="UIParent" text="Click Me">
        <Size x="120" y="30"/><Anchors><Anchor point="CENTER"/></Anchors>
    </Button></Ui>"#,
        "Button",
    );

    assert!(env.eval::<bool>("return XmlTestButton ~= nil").unwrap());
    assert_eq!(
        env.eval::<String>("return XmlTestButton:GetObjectType()")
            .unwrap(),
        "Button"
    );
    assert_eq!(
        env.eval::<String>("return XmlTestButton:GetText() or ''")
            .unwrap(),
        "Click Me"
    );
}

#[test]
fn test_create_frame_from_xml_with_scripts() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="ScriptTestFrame" parent="UIParent">
        <Size x="100" y="100"/>
        <Scripts><OnLoad>self.loadedFlag = true</OnLoad></Scripts>
    </Frame></Ui>"#,
        "Frame",
    );
    assert!(
        env.eval::<bool>("return ScriptTestFrame.loadedFlag == true")
            .unwrap()
    );
}

#[test]
fn test_create_frame_from_xml_with_keyvalues() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Frame name="KeyValueTestFrame" parent="UIParent">
        <Size x="100" y="100"/>
        <KeyValues>
            <KeyValue key="myString" value="hello" type="string"/>
            <KeyValue key="myNumber" value="42" type="number"/>
            <KeyValue key="myBool" value="true" type="boolean"/>
        </KeyValues>
    </Frame></Ui>"#,
        "Frame",
    );

    assert_eq!(
        env.eval::<String>("return KeyValueTestFrame.myString")
            .unwrap(),
        "hello"
    );
    assert_eq!(
        env.eval::<i32>("return KeyValueTestFrame.myNumber")
            .unwrap(),
        42
    );
    assert!(env.eval::<bool>("return KeyValueTestFrame.myBool").unwrap());
}

/// Count children of a specific widget type under a named frame.
fn count_typed_children(env: &WowLuaEnv, name: &str, wt: wow_ui_sim::widget::WidgetType) -> usize {
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name(name).unwrap();
    let frame = state.widgets.get(id).unwrap();
    frame
        .children
        .iter()
        .filter(|&&cid| state.widgets.get(cid).is_some_and(|c| c.widget_type == wt))
        .count()
}

#[test]
fn test_template_children_not_duplicated() {
    clear_templates();
    let env = WowLuaEnv::new().unwrap();
    create_first_frame(
        &env,
        r#"<Ui><Button name="TestCloseButtonBase" virtual="true">
        <Size x="24" y="24"/></Button></Ui>"#,
        "Button",
    );
    create_first_frame(
        &env,
        r#"<Ui><Button name="TestCloseButtonAnchored" virtual="true"
        inherits="TestCloseButtonBase">
        <Anchors><Anchor point="TOPRIGHT" x="-2" y="-2"/></Anchors>
    </Button></Ui>"#,
        "Button",
    );
    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestPanelTemplate" virtual="true">
        <Size x="400" y="300"/>
        <Frames><Button name="$parentCloseButton" parentKey="CloseButton"
            inherits="TestCloseButtonAnchored"/></Frames>
    </Frame></Ui>"#,
        "Frame",
    );
    create_first_frame(
        &env,
        r#"<Ui><Frame name="TestPanelInstance" parent="UIParent"
        inherits="TestPanelTemplate">
        <Anchors><Anchor point="CENTER"/></Anchors>
    </Frame></Ui>"#,
        "Frame",
    );

    assert!(
        env.eval::<bool>("return TestPanelInstance.CloseButton ~= nil")
            .unwrap()
    );
    let n = count_typed_children(
        &env,
        "TestPanelInstance",
        wow_ui_sim::widget::WidgetType::Button,
    );
    assert_eq!(
        n, 1,
        "Template child Button should be created exactly once, found {n}"
    );

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestPanelInstance").unwrap();
    let frame = state.widgets.get(id).unwrap();
    let btn_id = *frame.children_keys.get("CloseButton").unwrap();
    let btn = state.widgets.get(btn_id).unwrap();
    assert!(
        !btn.anchors.is_empty(),
        "CloseButton should have anchors from template"
    );
}
