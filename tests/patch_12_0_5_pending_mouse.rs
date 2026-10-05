//! Bounded parent-frame contract through unchanged cached RestrictedFrames.lua.
#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn load_restricted_mouse_fixture() -> WowLuaEnv {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let (env, _) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui,
        &["Blizzard_RestrictedAddOnEnvironment"],
        &[],
    );
    env.exec(
        r#"
        parent = CreateFrame('Frame', 'PendingMouseParent', UIParent, 'SecureHandlerBaseTemplate')
        parent:SetSize(100, 80)
        parent:SetPoint('BOTTOMLEFT', UIParent, 'BOTTOMLEFT', 100, 100)
        child = CreateFrame('Frame', 'PendingMouseChild', parent, 'SecureHandlerBaseTemplate')
        child:SetSize(20, 20)
        child:SetPoint('BOTTOMLEFT', parent, 'BOTTOMLEFT', 120, 0)
        plain = CreateFrame('Frame', 'PendingMousePlainChild', parent)
        plain:SetSize(20, 20)
        plain:SetPoint('BOTTOMLEFT', parent, 'BOTTOMLEFT', 160, 0)
        mouseHandle = GetFrameHandle(parent, true)
        assert(mouseHandle and parent:IsProtected() and child:IsProtected())
        assert(not plain:IsProtected())
        "#,
    )
    .expect("load real frame-handle implementation and anchored protected frames");
    env
}

fn move_cursor_to_bottom_origin(env: &WowLuaEnv, x: f32, y: f32) {
    let state = env.state();
    let mut state = state.borrow_mut();
    let renderer_y = state.screen_height - y;
    state.set_mouse_position(Some((x, renderer_y)));
}

#[test]
fn parent_under_mouse_uses_rectangle_not_hover_focus() {
    let env = load_restricted_mouse_fixture();
    move_cursor_to_bottom_origin(&env, 150.0, 140.0);
    env.exec(
        r#"
        assert(GetMouseFocus() == nil, 'no hovered-frame shortcut')
        assert(mouseHandle:IsUnderMouse(false), 'parent rectangle contains cursor')
        assert(mouseHandle:IsUnderMouse(true), 'recursive parent check also succeeds')
        "#,
    )
    .unwrap();
    move_cursor_to_bottom_origin(&env, 90.0, 140.0);
    env.exec("assert(not mouseHandle:IsUnderMouse(false))")
        .unwrap();
    move_cursor_to_bottom_origin(&env, 100.0, 100.0);
    env.exec("assert(mouseHandle:IsUnderMouse(false), 'inclusive bottom-left edge')")
        .unwrap();
}

#[test]
fn recursive_parent_under_mouse_checks_visible_protected_child() {
    let env = load_restricted_mouse_fixture();
    move_cursor_to_bottom_origin(&env, 230.0, 110.0);
    env.exec(
        r#"
        assert(not mouseHandle:IsUnderMouse(false), 'child lies outside parent rectangle')
        assert(mouseHandle:IsUnderMouse(true), 'protected visible child counts recursively')
        child:Hide()
        assert(not mouseHandle:IsUnderMouse(true), 'hidden child must not count')
        child:Show()
        assert(mouseHandle:IsUnderMouse(true), 'show restores recursive hit')
        "#,
    )
    .unwrap();
    move_cursor_to_bottom_origin(&env, 270.0, 110.0);
    env.exec("assert(not mouseHandle:IsUnderMouse(true), 'unprotected child excluded')")
        .unwrap();
}

#[test]
fn scaled_parent_under_mouse_normalizes_cursor_by_effective_scale() {
    let env = load_restricted_mouse_fixture();
    env.exec("parent:SetScale(2)").unwrap();
    move_cursor_to_bottom_origin(&env, 300.0, 280.0);
    env.exec(
        r#"
        assert(parent:GetEffectiveScale() == 2)
        assert(mouseHandle:IsUnderMouse(false), 'physical cursor normalized into scaled rectangle')
        "#,
    )
    .unwrap();
    move_cursor_to_bottom_origin(&env, 410.0, 280.0);
    env.exec("assert(not mouseHandle:IsUnderMouse(false), 'outside scaled parent')")
        .unwrap();
}
