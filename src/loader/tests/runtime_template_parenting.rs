use super::*;

#[test]
fn test_runtime_template_root_parent_key_registers_on_parent() {
    let t = load_test_xml(
        "runtime-root-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <ScrollFrame name="RuntimeRootScrollTemplate" parentKey="ScrollContainer" mixin="RuntimeRootScrollMixin" virtual="true">
                <ScrollChild>
                    <Frame parentKey="Child"/>
                </ScrollChild>
                <Scripts>
                    <OnLoad method="OnLoad"/>
                </Scripts>
            </ScrollFrame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            RuntimeRootScrollMixin = {
                OnLoad = function(self)
                    self.loaded = true
                end,
            }
            local host = CreateFrame("Frame", "RuntimeRootHost", UIParent)
            local scroll = CreateFrame("ScrollFrame", nil, host, "RuntimeRootScrollTemplate")
            assert(host.ScrollContainer == scroll, "root template parentKey should register the runtime instance on its parent")
            assert(scroll.Child ~= nil, "root template scroll child should still be created")
            assert(scroll.loaded == true, "root template OnLoad should still run")
        "#,
        )
        .unwrap();
}

#[test]
fn test_action_button_updates_use_registry_frame_refs_for_anonymous_buttons() {
    let t = load_test_xml(
        "runtime-anon-action-button-registry-ref",
        "<Ui xmlns=\"http://www.blizzard.com/wow/ui/\"/>",
    );

    t.env
        .exec(
            r#"
            __test_button = CreateFrame("Button", nil, UIParent)
            rawset(__test_button, "UpdateState", function(self)
                self.updateCalls = (self.updateCalls or 0) + 1
            end)
            SetActionUIButton(__test_button, 1)
        "#,
        )
        .unwrap();

    t.assert_lua_true(
        "return __test_button ~= nil",
        "anonymous action button should stay reachable",
    );

    {
        let mut lua = t.env.rilua_mut();
        crate::lua_api::globals::action_bar_api::push_action_button_state_update(
            t.env.state(),
            &mut lua,
        )
        .unwrap();
    }

    t.assert_lua_true(
        "__test_button.updateCalls == 1",
        "anonymous action button UpdateState should run through registry frame refs",
    );
}

#[test]
fn test_anonymous_runtime_button_parent_key_attaches_to_parent() {
    let t = load_test_xml(
        "runtime-anon-button-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="AnonymousButtonTemplate" virtual="true">
                <Frames>
                    <Button parentKey="ActionButton"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local frame = CreateFrame("Frame", "AnonymousButtonParentKeyProbe", UIParent, "AnonymousButtonTemplate")
            assert(frame.ActionButton ~= nil, "anonymous runtime button parentKey should attach to the parent")
            assert(frame.ActionButton:GetObjectType() == "Button", "anonymous runtime child should be a button")
        "#,
        )
        .unwrap();
}

#[test]
fn test_empty_direct_script_overrides_inherited_template_script() {
    let t = load_test_xml(
        "runtime-empty-script-clears-inherited",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="EmptyScriptBaseTemplate" virtual="true">
                <Scripts>
                    <OnUpdate>
                        EMPTY_SCRIPT_BASE_CALLED = true
                    </OnUpdate>
                </Scripts>
            </Frame>
            <Frame name="EmptyScriptOverrideFrame" parent="UIParent" inherits="EmptyScriptBaseTemplate">
                <Scripts>
                    <OnUpdate></OnUpdate>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.assert_lua_true(
        "return EmptyScriptOverrideFrame:GetScript('OnUpdate') == nil",
        "an explicitly empty script should clear the inherited handler",
    );
}

#[test]
fn test_empty_script_override_preserves_intrinsic_onupdate_binding() {
    let t = load_test_xml(
        "runtime-empty-script-keeps-intrinsic",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="EmptyScriptIntrinsicTemplate" virtual="true">
                <Scripts>
                    <OnUpdate intrinsicOrder="postcall">
                        EMPTY_SCRIPT_INTRINSIC_CALLED = true
                    </OnUpdate>
                </Scripts>
            </Frame>
            <Frame name="EmptyScriptIntrinsicOverrideFrame" parent="UIParent" inherits="EmptyScriptIntrinsicTemplate">
                <Scripts>
                    <OnUpdate></OnUpdate>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.assert_lua_true(
        "return EmptyScriptIntrinsicOverrideFrame:GetScript('OnUpdate') == nil",
        "empty XML OnUpdate should leave the normal binding clear",
    );

    t.env.fire_on_update(0.016).unwrap();

    t.assert_lua_true(
        "return EMPTY_SCRIPT_INTRINSIC_CALLED == true",
        "empty XML OnUpdate should not clear the inherited intrinsic binding",
    );
}

#[test]
fn test_runtime_template_mixin_and_key_values_apply() {
    let t = load_test_xml(
        "runtime-template-mixin-keyvalues",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="RuntimeTemplateTest" virtual="true" mixin="RuntimeTemplateMixin">
                <KeyValues>
                    <KeyValue key="myString" value="hello"/>
                    <KeyValue key="myNumber" value="42" type="number"/>
                    <KeyValue key="myBool" value="true" type="boolean"/>
                    <KeyValue key="myGlobal" value="RuntimeTemplateGlobals.Token" type="global"/>
                </KeyValues>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            RuntimeTemplateGlobals = { Token = "ready" }
            RuntimeTemplateMixin = {
                Describe = function(self)
                    return self.myString .. ":" .. tostring(self.myNumber) .. ":" .. tostring(self.myBool) .. ":" .. self.myGlobal
                end,
            }

            local frame = CreateFrame("Frame", "RuntimeTemplateInstance", UIParent, "RuntimeTemplateTest")
            assert(frame.myString == "hello", "string key value should apply")
            assert(frame.myNumber == 42, "numeric key value should apply")
            assert(frame.myBool == true, "boolean key value should apply")
            assert(frame.myGlobal == "ready", "dotted global key value should resolve")
            assert(frame:Describe() == "hello:42:true:ready", "mixin method should apply")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_template_method_scripts_apply() {
    let t = load_test_xml(
        "runtime-template-method-scripts",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="RuntimeMethodScriptTemplate" virtual="true" mixin="RuntimeMethodScriptMixin">
                <Scripts>
                    <OnLoad method="RuntimeMethodScriptTemplate_OnLoad"/>
                    <OnEvent method="RuntimeMethodScriptTemplate_OnEvent"/>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            RuntimeMethodScriptMixin = {
                RuntimeMethodScriptTemplate_OnLoad = function(self)
                    self.loadedByMethodScript = true
                end,
                RuntimeMethodScriptTemplate_OnEvent = function(self, event, payload)
                    self.lastMethodEvent = event .. ":" .. tostring(payload)
                end,
            }

            local frame = CreateFrame("Frame", "RuntimeMethodScriptFrame", UIParent, "RuntimeMethodScriptTemplate")
            assert(frame.loadedByMethodScript == true, "OnLoad method script should run")

            local onEvent = frame:GetScript("OnEvent")
            assert(type(onEvent) == "function", "OnEvent method script should be installed")
            onEvent(frame, "TEST_EVENT", "payload")
            assert(frame.lastMethodEvent == "TEST_EVENT:payload", "method script should dispatch through frame method")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_template_parent_array_registers_instance_on_parent() {
    let t = load_test_xml(
        "runtime-template-parent-array",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="RuntimeParentArrayTemplate" virtual="true" parentArray="Slots"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local parent = CreateFrame("Frame", "RuntimeParentArrayParent", UIParent)
            local child = CreateFrame("Frame", "RuntimeParentArrayChild", parent, "RuntimeParentArrayTemplate")
            assert(type(parent.Slots) == "table", "runtime template parentArray should create parent table")
            assert(parent.Slots[1] == child, "runtime template parentArray should register frame instance")
            assert(#parent.Slots == 1, "runtime template parentArray should not duplicate the same child")
        "#,
        )
        .unwrap();
}

#[test]
fn test_anonymous_runtime_template_parent_array_registers_instance_on_parent() {
    let t = load_test_xml(
        "runtime-template-parent-array-anonymous",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="RuntimeAnonymousParentArrayTemplate" virtual="true" parentArray="Slots"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local parent = CreateFrame("Frame", "RuntimeAnonymousParentArrayParent", UIParent)
            local child = CreateFrame("Frame", nil, parent, "RuntimeAnonymousParentArrayTemplate")
            assert(type(parent.Slots) == "table", "anonymous runtime template parentArray should create parent table")
            assert(parent.Slots[1] == child, "anonymous runtime template parentArray should register frame instance")
            assert(#parent.Slots == 1, "anonymous runtime template parentArray should not duplicate the same child")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_create_frame_inherited_child_parent_array_registers_once_in_order() {
    let t = load_test_xml(
        "runtime-create-frame-inherited-child-parent-array",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="InheritedParentArrayTemplate" virtual="true" parentArray="Tabs"/>
            <Frame name="ContainerTemplate" virtual="true">
                <Frames>
                    <Frame name="$parentTab1" inherits="InheritedParentArrayTemplate"/>
                    <Frame name="$parentTab2" inherits="InheritedParentArrayTemplate"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local frame = CreateFrame("Frame", "RuntimeParentArrayContainer", UIParent, "ContainerTemplate")
            assert(type(frame.Tabs) == "table", "runtime template child parentArray should create parent table")
            assert(#frame.Tabs == 2, "runtime template child parentArray should not duplicate inherited children")
            assert(frame.Tabs[1] == RuntimeParentArrayContainerTab1, "first runtime inherited child should keep first slot")
            assert(frame.Tabs[2] == RuntimeParentArrayContainerTab2, "second runtime inherited child should keep second slot")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_named_child_inherited_layers_use_child_name_for_parent_substitution() {
    let t = load_test_xml(
        "runtime-child-layer-name-subst",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="NamedBackgroundTemplate" virtual="true">
                <Layers>
                    <Layer>
                        <Texture name="$parentBackground" parentKey="Background"/>
                    </Layer>
                </Layers>
            </Frame>
            <Frame name="ChildHostTemplate" virtual="true">
                <Frames>
                    <Frame name="$parentButtonFrame" inherits="NamedBackgroundTemplate" parentKey="buttonFrame"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local host = CreateFrame("Frame", "RuntimeLayerNameHost", UIParent, "ChildHostTemplate")
            assert(host.buttonFrame ~= nil, "runtime template should create the named child frame")
            assert(host.Background == nil, "child inherited layers must not leak onto the outer parent name")
            assert(RuntimeLayerNameHostBackground == nil, "outer parent should not receive the child background global")
            assert(RuntimeLayerNameHostButtonFrameBackground ~= nil, "child inherited layer should use the child frame name")
            assert(host.buttonFrame.Background == RuntimeLayerNameHostButtonFrameBackground, "parentKey attachment should point at the child-named background")
            assert(RuntimeLayerNameHostButtonFrameBackground:GetParent() == host.buttonFrame, "child-named background should stay parented to the child frame")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_nested_wrapper_onload_can_publish_texture_to_named_ancestor() {
    let t = load_test_xml(
        "runtime-nested-wrapper-texture-parent",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="RuntimeBossButtonTemplate" virtual="true">
                <Frames>
                    <Frame>
                        <Layers>
                            <Layer>
                                <Texture name="$parentCreature" parentKey="creature"/>
                            </Layer>
                        </Layers>
                        <Scripts>
                            <OnLoad>
                                self:GetParent().wrapperLoaded = true
                                self:GetParent().creature = self.creature
                            </OnLoad>
                        </Scripts>
                    </Frame>
                </Frames>
            </Button>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local button = CreateFrame("Button", "RuntimeBossButton", UIParent, "RuntimeBossButtonTemplate")
            assert(button.wrapperLoaded == true, "nested wrapper OnLoad should run")
            assert(button.creature ~= nil, "nested wrapper OnLoad should publish creature texture onto button")
            assert(button.creature:GetParent():GetParent() == button, "published texture should stay under wrapper child")
        "#,
        )
        .unwrap();
}

#[test]
fn runtime_template_child_onload_can_publish_self_to_parent() {
    let t = load_test_xml(
        "runtime-template-child-self-parent-publication",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="RuntimeChildOnLoadTemplate" virtual="true">
                <Frames>
                    <Button parentKey="Toggle">
                        <Scripts>
                            <OnLoad>
                                self:GetParent().toggle = self;
                                self:RegisterForClicks("LeftButtonUp", "RightButtonUp");
                            </OnLoad>
                        </Scripts>
                    </Button>
                </Frames>
            </Button>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local button = CreateFrame("Button", "RuntimeChildOnLoadHost", UIParent, "RuntimeChildOnLoadTemplate")
            assert(button.toggle == button.Toggle, "child OnLoad should publish itself to its parent")
            "#,
        )
        .unwrap();
}

#[test]
fn runtime_template_child_onload_uses_local_self_for_dotted_value() {
    let t = load_test_xml(
        "runtime-template-child-dotted-self-parent-publication",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="RuntimeChildDottedSelfTemplate" virtual="true">
                <Frames>
                    <Button parentKey="Toggle">
                        <Frames>
                            <Frame parentKey="Part"/>
                        </Frames>
                        <Scripts>
                            <OnLoad>self:GetParent().publishedPart = self.Part</OnLoad>
                        </Scripts>
                    </Button>
                </Frames>
            </Button>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local previous = _G.self
            _G.self = { Part = "global sentinel" }
            local button = CreateFrame("Button", "RuntimeChildDottedSelfHost", UIParent, "RuntimeChildDottedSelfTemplate")
            _G.self = previous
            assert(button.publishedPart == button.Toggle.Part, "child OnLoad must use its local self field")
            "#,
        )
        .unwrap();
}

#[test]
fn runtime_template_child_onload_preserves_dotted_global_value() {
    let t = load_test_xml(
        "runtime-template-child-dotted-global-parent-publication",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="RuntimeChildDottedGlobalTemplate" virtual="true">
                <Frames>
                    <Button parentKey="Toggle">
                        <Scripts>
                            <OnLoad>self:GetParent().publishedGlobal = RuntimeLiteralFixture.Part</OnLoad>
                        </Scripts>
                    </Button>
                </Frames>
            </Button>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            RuntimeLiteralFixture = { Part = "global marker" }
            local button = CreateFrame("Button", "RuntimeChildDottedGlobalHost", UIParent, "RuntimeChildDottedGlobalTemplate")
            assert(button.publishedGlobal == RuntimeLiteralFixture.Part, "genuine global path must remain available")
            "#,
        )
        .unwrap();
}

#[test]
fn test_synthetic_ui_theme_container_intrinsic_applies_theme_mixin() {
    let t = load_test_xml(
        "synthetic-theme-container-intrinsic",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="PlainHost"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            UIThemeContainerMixin = { UpdateTheme = function(self) self.themeUpdated = true end }
            local frame = CreateFrame("Frame", "RuntimeThemeContainer", UIParent, "UIThemeContainerFrame")
            assert(type(frame.UpdateTheme) == "function", "UIThemeContainerFrame intrinsic should apply UIThemeContainerMixin")
            frame:UpdateTheme()
            assert(frame.themeUpdated == true, "UpdateTheme should dispatch on synthetic UIThemeContainerFrame")
        "#,
        )
        .unwrap();
}

#[test]
fn test_xml_inherited_parent_array_registers_children_once_in_order() {
    let t = load_test_xml(
        "xml-inherited-parent-array-once",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="InheritedParentArrayTemplate" virtual="true" parentArray="Tabs"/>
            <Frame name="InheritedParentArrayHost">
                <Frames>
                    <Frame name="$parentTab1" inherits="InheritedParentArrayTemplate"/>
                    <Frame name="$parentTab2" inherits="InheritedParentArrayTemplate"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(type(InheritedParentArrayHost.Tabs) == "table", "xml inherited parentArray should create parent table")
            assert(#InheritedParentArrayHost.Tabs == 2, "xml inherited parentArray should not duplicate inherited children")
            assert(InheritedParentArrayHost.Tabs[1] == InheritedParentArrayHostTab1, "first inherited child should keep first slot")
            assert(InheritedParentArrayHost.Tabs[2] == InheritedParentArrayHostTab2, "second inherited child should keep second slot")
        "#,
        )
        .unwrap();
}

#[test]
fn test_anonymous_runtime_scroll_template_parent_key_attaches_to_parent() {
    let t = load_test_xml(
        "runtime-scroll-parent-key-anonymous",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <ScrollFrame name="RuntimeScrollContainerTemplate" parentKey="ScrollContainer" virtual="true">
                <ScrollChild>
                    <Frame parentKey="Child"/>
                </ScrollChild>
            </ScrollFrame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local parent = CreateFrame("Frame", "RuntimeScrollParent", UIParent)
            local scroll = CreateFrame("ScrollFrame", nil, parent, "RuntimeScrollContainerTemplate")
            assert(parent.ScrollContainer == scroll, "anonymous runtime scroll template parentKey should attach the scroll frame on the parent")
            assert(scroll.Child ~= nil, "anonymous runtime scroll template should create and wire its scroll child")
        "#,
        )
        .unwrap();
}

#[test]
fn test_anonymous_top_level_xml_frame_with_parent_key_attaches_to_parent() {
    let t = load_test_xml(
        "top-level-anon-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="TopLevelAnonParentHost" parent="UIParent"/>
            <Frame parent="TopLevelAnonParentHost" parentKey="AttachedChild"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(TopLevelAnonParentHost.AttachedChild ~= nil, "anonymous top-level frame with explicit parent should attach via parentKey")
            assert(TopLevelAnonParentHost.AttachedChild:GetParent() == TopLevelAnonParentHost, "anonymous top-level frame should stay parented to explicit parent")
        "#,
        )
        .unwrap();
}

#[test]
fn test_child_onload_sees_seeded_parent_array() {
    let t = load_test_xml(
        "xml-parent-array-onload",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="ParentArrayOnLoadHost">
                <Frames>
                    <Frame name="ParentArrayOnLoadChild" parentArray="Buttons">
                        <Scripts>
                            <OnLoad>
                                local parent = self:GetParent()
                                parent.childSawButtonsTable = type(parent.Buttons) == "table"
                            </OnLoad>
                        </Scripts>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    let saw_buttons_table: bool = t
        .env
        .eval("return ParentArrayOnLoadHost.childSawButtonsTable == true")
        .unwrap();
    assert!(
        saw_buttons_table,
        "child OnLoad should see a seeded parentArray table on the parent"
    );
}

#[test]
fn test_xml_animation_parent_keys_attach_group_and_child_animation() {
    let t = load_test_xml(
        "xml-animation-parent-keys",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="AnimParentKeyFrame">
                <Animations>
                    <AnimationGroup parentKey="pulseAnim" looping="BOUNCE">
                        <Alpha parentKey="AlphaAnim" fromAlpha=".75" toAlpha=".2" duration="0.5236"/>
                    </AnimationGroup>
                </Animations>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(type(AnimParentKeyFrame.pulseAnim) == "table", "AnimationGroup parentKey should attach on the frame")
            assert(type(AnimParentKeyFrame.pulseAnim.AlphaAnim) == "table", "child animation parentKey should attach on the animation group")
            assert(type(AnimParentKeyFrame.pulseAnim.Play) == "function", "animation group should keep methods")
            assert(type(AnimParentKeyFrame.pulseAnim.AlphaAnim.SetFromAlpha) == "function", "child animation should keep methods")
        "#,
        )
        .unwrap();
}
