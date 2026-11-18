use super::*;

#[test]
fn test_fontstring_template_inherits_apply_mixin_methods() {
    let t = load_test_xml(
        "fontstring-template-mixin-inherits",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                TestFontStringElementMixin = {};
                function TestFontStringElementMixin:Init(value)
                    self.initializedValue = value;
                end
            </Script>
            <FontString name="TestFontStringElementTemplate" mixin="TestFontStringElementMixin" virtual="true"/>
            <Frame name="FontStringTemplateMixinParent">
                <Layers>
                    <Layer level="OVERLAY">
                        <FontString parentKey="Text" inherits="TestFontStringElementTemplate"/>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(FontStringTemplateMixinParent.Text.Init ~= nil, "inherited FontString mixin should provide Init")
            FontStringTemplateMixinParent.Text:Init("from-template")
            assert(FontStringTemplateMixinParent.Text.initializedValue == "from-template", "inherited FontString mixin should run")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_action_button_template_creates_named_children() {
    let t = load_test_xml(
        "runtime-action-button-template",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Cooldown name="CooldownFrameTemplate" hidden="true" setAllPoints="true" virtual="true"/>
            <CheckButton name="ActionButtonTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="TextOverlayContainer">
                        <Size x="10" y="11"/>
                        <Anchors>
                            <Anchor point="CENTER"/>
                        </Anchors>
                        <Scripts>
                            <OnLoad>self.loaded = true;</OnLoad>
                        </Scripts>
                    </Frame>
                    <Cooldown name="$parentCooldown" parentKey="cooldown" inherits="CooldownFrameTemplate" id="17">
                        <Anchors>
                            <Anchor point="TOPLEFT"/>
                            <Anchor point="BOTTOMRIGHT"/>
                        </Anchors>
                    </Cooldown>
                </Frames>
            </CheckButton>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local button = CreateFrame("CheckButton", "ActionButtonFastPath", UIParent, "ActionButtonTemplate")
            assert(button.TextOverlayContainer ~= nil, "TextOverlayContainer should exist")
            assert(button.TextOverlayContainer.loaded == true, "child OnLoad should fire")
            assert(button.cooldown ~= nil, "cooldown child should exist")
            assert(ActionButtonFastPathCooldown == button.cooldown, "named cooldown global should resolve")
            assert(button.cooldown:GetParent() == button, "cooldown parent should be button")
            assert(button.cooldown:GetID() == 17, "cooldown xml id should be preserved")
            assert(not button.cooldown:IsShown(), "inherited hidden cooldown should stay hidden")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_spellfx_template_creates_nested_inherited_children() {
    let t = load_test_xml(
        "runtime-spellfx-template",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="ActionButtonInterruptTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Highlight" hidden="true">
                        <Size x="7" y="8"/>
                        <Anchors>
                            <Anchor point="CENTER"/>
                        </Anchors>
                        <Scripts>
                            <OnLoad>self.loaded = true;</OnLoad>
                        </Scripts>
                    </Frame>
                </Frames>
            </Frame>
            <Frame name="ActionButtonCastingAnimFrameTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Fill">
                        <Size x="9" y="10"/>
                        <Anchors>
                            <Anchor point="CENTER"/>
                        </Anchors>
                        <Scripts>
                            <OnLoad>self.loaded = true;</OnLoad>
                        </Scripts>
                    </Frame>
                </Frames>
            </Frame>
            <CheckButton name="ActionButtonSpellFXTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="InterruptDisplay" inherits="ActionButtonInterruptTemplate" hidden="true"/>
                    <Frame parentKey="SpellCastAnimFrame" inherits="ActionButtonCastingAnimFrameTemplate" hidden="true"/>
                </Frames>
            </CheckButton>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local button = CreateFrame("CheckButton", "SpellFXFastPathButton", UIParent, "ActionButtonSpellFXTemplate")
            assert(button.InterruptDisplay ~= nil, "InterruptDisplay should exist")
            assert(button.SpellCastAnimFrame ~= nil, "SpellCastAnimFrame should exist")
            assert(not button.InterruptDisplay:IsShown(), "inherited hidden flag should be preserved")
            assert(not button.SpellCastAnimFrame:IsShown(), "spell cast child should inherit hidden state")

            assert(button.InterruptDisplay.Highlight ~= nil, "nested interrupt child should exist")
            assert(button.InterruptDisplay.Highlight.loaded == true, "nested interrupt OnLoad should fire")
            assert(button.InterruptDisplay.Highlight:GetParent() == button.InterruptDisplay, "nested interrupt child parent should match")

            assert(button.SpellCastAnimFrame.Fill ~= nil, "nested casting child should exist")
            assert(button.SpellCastAnimFrame.Fill.loaded == true, "nested casting OnLoad should fire")
            assert(button.SpellCastAnimFrame.Fill:GetParent() == button.SpellCastAnimFrame, "nested casting child parent should match")
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_button_template_inherits_texture_slots() {
    let t = load_test_xml(
        "runtime-button-template-textures",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="InheritedTextureButtonTemplate" virtual="true">
                <NormalTexture atlas="ui-questtrackerbutton-secondary-collapse" useAtlasSize="true"/>
                <PushedTexture atlas="ui-questtrackerbutton-secondary-collapse-pressed" useAtlasSize="true"/>
            </Button>
            <Frame name="TextureButtonHolder">
                <Frames>
                    <Button parentKey="TemplateButton" inherits="InheritedTextureButtonTemplate"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local button = TextureButtonHolder.TemplateButton
            assert(button ~= nil, "template child button should exist")
            assert(button:GetNormalTexture() ~= nil, "template normal texture should exist")
            assert(button:GetPushedTexture() ~= nil, "template pushed texture should exist")
            "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_template_partitioned_child_binds_private_method_handlers() {
    // INFERRED as native Wrath behavior; tests the modeled generic XML dispatcher.
    assert_partitioned_child_handlers("Frame");
}

#[cfg(feature = "aura-xml-widgets")]
#[test]
fn test_runtime_aura_container_partitioned_child_binds_private_method_handlers() {
    assert_partitioned_child_handlers("AuraContainer");
}

fn assert_partitioned_child_handlers(widget_type: &str) {
    let xml = r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PartitionedAuraPrivateMixin = {}
                function PartitionedAuraPrivateMixin:OnLoad_Intrinsic()
                    self:RegisterEvent("PLAYER_LOGIN")
                    self.loaded = true
                end
                function PartitionedAuraPrivateMixin:OnUpdate_Intrinsic(elapsed)
                    self.elapsed = elapsed
                end

                PartitionedAuraInboundMixin = {}
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <AuraContainer name="PartitionedAuraContainerTemplate" virtual="true">
                    <Mixins>
                        <Mixin key="PartitionedAuraInboundMixin" source="secure" targetPartition="public" inboundPartition="forbidden" secureDelegates="true"/>
                        <Mixin key="PartitionedAuraPrivateMixin" source="secure"/>
                    </Mixins>
                    <Scripts>
                        <OnLoad method="OnLoad_Intrinsic"/>
                        <OnUpdate method="OnUpdate_Intrinsic"/>
                    </Scripts>
                </AuraContainer>
            </ScopedModifier>
            <Frame name="PartitionedAuraContainerParent" parent="UIParent">
                <Frames>
                    <AuraContainer parentKey="Auras" inherits="PartitionedAuraContainerTemplate"/>
                </Frames>
            </Frame>
        </Ui>
        "#;
    let xml = xml
        .replace("<AuraContainer", &format!("<{widget_type}"))
        .replace("</AuraContainer>", &format!("</{widget_type}>"));
    let t = load_test_xml("runtime-template-partitioned-private-method-handlers", &xml);

    t.assert_lua_true(
        "return PartitionedAuraContainerParent.Auras ~= nil",
        "concrete inherited child should remain reachable through parentKey",
    );
    t.assert_lua_true(
        "return PartitionedAuraContainerParent.Auras:GetParent() == PartitionedAuraContainerParent",
        "concrete inherited child should remain parented",
    );
    t.assert_lua_true(
        "return (__secureenv.GetForbiddenObjectTable or GetForbiddenObjectTable)(PartitionedAuraContainerParent.Auras).loaded == true",
        "private OnLoad method should run with forbidden self",
    );
    t.assert_lua_true(
        "return PartitionedAuraContainerParent.Auras:IsEventRegistered('PLAYER_LOGIN')",
        "private OnLoad should dispatch frame methods through the public frame",
    );

    t.env.fire_on_update(0.016).unwrap();

    t.assert_lua_true(
        "return (__secureenv.GetForbiddenObjectTable or GetForbiddenObjectTable)(PartitionedAuraContainerParent.Auras).elapsed == 0.016",
        "private OnUpdate method should run with forbidden self",
    );
}

#[test]
fn test_xml_partitioned_intrinsic_onload_uses_forbidden_self() {
    let t = load_test_xml(
        "xml-partitioned-intrinsic-onload-forbidden-self",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PartitionedIntrinsicPrivateMixin = {}
                function PartitionedIntrinsicPrivateMixin:OnLoad_Intrinsic()
                    self.intrinsicLoaded = true
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <Frame name="PartitionedIntrinsicFrame" parent="UIParent">
                    <Mixins>
                        <Mixin key="PartitionedIntrinsicPrivateMixin" source="secure"/>
                    </Mixins>
                    <Scripts>
                        <OnLoad/>
                    </Scripts>
                </Frame>
            </ScopedModifier>
        </Ui>
        "#,
    );

    t.assert_lua_true(
        "return type((__secureenv.GetForbiddenObjectTable or GetForbiddenObjectTable)(PartitionedIntrinsicFrame).OnLoad_Intrinsic) == 'function'",
        "private intrinsic method should be installed on forbidden frame table",
    );
    t.assert_lua_true(
        "return PartitionedIntrinsicFrame.intrinsicLoaded == nil",
        "private intrinsic OnLoad should not write to public frame",
    );
    t.assert_lua_true(
        "return (__secureenv.GetForbiddenObjectTable or GetForbiddenObjectTable)(PartitionedIntrinsicFrame).intrinsicLoaded == true",
        "private intrinsic OnLoad should run with forbidden self",
    );
}

#[test]
fn test_runtime_template_creates_inherited_layer_regions() {
    let t = load_test_xml(
        "runtime-template-layer-regions",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="HeaderTemplate" virtual="true">
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Text" justifyH="LEFT" maxLines="1"/>
                    </Layer>
                </Layers>
            </Frame>
            <Frame name="ContainerTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Header" inherits="HeaderTemplate"/>
                </Frames>
            </Frame>
            <Frame name="ContainerInstance" inherits="ContainerTemplate"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(ContainerInstance.Header ~= nil, "template child frame should exist")
            assert(ContainerInstance.Header.Text ~= nil, "inherited fontstring should exist")
            "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_create_frame_creates_inherited_layer_regions() {
    let t = load_test_xml(
        "runtime-create-frame-template-layer-regions",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="HeaderTemplate" virtual="true">
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Text" justifyH="LEFT" maxLines="1"/>
                    </Layer>
                </Layers>
            </Frame>
            <Frame name="ContainerTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Header" inherits="HeaderTemplate"/>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(r#"CreateFrame("Frame", "RuntimeTemplateLayerFrame", UIParent, "ContainerTemplate")"#)
        .unwrap();

    {
        let state = t.env.state().borrow();
        let parent_id = state
            .widgets
            .get_id_by_name("RuntimeTemplateLayerFrame")
            .expect("runtime frame should exist");
        let parent = state
            .widgets
            .get(parent_id)
            .expect("runtime frame should be registered");
        assert!(
            parent.children_keys.contains_key("Header"),
            "runtime template should assign Header in Rust children_keys"
        );
    }

    t.env
        .exec(
            r#"
            local frame = RuntimeTemplateLayerFrame
            assert(rawget(frame, "Header") ~= nil, "template child frame should exist in raw table")
            assert(frame.Header ~= nil, "template child frame should exist")
            assert(frame.Header.Text ~= nil, "inherited fontstring should exist")
            "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_create_frame_keeps_inherited_checkbutton_parent_key() {
    let t = load_test_xml(
        "runtime-create-frame-inherited-checkbutton-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <CheckButton name="ChatConfigBaseCheckButtonTemplate" virtual="true">
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Text" justifyH="LEFT" maxLines="1"/>
                    </Layer>
                </Layers>
            </CheckButton>
            <CheckButton name="ChatConfigCheckButtonTemplate" parentKey="CheckButton" inherits="ChatConfigBaseCheckButtonTemplate" virtual="true"/>
            <Frame name="ChatConfigCheckboxTemplate" virtual="true">
                <Frames>
                    <CheckButton name="$parentCheck" parentKey="CheckButton" inherits="ChatConfigCheckButtonTemplate"/>
                </Frames>
            </Frame>
            <Frame name="ChatConfigWideCheckboxWithSwatchTemplate" inherits="ChatConfigCheckboxTemplate" virtual="true">
                <Scripts>
                    <OnLoad inherit="prepend">
                        assert(self.CheckButton ~= nil, "template OnLoad should see inherited CheckButton child")
                        assert(self.CheckButton.Text ~= nil, "template OnLoad should see inherited CheckButton Text")
                    </OnLoad>
                </Scripts>
            </Frame>
            <Frame name="MovableChatConfigWideCheckboxWithSwatchTemplate" parentArray="WideCheckboxes" mixin="ChatConfigWideCheckboxMixin" inherits="ChatConfigWideCheckboxWithSwatchTemplate" virtual="true">
                <Scripts>
                    <OnLoad inherit="prepend" method="OnLoad"/>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            ChatConfigWideCheckboxMixin = {}
            function ChatConfigWideCheckboxMixin:OnLoad()
                assert(self.CheckButton ~= nil, "mixin OnLoad should see inherited CheckButton child")
                assert(self.CheckButton.Text ~= nil, "mixin OnLoad should see inherited CheckButton Text")
            end
            "#,
        )
        .unwrap();

    t.env
        .exec(
            r#"
            CreateFrame("Frame", "RuntimeChatCheckboxFrame", UIParent, "MovableChatConfigWideCheckboxWithSwatchTemplate")
            assert(RuntimeChatCheckboxFrame.CheckButton ~= nil, "runtime frame should keep inherited CheckButton child")
            assert(RuntimeChatCheckboxFrame.CheckButton.Text ~= nil, "runtime frame should keep inherited CheckButton Text child")
            assert(RuntimeChatCheckboxFrameCheck == RuntimeChatCheckboxFrame.CheckButton, "named CheckButton child should remain globally addressable")
            "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_template_layers_can_anchor_to_template_child_parent_keys() {
    let t = load_test_xml(
        "runtime-template-layers-anchor-to-template-child-parent-keys",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <CheckButton name="ChatConfigBaseCheckButtonTemplate" motionScriptsWhileDisabled="true" virtual="true">
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Text" justifyH="LEFT" maxLines="1"/>
                    </Layer>
                </Layers>
            </CheckButton>
            <CheckButton name="ChatConfigCheckButtonTemplate" parentKey="CheckButton" inherits="ChatConfigBaseCheckButtonTemplate" virtual="true"/>
            <Frame name="ChatConfigCheckboxTemplate" virtual="true">
                <Frames>
                    <CheckButton name="$parentCheck" parentKey="CheckButton" inherits="ChatConfigCheckButtonTemplate"/>
                </Frames>
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="BlankText">
                            <Anchors>
                                <Anchor point="LEFT" relativeKey="$parent.CheckButton.Text" relativePoint="LEFT" x="0" y="0"/>
                            </Anchors>
                        </FontString>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
        "#,
    );

    let before_errors = t.env.state().borrow().lua_errors.len();
    t.env
        .exec(
            r#"
            local frame = CreateFrame("Frame", "RuntimeAnchoredCheckboxFrame", UIParent, "ChatConfigCheckboxTemplate")
            assert(frame.CheckButton ~= nil, "runtime frame should expose its template CheckButton child")
            assert(frame.BlankText ~= nil, "runtime frame should create its anchored fontstring")
            "#,
        )
        .unwrap();
    let after_errors = t.env.state().borrow().lua_errors.clone();
    let targeted: Vec<_> = after_errors
        .into_iter()
        .skip(before_errors)
        .filter(|message| message.contains("CheckButton"))
        .collect();
    assert!(
        targeted.is_empty(),
        "runtime template creation should not emit CheckButton errors for sibling anchors: {targeted:?}"
    );
}

#[test]
fn test_rebuild_anchor_index_resolves_relative_key_to_late_layer_parent_key() {
    let t = load_test_xml(
        "anchor-index-rebuild-resolves-late-layer-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="AnchorOrderFrame">
                <Frames>
                    <Frame parentKey="LoadSystem">
                        <Anchors>
                            <Anchor point="LEFT" relativeKey="$parent.BottomBar" relativePoint="LEFT" x="48" y="0"/>
                        </Anchors>
                    </Frame>
                </Frames>
                <Layers>
                    <Layer level="BACKGROUND">
                        <Texture parentKey="BottomBar">
                            <Size x="400" y="30"/>
                            <Anchors>
                                <Anchor point="BOTTOM"/>
                            </Anchors>
                        </Texture>
                    </Layer>
                </Layers>
            </Frame>
        </Ui>
        "#,
    );

    t.env.state().borrow_mut().widgets.rebuild_anchor_index();

    t.env
        .exec(
            r#"
            assert(AnchorOrderFrame.BottomBar ~= nil, "expected BottomBar layer parentKey to be attached on parent frame")
            local point, relativeTo, relativePoint, x, y = AnchorOrderFrame.LoadSystem:GetPoint(1)
            assert(point == "LEFT", "expected LEFT anchor point")
            assert(relativeTo == AnchorOrderFrame.BottomBar, "expected LoadSystem to anchor to BottomBar after anchor index rebuild")
            assert(relativePoint == "LEFT", "expected LEFT relative point")
            assert(x == 48 and y == 0, "expected anchor offsets to be preserved")
            "#,
        )
        .unwrap();
}

#[test]
fn test_xml_instance_keeps_inherited_checkbutton_parent_key() {
    let t = load_test_xml(
        "xml-instance-inherited-checkbutton-parent-key",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                ChatConfigWideCheckboxMixin = {}
                function ChatConfigWideCheckboxMixin:OnLoad()
                    assert(self.CheckButton ~= nil, "xml mixin OnLoad should see inherited CheckButton child")
                    assert(self.CheckButton.Text ~= nil, "xml mixin OnLoad should see inherited CheckButton Text")
                end
            </Script>
            <CheckButton name="ChatConfigBaseCheckButtonTemplate" virtual="true">
                <Layers>
                    <Layer level="ARTWORK">
                        <FontString parentKey="Text" justifyH="LEFT" maxLines="1"/>
                    </Layer>
                </Layers>
            </CheckButton>
            <CheckButton name="ChatConfigCheckButtonTemplate" parentKey="CheckButton" inherits="ChatConfigBaseCheckButtonTemplate" virtual="true"/>
            <Frame name="ChatConfigCheckboxTemplate" virtual="true">
                <Frames>
                    <CheckButton name="$parentCheck" parentKey="CheckButton" inherits="ChatConfigCheckButtonTemplate"/>
                </Frames>
            </Frame>
            <Frame name="ChatConfigWideCheckboxWithSwatchTemplate" inherits="ChatConfigCheckboxTemplate" virtual="true">
                <Scripts>
                    <OnLoad inherit="prepend">
                        assert(self.CheckButton ~= nil, "xml template OnLoad should see inherited CheckButton child")
                        assert(self.CheckButton.Text ~= nil, "xml template OnLoad should see inherited CheckButton Text")
                    </OnLoad>
                </Scripts>
            </Frame>
            <Frame name="MovableChatConfigWideCheckboxWithSwatchTemplate" parentArray="WideCheckboxes" mixin="ChatConfigWideCheckboxMixin" inherits="ChatConfigWideCheckboxWithSwatchTemplate" virtual="true">
                <Scripts>
                    <OnLoad inherit="prepend" method="OnLoad"/>
                </Scripts>
            </Frame>
            <Frame name="XmlChatCheckboxFrame" inherits="MovableChatConfigWideCheckboxWithSwatchTemplate"/>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(XmlChatCheckboxFrame.CheckButton ~= nil, "xml frame should keep inherited CheckButton child")
            assert(XmlChatCheckboxFrame.CheckButton.Text ~= nil, "xml frame should keep inherited CheckButton Text child")
            assert(XmlChatCheckboxFrameCheck == XmlChatCheckboxFrame.CheckButton, "xml named CheckButton child should remain globally addressable")
            "#,
        )
        .unwrap();
}

post_wrath_ui_fixture! {
#[test]
fn test_catalog_shop_file_prefix_numeric_error_probe() {
    let env = WowLuaEnv::new().expect("lua env");
    env.set_screen_size(1024.0, 768.0);
    // VERIFIED — R88g.3.q4: seterrorhandler and debug are ordinary-environment members. Evidence: pr46-49-decomp-answers.md, sections R88g.3.q4.
    env.exec(
        r#"
        __catalog_shop_probe_traces = {}
        local original_handler = geterrorhandler()
        seterrorhandler(function(msg)
            table.insert(__catalog_shop_probe_traces, tostring(msg) .. "\n" .. debug.traceback())
            if original_handler then
                return original_handler(msg)
            end
        end)
        "#,
    )
    .expect("should install catalog shop probe handler");

    let ui = blizzard_ui_dir();
    let addons = crate::loader::discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if matches!(
            name.as_str(),
            "Blizzard_CatalogShop"
                | "Blizzard_CatalogShopTopUpFlow"
                | "Blizzard_CatalogShopRefundFlow"
        ) {
            continue;
        }
        let _ = crate::loader::load_addon(&env.loader_env(), toc_path);
    }

    let toc_path = ui.join("Blizzard_CatalogShop/Blizzard_CatalogShop.toc");
    let toc = crate::toc::TocFile::from_file(&toc_path).expect("catalog shop toc");
    let addon_table = env.create_addon_table().expect("catalog shop addon table");
    let ctx = crate::loader::addon::AddonContext {
        name: "Blizzard_CatalogShop",
        table: addon_table,
        addon_root: toc.addon_dir.as_path(),
        use_secure_env: toc.is_secure_env(),
        taint: false,
    };

    for file in toc.file_paths() {
        let before = env.state().borrow().lua_errors.len();
        let ext = file
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        match ext {
            "lua" => {
                super::lua_file::load_lua_file(
                    &env.loader_env(),
                    &file,
                    &ctx,
                    &mut LoadTiming::default(),
                )
                .unwrap_or_else(|error| panic!("{} should load: {error}", file.display()));
            }
            "xml" => {
                super::xml_file::load_xml_file(
                    &env.loader_env(),
                    &file,
                    &ctx,
                    &mut LoadTiming::default(), &mut Vec::new())
                .unwrap_or_else(|error| panic!("{} should load: {error}", file.display()));
            }
            _ => continue,
        }
        let errors = env.state().borrow().lua_errors.clone();
        let targeted: Vec<_> = errors
            .into_iter()
            .skip(before)
            .filter(|message| message.contains("expected number, got nil at argument 1"))
            .collect();
        let traces: Vec<String> = env
            .eval("return __catalog_shop_probe_traces")
            .expect("probe traces should stringify");
        assert!(
            targeted.is_empty(),
            "{} introduced CatalogShop numeric load error: {targeted:?}\ntraces:\n{}",
            file.display(),
            traces.join("\n---\n")
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn test_catalog_shop_xml_numeric_error_without_main_onload() {
    let env = WowLuaEnv::new().expect("lua env");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    let addons = crate::loader::discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if matches!(
            name.as_str(),
            "Blizzard_CatalogShop"
                | "Blizzard_CatalogShopTopUpFlow"
                | "Blizzard_CatalogShopRefundFlow"
        ) {
            continue;
        }
        let _ = crate::loader::load_addon(&env.loader_env(), toc_path);
    }

    let shared_templates_toc =
        ui.join("Blizzard_CatalogShopSharedTemplates/Blizzard_CatalogShopSharedTemplates.toc");
    crate::loader::load_addon(&env.loader_env(), &shared_templates_toc)
        .expect("catalog shop shared templates should load");

    let toc_path = ui.join("Blizzard_CatalogShop/Blizzard_CatalogShop.toc");
    let toc = crate::toc::TocFile::from_file(&toc_path).expect("catalog shop toc");
    let addon_table = env.create_addon_table().expect("catalog shop addon table");
    let ctx = crate::loader::addon::AddonContext {
        name: "Blizzard_CatalogShop",
        table: addon_table,
        addon_root: toc.addon_dir.as_path(),
        use_secure_env: toc.is_secure_env(),
        taint: false,
    };

    for file in toc.file_paths() {
        let ext = file
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default();
        match ext {
            "lua" => {
                super::lua_file::load_lua_file(
                    &env.loader_env(),
                    &file,
                    &ctx,
                    &mut LoadTiming::default(),
                )
                .unwrap_or_else(|error| panic!("{} should load: {error}", file.display()));
            }
            _ => continue,
        }
    }

    env.exec_rilua_secure(
        r#"
        local original = CatalogShopDefaultProductCardMixin.Layout
        CatalogShopDefaultProductCardMixin.Layout = function(self)
            self.productInfo = self.productInfo
            return nil
        end
        __catalog_shop_original_layout = original
        "#,
    )
    .expect("should patch CatalogShop default product card layout");

    let xml_path = ui.join("Blizzard_CatalogShop/Blizzard_CatalogShop.xml");
    let before = env.state().borrow().lua_errors.len();
    super::xml_file::load_xml_file(
        &env.loader_env(),
        &xml_path,
        &ctx,
        &mut LoadTiming::default(), &mut Vec::new())
    .unwrap_or_else(|error| panic!("{} should load: {error}", xml_path.display()));
    let errors = env.state().borrow().lua_errors.clone();
    let targeted: Vec<_> = errors
        .into_iter()
        .skip(before)
        .filter(|message| message.contains("expected number, got nil at argument 1"))
        .collect();
    assert!(
        targeted.is_empty(),
        "CatalogShop xml still introduced numeric load error with product card layout disabled: {targeted:?}"
    );
}
}

#[test]
fn test_layer_region_anchored_to_child_frame_defined_after_layers() {
    // Mirrors RenownCardButtonTemplate (Blizzard_Journeys.xml): a <Layers>
    // FontString anchors via relativeKey to a child frame from the <Frames>
    // section. The loader creates layer regions before child frames, so the
    // key cannot resolve at SetPoint time — it must be stored on the anchor
    // and resolved by the post-children finalize pass. Before that fix the
    // anchor silently fell back to the parent button and the text rendered
    // off the right edge of the card (behind the next column's card).
    let t = load_test_xml(
        "layer-region-keyed-to-child-frame",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Button name="KeyedAnchorCard" parent="UIParent">
                <Size x="374" y="112"/>
                <Anchors>
                    <Anchor point="TOPLEFT"/>
                </Anchors>
                <Frames>
                    <Frame parentKey="IconFrame">
                        <Size x="60" y="60"/>
                        <Anchors>
                            <Anchor point="LEFT" x="20"/>
                        </Anchors>
                    </Frame>
                </Frames>
                <Layers>
                    <Layer level="OVERLAY">
                        <FontString parentKey="Name" justifyH="LEFT">
                            <Size x="225" y="20"/>
                            <Anchors>
                                <Anchor point="LEFT" relativeKey="$parent.IconFrame" relativePoint="RIGHT" x="5" y="5"/>
                            </Anchors>
                        </FontString>
                    </Layer>
                </Layers>
            </Button>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local card = KeyedAnchorCard
            local point, relTo, relPoint, x = card.Name:GetPoint(1)
            assert(relTo == card.IconFrame,
                "Name must anchor to IconFrame, got " .. tostring(relTo == card and "the card" or relTo))
            assert(point == "LEFT" and relPoint == "RIGHT" and x == 5, "anchor shape preserved")
            local expected = card.IconFrame:GetRight() + 5
            assert(math.abs(card.Name:GetLeft() - expected) < 0.01,
                ("Name left %s should be IconFrame right + 5 (%s)"):format(card.Name:GetLeft(), expected))
        "#,
        )
        .unwrap();
}
