use super::*;

#[test]
fn test_runtime_minimal_scrollbar_avoids_lua_createframe_for_nested_thumb() {
    let t = load_test_xml(
        "runtime-minimal-scrollbar-direct-grandchildren",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <EventFrame name="MinimalScrollBar" virtual="true">
                <Frames>
                    <Frame parentKey="Track">
                        <Frames>
                            <EventButton parentKey="Thumb">
                                <Scripts>
                                    <OnLoad>self.loaded = true;</OnLoad>
                                </Scripts>
                            </EventButton>
                        </Frames>
                    </Frame>
                </Frames>
            </EventFrame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local originalCreateFrame = CreateFrame
            local createCount = 0
            CreateFrame = function(...)
                createCount = createCount + 1
                return originalCreateFrame(...)
            end

            local scrollbar = CreateFrame("EventFrame", "MinimalScrollBarFastPath", UIParent, "MinimalScrollBar")
            assert(scrollbar.Track ~= nil, "Track child should exist")
            assert(scrollbar.Track.Thumb ~= nil, "Thumb grandchild should exist")
            assert(scrollbar.Track.Thumb.loaded == true, "Thumb OnLoad should fire")
            assert(createCount == 1, "nested thumb should avoid Lua CreateFrame fallback, got " .. createCount)
        "#,
        )
        .unwrap();
}

#[test]
fn test_runtime_template_anchor_keeps_direct_offset_attributes() {
    let t = load_test_xml(
        "runtime-template-direct-offset",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="DirectOffsetTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Child">
                        <Size x="10" y="10"/>
                        <Anchors>
                            <Anchor point="BOTTOMLEFT" relativePoint="BOTTOMLEFT">
                                <Offset x="19" y="-30"/>
                            </Anchor>
                        </Anchors>
                    </Frame>
                </Frames>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local parent = CreateFrame("Frame", "DirectOffsetTemplateParent", UIParent, "DirectOffsetTemplate")
            local point, relativeTo, relativePoint, x, y = parent.Child:GetPoint(1)
            assert(point == "BOTTOMLEFT", "point=" .. tostring(point))
            assert(relativePoint == "BOTTOMLEFT", "relativePoint=" .. tostring(relativePoint))
            assert(x == 19, "x=" .. tostring(x))
            assert(y == -30, "y=" .. tostring(y))
        "#,
        )
        .unwrap();
}

#[test]
fn test_anonymous_runtime_template_uses_registry_frame_refs_without_global_alias() {
    let t = load_test_xml(
        "runtime-anon-template-registry-ref",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Frame name="AnonymousTemplate" virtual="true">
                <Frames>
                    <Frame parentKey="Child">
                        <Scripts>
                            <OnLoad>self.loaded = true;</OnLoad>
                        </Scripts>
                    </Frame>
                </Frames>
                <Scripts>
                    <OnLoad>self.loaded = true;</OnLoad>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            __test_frame = CreateFrame("Frame", nil, UIParent, "AnonymousTemplate")
            assert(__test_frame.loaded == true, "anonymous template OnLoad should fire")
            assert(__test_frame.Child ~= nil, "anonymous template child should exist")
            assert(__test_frame.Child.loaded == true, "anonymous template child OnLoad should fire")
        "#,
        )
        .unwrap();

    t.assert_lua_true(
        "return __test_frame ~= nil and __test_frame.Child ~= nil",
        "anonymous runtime template frame should stay reachable",
    );
}

#[test]
fn test_partitioned_mixins_use_forbidden_object_table() {
    let t = load_test_xml(
        "partitioned-mixins-use-forbidden-object-table",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PartitionedInboundMixin = {};
                function PartitionedInboundMixin:ReadSecret()
                    return self.secret;
                end
                function PartitionedInboundMixin:WriteSecret(value)
                    self.secret = value;
                end

                PartitionedPrivateMixin = {};
                function PartitionedPrivateMixin:PrivateReadSecret()
                    return self.secret;
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <Frame name="PartitionedFrame" parent="UIParent">
                    <KeyValues>
                        <KeyValue key="secret" value="hidden" type="string"/>
                    </KeyValues>
                    <Mixins>
                        <Mixin key="PartitionedInboundMixin" source="secure" targetPartition="public" inboundPartition="forbidden" secureDelegates="true"/>
                        <Mixin key="PartitionedPrivateMixin" source="secure"/>
                    </Mixins>
                </Frame>
            </ScopedModifier>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(type(GetForbiddenObjectTable) == "function", "forbidden object table helper should exist")
            local forbidden = GetForbiddenObjectTable(PartitionedFrame)
            assert(type(forbidden) == "table", "forbidden partition should be a table")
            assert(forbidden ~= PartitionedFrame, "forbidden partition should be distinct from public frame")
            assert(PartitionedFrame.secret == nil, "KeyValues should not leak onto public partition")
            assert(forbidden.secret == "hidden", "KeyValues should land on forbidden partition")
            assert(PartitionedFrame.PrivateReadSecret == nil, "private mixin should not be installed on public partition")
            assert(type(forbidden.PrivateReadSecret) == "function", "private mixin should install on forbidden partition")
            assert(PartitionedFrame:ReadSecret() == "hidden", "public delegate should call with forbidden self")
            PartitionedFrame:WriteSecret("changed")
            assert(forbidden.secret == "changed", "delegate writes should update forbidden partition")
            assert(PartitionedFrame.secret == nil, "delegate writes should not leak onto public partition")
        "#,
        )
        .unwrap();
}

#[test]
fn test_plain_literal_mixins_block_still_applies_to_public_frame() {
    let t = load_test_xml(
        "plain-literal-mixins-block",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PlainLiteralMixin = {};
                function PlainLiteralMixin:MarkPlain()
                    self.plainApplied = true;
                end
            </Script>
            <Frame name="PlainLiteralMixinFrame" parent="UIParent">
                <Mixins>
                    <Mixin key="PlainLiteralMixin"/>
                </Mixins>
            </Frame>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            assert(type(PlainLiteralMixinFrame.MarkPlain) == "function", "plain literal Mixins block should apply to public frame")
            PlainLiteralMixinFrame:MarkPlain()
            assert(PlainLiteralMixinFrame.plainApplied == true, "plain literal mixin should run with public self")
        "#,
        )
        .unwrap();
}

#[test]
fn test_partitioned_runtime_template_uses_forbidden_object_table() {
    let t = load_test_xml(
        "partitioned-runtime-template-use-forbidden-object-table",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PartitionedTemplateInboundMixin = {};
                function PartitionedTemplateInboundMixin:ReadSecret()
                    return self.secret;
                end

                PartitionedTemplatePrivateMixin = {};
                function PartitionedTemplatePrivateMixin:PrivateReadSecret()
                    return self.secret;
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <Frame name="PartitionedRuntimeTemplate" virtual="true">
                    <KeyValues>
                        <KeyValue key="secret" value="template-hidden" type="string"/>
                    </KeyValues>
                    <Mixins>
                        <Mixin key="PartitionedTemplateInboundMixin" source="secure" targetPartition="public" inboundPartition="forbidden" secureDelegates="true"/>
                        <Mixin key="PartitionedTemplatePrivateMixin" source="secure"/>
                    </Mixins>
                </Frame>
            </ScopedModifier>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local frame = CreateFrame("Frame", "PartitionedRuntimeFrame", UIParent, "PartitionedRuntimeTemplate")
            local forbidden = GetForbiddenObjectTable(frame)
            assert(frame.secret == nil, "runtime template KeyValues should not leak onto public partition")
            assert(forbidden.secret == "template-hidden", "runtime template KeyValues should land on forbidden partition")
            assert(type(frame.ReadSecret) == "function", "runtime template inbound mixin should expose public delegate")
            assert(frame.PrivateReadSecret == nil, "runtime template private mixin should not be public")
            assert(type(forbidden.PrivateReadSecret) == "function", "runtime template private mixin should be forbidden")
            assert(frame:ReadSecret() == "template-hidden", "runtime template delegate should use forbidden self")
        "#,
        )
        .unwrap();
}

#[test]
fn test_frame_literal_mixins_block_applies_mixins() {
    let t = load_test_xml(
        "frame-literal-mixins-block",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                LiteralAuraContainerInboundMixin = {};
                function LiteralAuraContainerInboundMixin:MarkInbound()
                    self.inboundApplied = true;
                end
                LiteralAuraContainerPrivateMixin = {};
                function LiteralAuraContainerPrivateMixin:MarkPrivate()
                    self.privateApplied = true;
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <Frame name="LiteralAuraContainer" parent="UIParent">
                    <Mixins>
                        <Mixin key="LiteralAuraContainerInboundMixin" source="secure" targetPartition="public" inboundPartition="forbidden" secureDelegates="true"/>
                        <Mixin key="LiteralAuraContainerPrivateMixin" source="secure"/>
                    </Mixins>
                </Frame>
            </ScopedModifier>
            <Script>
                local _, addon = ...
                addon.AddonLocalDirectMixin = {}
                function addon.AddonLocalDirectMixin:OnLoad()
                    self.addonLocalLoaded = true
                end
            </Script>
            <Script>
                TargetFrameInstanceMixin = {}
                function TargetFrameInstanceMixin:OnLoad()
                    self.slotBackedLoaded = true
                end
            </Script>
            <ScopedModifier scriptsUseGivenEnv="true">
                <Script>
                    local _, addon = ...
                    local scoped = setmetatable({}, { __index = addon })
                    scoped._G = scoped
                    setfenv(__wow_xml_lookup_local, scoped)
                    setfenv(__wow_resolve_xml_mixin, scoped)
                </Script>
                <Frame name="AddonLocalDirectMixinFrame" parent="UIParent" mixin="AddonLocalDirectMixin">
                    <KeyValues>
                        <KeyValue key="usesSlowXmlPath" value="true" type="boolean"/>
                    </KeyValues>
                    <Scripts>
                        <OnLoad method="OnLoad"/>
                    </Scripts>
                </Frame>
                <Frame name="SlotBackedDirectMixinFrame" parent="UIParent" mixin="TargetFrameInstanceMixin">
                    <KeyValues>
                        <KeyValue key="usesSlowXmlPath" value="true" type="boolean"/>
                    </KeyValues>
                    <Scripts>
                        <OnLoad method="OnLoad"/>
                    </Scripts>
                </Frame>
            </ScopedModifier>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local forbidden = GetForbiddenObjectTable(LiteralAuraContainer)
            assert(LiteralAuraContainer.MarkInbound ~= nil, "literal Mixins block should apply first mixin")
            assert(LiteralAuraContainer.MarkPrivate == nil, "private secure mixin should not apply to public partition")
            assert(forbidden.MarkPrivate ~= nil, "private secure mixin should apply to forbidden partition")
            LiteralAuraContainer:MarkInbound()
            forbidden:MarkPrivate()
            assert(LiteralAuraContainer.inboundApplied == nil, "inbound delegate should not write to public partition")
            assert(forbidden.inboundApplied == true, "inbound delegate should run with forbidden self")
            assert(forbidden.privateApplied == true, "second literal mixin method should run on forbidden partition")

            local publicMixin = { directPublic = function() end }
            local secureMixin = { directSecure = function() end }
            __secureMixinMethods = {
                [publicMixin] = { applyPublic = function(self) self.publicApplied = true end },
            }
            __secureenv.__secureMixinMethods = {
                [secureMixin] = { applySecure = function(self) self.secureApplied = true end },
            }

            local composite = CreateFromMixins(publicMixin, secureMixin)
            assert(type(composite.applyPublic) == "function", "public secure registry methods should materialize into composite mixins")
            assert(type(composite.applySecure) == "function", "secure registry methods should materialize into composite mixins")
            assert(composite.directPublic == nil and composite.directSecure == nil, "direct entries added after secure materialization should stay excluded")

            local staticFrame = CreateFrame("Frame", "LiteralSecureStaticMixinFrame", UIParent)
            __wow_apply_xml_mixin(staticFrame, publicMixin)
            __wow_apply_xml_mixin(staticFrame, secureMixin)
            staticFrame:applyPublic()
            staticFrame:applySecure()
            assert(staticFrame.publicApplied == true and staticFrame.secureApplied == true, "static XML mixin application should resolve both secure registries")
            assert(staticFrame.directPublic == nil and staticFrame.directSecure == nil, "static XML mixin application should exclude direct entries")

            assert(rawget(_G, "AddonLocalDirectMixin") == nil, "addon-local mixin should not leak into the public globals")
            assert(AddonLocalDirectMixinFrame.addonLocalLoaded == true, "static XML should resolve an ordinary direct mixin from the loading addon table before lifecycle dispatch")
            assert(SlotBackedDirectMixinFrame.slotBackedLoaded == true, "static XML should resolve a slot-backed public mixin after resolver functions receive a scoped fenv")
        "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "client-retail")]
fn test_blizzard_help_plate_method_bindings_do_not_report_self_as_global() {
    let env = WowLuaEnv::new().expect("HelpPlate environment should initialize");
    let toc_path = blizzard_ui_dir().join("Blizzard_HelpPlate/Blizzard_HelpPlate.toc");

    let result = load_addon(&env.loader_env(), &toc_path).expect("HelpPlate should load");

    assert!(
        !result.nil_symbol_observations.iter().any(|observation| {
            matches!(
                &observation.kind,
                crate::loader::NilSymbolObservationKind::Global { name } if name == "self"
            )
        }),
        "XML method self parameter must not be classified as a global: {:?}",
        result.nil_symbol_observations
    );
    let global_self_type: String = env
        .eval("return type(rawget(_G, 'self'))")
        .expect("global self state should be readable");
    assert_eq!(
        global_self_type, "nil",
        "lifecycle dispatch must restore global self"
    );
}

#[test]
fn test_xml_method_script_binds_after_inherited_template_overrides() {
    let t = load_test_xml(
        "xml-method-binding-after-template-overrides",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                XmlTemplateMethodBindingResult = nil
                function XmlTemplateMethodBindingBase(self)
                    XmlTemplateMethodBindingResult = "base"
                end
                function XmlTemplateMethodBindingOverride(self)
                    XmlTemplateMethodBindingResult = "override"
                end
            </Script>
            <Frame name="XmlMethodBindingBaseTemplate" virtual="true">
                <KeyValues>
                    <KeyValue key="Foo" value="XmlTemplateMethodBindingBase" type="global"/>
                </KeyValues>
                <Scripts>
                    <OnLoad method="Foo"/>
                </Scripts>
            </Frame>
            <Frame name="XmlMethodBindingOverrideTemplate" virtual="true" inherits="XmlMethodBindingBaseTemplate">
                <KeyValues>
                    <KeyValue key="Foo" value="XmlTemplateMethodBindingOverride" type="global"/>
                </KeyValues>
            </Frame>
            <Frame name="XmlMethodBindingOverrideFrame" parent="UIParent" inherits="XmlMethodBindingOverrideTemplate"/>
        </Ui>
        "#,
    );

    t.assert_lua_str("return XmlTemplateMethodBindingResult", "override");
}

#[test]
fn test_partitioned_xml_method_script_uses_forbidden_object_table() {
    let t = load_test_xml(
        "partitioned-xml-method-script-forbidden-self",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                PartitionedMethodPrivateMixin = {}
                function PartitionedMethodPrivateMixin:OnLoad_Intrinsic()
                    self.loadedSecret = true
                end
                function PartitionedMethodPrivateMixin:OnEvent_Intrinsic(event)
                    self.eventSecret = event
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <Frame name="PartitionedMethodFrame" parent="UIParent">
                    <Mixins>
                        <Mixin key="PartitionedMethodPrivateMixin" source="secure"/>
                    </Mixins>
                    <Scripts>
                        <OnLoad method="OnLoad_Intrinsic"/>
                        <OnEvent method="OnEvent_Intrinsic"/>
                    </Scripts>
                </Frame>
            </ScopedModifier>
        </Ui>
        "#,
    );

    t.env
        .exec(
            r#"
            local handler = PartitionedMethodFrame:GetScript("OnEvent")
            assert(type(handler) == "function", "OnEvent method binding should install a handler")
            handler(PartitionedMethodFrame, "XML_METHOD_PROBE")
        "#,
        )
        .unwrap();

    t.assert_lua_true(
        "return PartitionedMethodFrame.loadedSecret == nil",
        "private XML method script should not write to public frame",
    );
    t.assert_lua_true(
        "return GetForbiddenObjectTable(PartitionedMethodFrame).loadedSecret == true",
        "private XML method script should run with forbidden self",
    );
    t.assert_lua_true(
        "return GetForbiddenObjectTable(PartitionedMethodFrame).eventSecret == 'XML_METHOD_PROBE'",
        "private XML event method should run with forbidden self",
    );
}

#[test]
fn test_xml_method_script_handler_storage_is_separate_from_object_field() {
    let t = load_test_xml(
        "xml-method-binding-script-storage",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                XmlMethodStorageLog = {}
                XmlMethodStorageObservations = {}
                XmlMethodStorageMixin = {}
                function XmlMethodStorageMixin:OnHide()
                    table.insert(XmlMethodStorageLog, "hide1")
                end
            </Script>
            <Frame name="XmlMethodStorageFrame" parent="UIParent" mixin="XmlMethodStorageMixin">
                <Scripts>
                    <OnLoad>
                        table.insert(XmlMethodStorageLog, "load")
                        XmlMethodStorageObservations.initialEqual = self:GetScript("OnHide") == self.OnHide
                        self.OnHide = function()
                            table.insert(XmlMethodStorageLog, "hide2")
                        end
                        XmlMethodStorageObservations.afterFieldAssignmentEqual = self:GetScript("OnHide") == self.OnHide
                        self:OnHide()
                        self:SetScript("OnHide", function()
                            table.insert(XmlMethodStorageLog, "hide3")
                        end)
                        XmlMethodStorageObservations.afterSetScriptEqual = self:GetScript("OnHide") == self.OnHide
                        self:Hide()
                    </OnLoad>
                    <OnHide method="OnHide"/>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.assert_lua_str(
        "return table.concat(XmlMethodStorageLog, ',')",
        "load,hide2,hide3",
    );
    t.assert_lua_true(
        "return XmlMethodStorageObservations.initialEqual == true",
        "XML method binding should initially install the same function visible on the object field",
    );
    t.assert_lua_true(
        "return XmlMethodStorageObservations.afterFieldAssignmentEqual == false",
        "object field assignment should not replace the script handler",
    );
    t.assert_lua_true(
        "return XmlMethodStorageObservations.afterSetScriptEqual == false",
        "SetScript should not replace the object field",
    );
}

#[test]
fn test_xml_method_script_binds_before_sibling_onload_mutation() {
    let t = load_test_xml(
        "xml-method-binding-before-execution",
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                XmlMethodBindingLog = {}
                XmlMethodBindingMixin = {}
                function XmlMethodBindingMixin:OnHide()
                    table.insert(XmlMethodBindingLog, "hide1")
                end
            </Script>
            <Frame name="XmlMethodBindingFrame" parent="UIParent" mixin="XmlMethodBindingMixin">
                <Scripts>
                    <OnLoad>
                        table.insert(XmlMethodBindingLog, "load")
                        self.OnHide = function()
                            table.insert(XmlMethodBindingLog, "hide2")
                        end
                        self:Hide()
                    </OnLoad>
                    <OnHide method="OnHide"/>
                </Scripts>
            </Frame>
        </Ui>
        "#,
    );

    t.assert_lua_str(
        "return table.concat(XmlMethodBindingLog, ',')",
        "load,hide1",
    );
}
