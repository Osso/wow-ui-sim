//! Next smallest remaining O subset: original panel stack and menu owners.
//! Bootstrap absence is retirement RED; loaded-owner behaviors are preservation.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]
use crate::common::blizzard_addon_harness::{
    load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn original_panel_and_game() -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    wow_ui_sim::xml::register_intrinsic_templates();
    crate::common::panel_fixtures::load_panel_addons(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_Game"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_Game"));
    env.apply_post_load_workarounds();
    for error in &env.state().borrow().lua_errors {
        eprintln!("O panel/menu dependency error: {error:?}");
    }
    env.exec(
        r#"
        local function original(fn, path)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(path, 1, true), source)
        end
        for _, fn in ipairs({RegisterUIPanel, ShowUIPanel, HideUIPanel, CloseAllWindows}) do
            original(fn, 'Blizzard_UIParentPanelManager/Shared/UIParentPanelManager.lua')
        end
        original(CloseMenus, 'Blizzard_Game/Shared/Game.lua')
    "#,
    )
    .expect("selected original panel and Game owners");
    env
}

#[test]
fn panel_and_menu_helpers_are_not_preowner_bootstrap_fallbacks() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'ShowUIPanel', 'HideUIPanel', 'CloseAllWindows', 'CloseMenus'}) do
            assert(rawget(_G, name) == nil, 'pre-owner fallback remains: ' .. name)
        end
    "#,
    )
    .unwrap();
}

#[test]
fn original_registered_panel_stack_displaces_and_closes_preservation() {
    let env = original_panel_and_game();
    env.exec(r#"
        -- Same legitimate addon registration pattern as test_showuipanel.rs543-620.
        -- Frames are fixture inputs; no original functions or native data replaced.
        local first = CreateFrame('Frame', 'StubOPanelStackFirst', UIParent)
        local second = CreateFrame('Frame', 'StubOPanelStackSecond', UIParent)
        for _, panel in ipairs({first, second}) do
            panel:SetSize(300, 400)
            panel:Hide()
            RegisterUIPanel(panel, {area = 'center', pushable = 0, whileDead = 1})
        end
        assert(select('#', ShowUIPanel(first)) == 0, 'original ShowUIPanel returns no success boolean')
        assert(first:IsShown() and not second:IsShown())
        assert(GetUIPanel('center') == first, 'real delegate did not select first panel')
        assert(first:GetAttribute('UIPanelLayout-defined'), 'original lazy attributes missing')
        ShowUIPanel(second)
        assert(not first:IsShown() and second:IsShown(), 'original center slot did not displace')
        assert(GetUIPanel('center') == second)
        assert(select('#', HideUIPanel(second)) == 0, 'original HideUIPanel returns no success boolean')
        assert(not second:IsShown())
        ShowUIPanel(second)
        -- A valid addon-owned UISpecialFrames registration gives CloseAllWindows
        -- a definite truthy populated return independently of panel slot policy.
        local special = CreateFrame('Frame', 'StubOSpecialWindow', UIParent)
        table.insert(UISpecialFrames, special:GetName())
        special:Show()
        assert(CloseAllWindows(), 'original closer did not report populated special window')
        assert(not first:IsShown() and not second:IsShown() and not special:IsShown())
        assert(GetUIPanel('center') == nil)
        for i = #UISpecialFrames, 1, -1 do
            if UISpecialFrames[i] == special:GetName() then table.remove(UISpecialFrames, i) end
        end
    "#).expect("original panel delegate lifecycle and special-window consumer");
}

#[test]
fn original_close_menus_hides_xml_menus_and_returns_no_values_preservation() {
    let env = original_panel_and_game();
    env.exec(
        r#"
        -- Game owns UIMenus={'DropDownList1','DropDownList2'}; selected SharedXML
        -- Mainline/UIDropDownMenu.xml owns these real template-backed frames.
        assert(DropDownList1 and DropDownList2, 'selected dropdown XML owners missing')
        assert(DropDownList1:GetScript('OnShow'), 'actual dropdown script binding missing')
        DropDownList1:Show()
        DropDownList2:Show()
        assert(DropDownList1:IsShown() and DropDownList2:IsShown())
        assert(select('#', CloseMenus()) == 0, 'original secure wrapper has zero returns')
        assert(not DropDownList1:IsShown() and not DropDownList2:IsShown())
        assert(select('#', CloseMenus()) == 0, 'empty menu close still has zero returns')
    "#,
    )
    .expect("original secure menu closer over selected XML frames");
}
