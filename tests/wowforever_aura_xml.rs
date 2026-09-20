//! Aura XML aliases must reach the ordinary frame/template/script pipeline.
#![cfg(any(feature = "client-wowforever", feature = "retail-12-1-0"))]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn aura_tags_create_nested_frames_with_mixin_scripts_and_factory_aliases() {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("AuraXmlProbe.toc");
    std::fs::write(&toc, "## Title: AuraXmlProbe\nProbe.xml\n").unwrap();
    std::fs::write(root.path().join("Probe.xml"), r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                AuraXmlProbeMixin = {}
                function AuraXmlProbeMixin:OnLoad()
                    self:SetSize(42, 24)
                    self:RegisterEvent("PLAYER_LOGIN")
                    self.loaded = true
                end
                function AuraXmlProbeMixin:OnUpdate(elapsed)
                    self.elapsed = elapsed
                end
            </Script>
            <AuraContainer name="AuraXmlContainerTemplate" virtual="true" mixin="AuraXmlProbeMixin">
                <Scripts><OnLoad method="OnLoad"/><OnUpdate method="OnUpdate"/></Scripts>
            </AuraContainer>
            <ManagedAuraContainer name="AuraXmlManagedTemplate" virtual="true" mixin="AuraXmlProbeMixin">
                <Scripts><OnLoad method="OnLoad"/></Scripts>
            </ManagedAuraContainer>
            <AuraButton name="AuraXmlButtonTemplate" virtual="true" mixin="AuraXmlProbeMixin">
                <Scripts><OnLoad method="OnLoad"/></Scripts>
            </AuraButton>
            <Frame name="AuraXmlParent" parent="UIParent">
                <Frames>
                    <AuraContainer parentKey="Auras" inherits="AuraXmlContainerTemplate"/>
                    <ManagedAuraContainer parentKey="Managed" inherits="AuraXmlManagedTemplate"/>
                    <AuraButton parentKey="Button" inherits="AuraXmlButtonTemplate"/>
                </Frames>
            </Frame>
        </Ui>
    "#).unwrap();
    let result = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    env.exec(r#"
        for _, key in ipairs({'Auras', 'Managed', 'Button'}) do
            local child = assert(AuraXmlParent[key])
            assert(child.loaded and child:GetParent() == AuraXmlParent)
            assert(child:GetWidth() == 42 and child:GetHeight() == 24)
            assert(child:IsEventRegistered('PLAYER_LOGIN'))
        end
        AuraXmlParent.Button:SetText('aura')
        assert(AuraXmlParent.Button:GetText() == 'aura')
        for _, kind in ipairs({'AuraContainer', 'ManagedAuraContainer', 'AuraButton'}) do
            local template = ({AuraContainer='AuraXmlContainerTemplate',
                ManagedAuraContainer='AuraXmlManagedTemplate', AuraButton='AuraXmlButtonTemplate'})[kind]
            local frame = CreateFrame(kind, nil, AuraXmlParent, template)
            assert(frame.loaded and frame:GetWidth() == 42)
        end
    "#).unwrap();
    env.fire_on_update(0.125).unwrap();
    env.exec("assert(AuraXmlParent.Auras.elapsed == 0.125)")
        .unwrap();
}

#[test]
fn scoped_aura_template_dispatches_private_scripts_to_forbidden_partition() {
    let env = WowLuaEnv::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("PartitionedAura.toc");
    std::fs::write(&toc, "## Title: PartitionedAura\nProbe.xml\n").unwrap();
    // TargetFrameAuraContainer.xml's ScopedModifier/Mixins/script structure,
    // with concrete fixture behavior instead of its full unit/aura subsystem.
    std::fs::write(
        root.path().join("Probe.xml"),
        r#"
        <Ui xmlns="http://www.blizzard.com/wow/ui/">
            <Script>
                AuraPartitionPrivate = {}
                function AuraPartitionPrivate:OnLoad()
                    self:RegisterEvent("PLAYER_LOGIN")
                    self.loaded = true
                end
                function AuraPartitionPrivate:OnUpdate(elapsed)
                    self.elapsed = elapsed
                end
            </Script>
            <ScopedModifier useForbiddenObjectTable="true">
                <AuraContainer name="AuraPartitionTemplate" virtual="true">
                    <Mixins><Mixin key="AuraPartitionPrivate" source="secure"/></Mixins>
                    <Scripts><OnLoad method="OnLoad"/><OnUpdate method="OnUpdate"/></Scripts>
                </AuraContainer>
            </ScopedModifier>
            <Frame name="AuraPartitionParent" parent="UIParent">
                <Frames><AuraContainer parentKey="Auras" inherits="AuraPartitionTemplate"/></Frames>
            </Frame>
        </Ui>
    "#,
    )
    .unwrap();
    let result = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    env.exec(
        r#"
        local frame = assert(AuraPartitionParent.Auras)
        assert(frame:GetParent() == AuraPartitionParent)
        assert(frame:IsEventRegistered('PLAYER_LOGIN'))
        assert(GetForbiddenObjectTable(frame).loaded == true)
        assert(frame.loaded == nil)
    "#,
    )
    .unwrap();
    env.fire_on_update(0.25).unwrap();
    env.exec("assert(GetForbiddenObjectTable(AuraPartitionParent.Auras).elapsed == 0.25)")
        .unwrap();
}
