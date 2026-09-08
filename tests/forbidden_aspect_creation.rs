#![cfg(feature = "retail-12-1-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_xml_fixture(env: &WowLuaEnv, xml: &str) -> wow_ui_sim::loader::LoadResult {
    let directory = tempfile::tempdir().expect("create fixture directory");
    let toc = directory.path().join("AspectFixture.toc");
    std::fs::write(&toc, "## Title: AspectFixture\nfixture.xml\n").unwrap();
    std::fs::write(directory.path().join("fixture.xml"), xml).unwrap();
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("load aspect fixture")
}

#[test]
fn native_children_inherit_only_hierarchy_aspects_at_creation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local e, path = Enum.ForbiddenAspect, Enum.ScriptObjectPropagationPath
        assert(path.Hierarchy == 0 and path.Layout == 1, 'native propagation path values')
        local meta = Enum.ScriptObjectPropagationPathMeta
        assert(meta.MinValue == 0 and meta.MaxValue == 1 and meta.NumValues == 2)
        local parent = CreateFrame('Frame')
        parent:AddForbiddenAspects(bit.bor(e.UntrustedScriptExecution, e.UntrustedLayoutScriptExecution, e.AlwaysPropagateInput))
        assert(parent:GetInheritableForbiddenAspects(path.Hierarchy) == 44, 'hierarchy path is enum value zero')
        assert(parent:GetInheritableForbiddenAspects(path.Layout) == 8, 'only layout restrictions follow anchors by default')
        local frame = CreateFrame('Frame', nil, parent)
        local children = {frame, parent:CreateTexture(), parent:CreateFontString(), parent:CreateMaskTexture(), parent:CreateLine()}
        for _, child in ipairs(children) do
            assert(child:GetForbiddenAspects() == 45, 'child owns inherited aspects plus SetToDefaults')
            assert(child:GetInheritableForbiddenAspects(path.Hierarchy) == 44)
            assert(child:GetInheritableForbiddenAspects(path.Layout) == 8)
        end
        local grandchild = frame:CreateTexture()
        assert(grandchild:GetForbiddenAspects() == 45, 'hierarchy inheritance remains transitive')
        local slider = CreateFrame('Slider', nil, parent)
        local editBox = CreateFrame('EditBox', nil, parent)
        assert(slider.Low:GetForbiddenAspects() == 45 and slider.ThumbTexture:GetForbiddenAspects() == 45)
        assert(editBox.Text:GetForbiddenAspects() == 45, 'native default regions inherit before lookup')
        parent:AddForbiddenAspects(e.RemoveSecretAspects)
        assert(bit.band(parent:GetInheritableForbiddenAspects(path.Layout), e.RemoveSecretAspects) == 0)
        assert(bit.band(frame:GetForbiddenAspects(), e.RemoveSecretAspects) == 0, 'non-inheritable additions stay local')

        local foreign = CreateFrame('Frame')
        foreign:AddForbiddenAspects(e.UntrustedLayoutScriptExecution)
        local plain = CreateFrame('Frame')
        local ok, err = pcall(plain.SetPoint, plain, 'TOPLEFT', foreign, 'TOPLEFT')
        assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
        ok, err = pcall(plain.SetParent, plain, foreign)
        assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
        assert(plain:GetNumPoints() == 0 and plain:GetParent() ~= foreign)
    "#).expect("creation inherits, later foreign relationships remain denied");
}

#[test]
fn xml_aspects_use_active_enum_values_and_merge_inherited_state() {
    let env = WowLuaEnv::new().unwrap();
    let result = load_xml_fixture(
        &env,
        r#"<Ui>
      <Frame name="AllAspectTemplate" virtual="true"><ForbiddenAspects>
        <ForbiddenAspect aspect="SetToDefaults"/><ForbiddenAspect aspect="ScriptBindings"/>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/><ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
        <ForbiddenAspect aspect="EventRegistrations"/><ForbiddenAspect aspect="AlwaysPropagateInput"/>
        <ForbiddenAspect aspect="ScriptedInput"/><ForbiddenAspect aspect="QueryFocus"/>
        <ForbiddenAspect aspect="ChangeAnimationTarget"/><ForbiddenAspect aspect="RemoveSecretAspects"/>
        <ForbiddenAspect aspect="ChangeParent"/>
      </ForbiddenAspects></Frame>
      <Frame name="AspectParent"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/>
        <ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
        <ForbiddenAspect aspect="AlwaysPropagateInput"/>
      </ForbiddenAspects><Frames><Frame parentKey="Child"><ForbiddenAspects>
        <ForbiddenAspect aspect="ScriptedInput"/>
      </ForbiddenAspects></Frame></Frames></Frame>
      <Frame name="HierarchyOnly"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedScriptExecution"/>
      </ForbiddenAspects></Frame>
      <Frame name="HierarchyAndLayout"><ForbiddenAspects>
        <ForbiddenAspect aspect="UntrustedLayoutScriptExecution"/>
      </ForbiddenAspects></Frame>
      <Frame name="AllLiteral" inherits="AllAspectTemplate"/>
    </Ui>"#,
    );
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    env.exec(r#"
        local path = Enum.ScriptObjectPropagationPath
        assert(AllLiteral:GetForbiddenAspects() == 2047, 'literal XML resolves all eleven active enum names')
        local runtime = CreateFrame('Frame', nil, UIParent, 'AllAspectTemplate')
        assert(runtime:GetForbiddenAspects() == 2047, 'runtime templates apply the same aspects')
        assert(AspectParent.Child:GetForbiddenAspects() == 109, 'own XML aspects retain inherited 44 and implied bit 1')
        assert(HierarchyOnly:GetInheritableForbiddenAspects(path.Hierarchy) == 4)
        assert(HierarchyOnly:GetInheritableForbiddenAspects(path.Layout) == 0)
        assert(HierarchyAndLayout:GetInheritableForbiddenAspects(path.Hierarchy) == 8)
        assert(HierarchyAndLayout:GetInheritableForbiddenAspects(path.Layout) == 8)
    "#).expect("XML and runtime template mapping match native enum values");
}

#[test]
fn xml_unknown_forbidden_aspect_reports_a_load_error() {
    let env = WowLuaEnv::new().unwrap();
    let result = load_xml_fixture(
        &env,
        r#"<Ui><Frame name="InvalidAspect"><ForbiddenAspects>
        <ForbiddenAspect aspect="NotARealForbiddenAspect"/>
    </ForbiddenAspects></Frame></Ui>"#,
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("NotARealForbiddenAspect")),
        "unknown aspects must not silently become zero: {:?}",
        result.warnings
    );
}

#[test]
fn aura_button_icon_and_overlay_border_follow_real_initializer_order() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
            &["Blizzard_AuraContainer"],
            &[],
            |env, _| {
                env.exec(r#"
                    local button = CreateFrame('AuraButton', nil, UIParent, 'CustomAuraButtonTemplate')
                    local icon = button:CreateTexture(nil, 'BACKGROUND')
                    icon:SetAllPoints(button)
                    button:SetIcon(icon)
                    local cooldown = CreateFrame('Cooldown', nil, button, 'CooldownFrameTemplate')
                    cooldown:SetAllPoints(icon)
                    button:SetDurationCooldown(cooldown)
                    local overlay = CreateFrame('Frame', nil, button)
                    overlay:SetAllPoints(button)
                    local count = overlay:CreateFontString(nil, 'OVERLAY', 'NumberFontNormalSmall')
                    button:SetApplicationCount(count)
                    local border = overlay:CreateTexture(nil, 'OVERLAY', nil, 6)
                    border:ClearAllPoints()
                    border:SetPoint('TOPLEFT', icon, 'TOPLEFT', -1, 1)
                    border:SetPoint('BOTTOMRIGHT', icon, 'BOTTOMRIGHT', 1, -1)
                    assert(border:GetNumPoints() == 2)
                    local _, target = border:GetPoint(1)
                    assert(target == icon)
                    local e = Enum.ForbiddenAspect
                    assert(bit.band(button:GetForbiddenAspects(), e.ChangeParent) ~= 0, 'intrinsic XML aspect is applied')
                    assert(bit.band(border:GetForbiddenAspects(), e.UntrustedLayoutScriptExecution) ~= 0)
                    local foreign = CreateFrame('Frame')
                    local ok, err = pcall(foreign.SetPoint, foreign, 'CENTER', icon, 'CENTER')
                    assert(not ok and string.find(err, 'Cannot implicitly gain forbidden aspects', 1, true))
                "#).expect("BetterBlizzFrames icon/cooldown/overlay/border initialization keeps valid ownership");
            },
        );
    });
}
