//! Migrated helpers exercised through unmodified SharedXML and native frames/loaders.
#![cfg(feature = "retail-12-1-0")]
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_shared_xml() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui, &["Blizzard_SharedXML"], &[],
    ).0
}

#[test]
fn patch_12_1_0_migrated_mouse_helper_queries_real_frame_with_offsets() {
    let env = load_shared_xml();
    env.exec(r#"
        assert(MouseIsOver == nil)
        AuditMouse = CreateFrame('Frame', nil, UIParent)
        AuditMouse:SetSize(40, 30)
        AuditMouse:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 100, -100)
        AuditMouse:EnableMouse(true)
    "#).unwrap();
    for (position, expected) in [((120.0, 115.0), true), ((98.0, 115.0), false)] {
        env.state().borrow_mut().set_mouse_position(Some(position));
        assert_eq!(env.eval::<bool>("return InputUtil.IsMouseOver(AuditMouse)").unwrap(), expected);
    }
    env.state().borrow_mut().set_mouse_position(Some((98.0, 115.0)));
    env.exec(r#"
        assert(select('#', InputUtil.IsMouseOver(AuditMouse, 3, 0, 0, 0)) == 1)
        assert(InputUtil.IsMouseOver(AuditMouse, 3, 0, 0, 0))
        assert(not InputUtil.IsMouseOver(AuditMouse, 0, 3, 0, 0))
        AuditMouse:Hide()
        assert(not InputUtil.IsMouseOver(AuditMouse, 3, 0, 0, 0))
    "#).unwrap();
}

#[test]
fn patch_12_1_0_migrated_loader_handles_success_missing_and_disabled_addons() {
    let env = load_shared_xml();
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("AuditLazy");
    std::fs::create_dir(&dir).unwrap();
    std::fs::write(dir.join("AuditLazy.toc"), "## LoadOnDemand: 1\nBody.lua\n").unwrap();
    std::fs::write(dir.join("Body.lua"), "AuditLazyCalls = (AuditLazyCalls or 0) + 1").unwrap();
    env.state().borrow_mut().addon_base_paths.insert(0, root.path().to_path_buf());
    env.exec(r#"
        assert(UIParentLoadAddOn == nil, 'old loader publication must be absent')
        assert(select('#', LoadAddOnWithErrorHandling('AuditLazy')) == 1, 'loader returns one result')
        assert(AuditLazyCalls == 1, 'loader must execute real addon body')
        assert(LoadAddOnWithErrorHandling('AuditLazy') and AuditLazyCalls == 1, 'repeated load must be inert')
        BasicMessageDialog:Hide()
        assert(not LoadAddOnWithErrorHandling('AuditMissing'))
        assert(BasicMessageDialog:IsShown())
        assert(BasicMessageDialog.Text:GetText():find('AuditMissing', 1, true))
        BasicMessageDialog:Hide()
        assert(not LoadAddOnWithErrorHandling('AuditMissing'))
        assert(not BasicMessageDialog:IsShown(), 'same failure reports once')
    "#).unwrap();
    let disabled = root.path().join("AuditDisabled");
    std::fs::create_dir(&disabled).unwrap();
    std::fs::write(disabled.join("AuditDisabled.toc"), "## LoadOnDemand: 1\nBody.lua\n").unwrap();
    std::fs::write(disabled.join("Body.lua"), "AuditDisabledRan = true").unwrap();
    env.exec(r#"
        A_Admin.RegisterTestAddon('AuditDisabled')
        C_AddOns.DisableAddOn('AuditDisabled')
        assert(C_AddOns.GetAddOnEnableState('AuditDisabled') == 0)
        assert(not LoadAddOnWithErrorHandling('AuditDisabled'), 'disabled load must reject')
        assert(AuditDisabledRan == nil)
        assert(BasicMessageDialog:IsShown())
        assert(BasicMessageDialog.Text:GetText():find('AuditDisabled', 1, true))
    "#).unwrap();
}
