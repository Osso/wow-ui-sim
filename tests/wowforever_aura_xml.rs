//! Aura XML aliases must reach the ordinary frame/template/script pipeline.
#![cfg(any(feature = "client-wowforever", feature = "retail-12-1-0"))]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-wowforever")]
#[test]
fn native_target_aura_callback_roundtrips_before_cleanup() {
    use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen_with_overrides;
    use wow_ui_sim::screen::ScreenKind;

    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    let closure = discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui,
        ScreenKind::Game,
        &["Blizzard_UnitFrame"],
        &[],
    );
    let mut loaded = Vec::new();
    let mut warnings = Vec::new();
    // Load manually: the shared closure loader discards successful-load warnings.
    // Do not settle startup or apply post-load cleanup before inspecting definitions.
    for (name, toc) in closure {
        assert!(
            !name.contains("EnvironmentCleanup"),
            "diagnostic closure must stop before cleanup: {name}; loaded={loaded:?}"
        );
        let result = load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| {
            panic!("loading {name}: {error}; loaded={loaded:?}; warnings={warnings:?}")
        });
        warnings.extend(
            result
                .warnings
                .into_iter()
                .map(|warning| format!("{name}: {warning}")),
        );
        loaded.push(name);
    }
    let template = wow_ui_sim::xml::get_template("TargetFrameAuraContainerTemplate")
        .map(|entry| (entry.name, entry.widget_type));
    let result: Result<(bool, String), _> = env.eval(
        r#"
        local diagnostics = {}
        local frame
        local setter = 'SetAuraContainerAnchorsChangedCallback'
        local getter = 'GetAuraContainerAnchorsChangedCallback'
        local function describe(label, value)
            local members = {}
            if type(value) == 'table' then
                for key, member in pairs(value) do
                    if type(member) == 'function' then members[#members + 1] = tostring(key) end
                end
                table.sort(members)
            end
            diagnostics[#diagnostics + 1] = label .. ': type=' .. type(value)
                .. ', setter=' .. type(value and value[setter])
                .. ', getter=' .. type(value and value[getter])
                .. ', functions=[' .. table.concat(members, ',') .. ']'
        end
        local function snapshot(phase)
            for _, name in ipairs({
                'TargetFrameAuraContainerSharedMixin',
                'TargetFrameAuraContainerInboundMixin',
                'TargetFrameAuraContainerPrivateMixin',
            }) do
                describe(phase .. '.public.' .. name, rawget(_G, name))
                describe(phase .. '.secure.' .. name,
                    __secureenv and rawget(__secureenv, name))
            end
        end
        snapshot('before-template')
        local ok, failure = xpcall(function()
            frame = CreateFrame('AuraContainer', nil, UIParent, 'TargetFrameAuraContainerTemplate')
            local private = GetForbiddenObjectTable(frame)
            assert(type(frame[setter]) == 'function', 'public callback setter missing')
            assert(type(frame[getter]) == 'function', 'public callback getter missing')
            assert(private ~= frame, 'public and forbidden views must remain distinct')
            local callback = function() end
            frame:SetAuraContainerAnchorsChangedCallback(callback)
            assert(frame:GetAuraContainerAnchorsChangedCallback() == callback)
            assert(private.auraContainerAnchorsChangedCallback == callback,
                'setter did not store callback in forbidden partition')
            assert(frame.auraContainerAnchorsChangedCallback == nil,
                'private callback storage leaked into public fields')
            frame:SetAuraContainerAnchorsChangedCallback(nil)
            assert(frame:GetAuraContainerAnchorsChangedCallback() == nil)
            assert(private.auraContainerAnchorsChangedCallback == nil)
        end, debug.traceback)
        snapshot('after-template')
        describe('instance.public', frame)
        describe('instance.forbidden', frame and GetForbiddenObjectTable(frame))
        if failure then diagnostics[#diagnostics + 1] = tostring(failure) end
        return ok, table.concat(diagnostics, '\n')
        "#,
    );
    let lua_errors = env.state().borrow().lua_errors.clone();
    let (passed, diagnostics) = result.unwrap_or_else(|error| {
        panic!(
            "diagnostic evaluation failed: {error}; template={template:?}; \
             loaded={loaded:?}; warnings={warnings:?}; lua_errors={lua_errors:?}"
        )
    });
    assert!(
        passed,
        "native target aura callback failed before cleanup; template={template:?}; \
         loaded={loaded:?}; warnings={warnings:?}; lua_errors={lua_errors:?}\n{diagnostics}"
    );
}

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
