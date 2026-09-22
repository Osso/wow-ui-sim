use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn template_existence_tracks_loaded_virtual_xml_and_frame_creation() {
    wow_ui_sim::xml::clear_templates();
    let env = WowLuaEnv::new().unwrap();
    let addon = tempfile::tempdir().unwrap();
    std::fs::write(
        addon.path().join("TemplateExistence.toc"),
        "## Title: TemplateExistence\nTemplates.xml\n",
    )
    .unwrap();
    std::fs::write(
        addon.path().join("Templates.xml"),
        r#"<Ui>
        <Frame name="RegistryPredicateVirtual" virtual="true">
            <Size x="41" y="23"/>
        </Frame>
        <Frame name="RegistryPredicateDerived" inherits="RegistryPredicateVirtual" virtual="true"/>
        <Frame name="RegistryPredicateOrdinary" parent="UIParent"/>
    </Ui>"#,
    )
    .unwrap();
    env.exec(
        r#"
        assert(not DoesTemplateExist('RegistryPredicateVirtual'))
        assert(not DoesTemplateExist('RegistryPredicateOrdinary'))
    "#,
    )
    .unwrap();
    load_addon(
        &env.loader_env(),
        &addon.path().join("TemplateExistence.toc"),
    )
    .unwrap();
    env.exec(r#"
        assert(DoesTemplateExist('RegistryPredicateVirtual'))
        assert(DoesTemplateExist('registrypredicatevirtual'))
        assert(DoesTemplateExist('RegistryPredicateDerived'))
        assert(C_XMLUtil.GetTemplateInfo('RegistryPredicateVirtual') ~= nil)
        assert(RegistryPredicateOrdinary ~= nil)
        assert(not DoesTemplateExist('RegistryPredicateOrdinary'))
        local frame = CreateFrame('Frame', 'RegistryPredicateInstance', UIParent, 'RegistryPredicateDerived')
        assert(frame:GetWidth() == 41 and frame:GetHeight() == 23)
        assert(DoesTemplateExist('RegistryPredicateVirtual'))
        assert(not DoesTemplateExist('RegistryPredicateInstance'))
        assert(not DoesTemplateExist('RegistryPredicateVirtual,RegistryPredicateDerived'))
    "#).unwrap();
}

#[test]
fn template_existence_rejects_non_strings_and_reports_missing_names() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(not DoesTemplateExist(''))
        assert(not DoesTemplateExist('RegistryPredicateNeverLoaded'))
        assert(not DoesTemplateExist('UIParent'))
        assert(not pcall(DoesTemplateExist))
        assert(not pcall(DoesTemplateExist, nil))
        for _, value in ipairs({false, 123, {}, function() end}) do
            assert(not pcall(DoesTemplateExist, value))
        end
        local frame = CreateFrame('Frame', 'RegistryPredicateLuaFrame', UIParent)
        assert(frame == RegistryPredicateLuaFrame)
        assert(not DoesTemplateExist('RegistryPredicateLuaFrame'))
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn template_existence_sees_loaded_blizzard_virtual_templates() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_closure(
        &["Blizzard_AuraContainer"],
        &[],
        |env, _| {
            env.exec(r#"
                assert(DoesTemplateExist('CustomAuraContainerTemplate'))
                assert(C_XMLUtil.GetTemplateInfo('CustomAuraContainerTemplate') ~= nil)
                local frame = CreateFrame('AuraContainer', 'RegistryPredicateNativeInstance', UIParent, 'CustomAuraContainerTemplate')
                assert(frame:GetName() == 'RegistryPredicateNativeInstance')
                assert(DoesTemplateExist('CustomAuraContainerTemplate'))
                assert(not DoesTemplateExist('RegistryPredicateNativeInstance'))
            "#).unwrap();
        },
    );
}
