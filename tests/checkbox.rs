//! Tests for MinimalCheckboxTemplate and button mouse-enable defaults.

mod common;

use common::env_with_shared_xml;
use wow_ui_sim::iced_app::build_quad_batch_for_registry;

// ============================================================================
// MinimalCheckboxTemplate
// ============================================================================

#[test]
fn minimal_checkbox_size() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCheckbox", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCheckbox").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert_eq!(frame.width, 30.0, "MinimalCheckboxTemplate width should be 30");
    assert_eq!(frame.height, 29.0, "MinimalCheckboxTemplate height should be 29");
}

#[test]
fn minimal_checkbox_normal_texture_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbNormal", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbNormal").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert!(
        frame.normal_texture.is_some(),
        "normal_texture should be set from checkbox-minimal atlas"
    );
    let path = frame.normal_texture.as_ref().unwrap();
    assert!(
        path.to_lowercase().contains("minimalcheckbox"),
        "normal_texture path should reference minimalcheckbox: {}",
        path
    );
    assert!(
        frame.normal_tex_coords.is_some(),
        "normal_tex_coords should be set from atlas"
    );
}

#[test]
fn minimal_checkbox_pushed_texture_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbPushed", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbPushed").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert!(
        frame.pushed_texture.is_some(),
        "pushed_texture should be set from checkbox-minimal atlas"
    );
    assert!(
        frame.pushed_tex_coords.is_some(),
        "pushed_tex_coords should be set from atlas"
    );
}

#[test]
fn minimal_checkbox_highlight_texture_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbHighlight", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbHighlight").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert!(
        frame.highlight_texture.is_some(),
        "highlight_texture should be set from checkbox-minimal atlas"
    );
    assert!(
        frame.highlight_tex_coords.is_some(),
        "highlight_tex_coords should be set from atlas"
    );
}

#[test]
fn minimal_checkbox_checked_texture_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbChecked", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbChecked").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert!(
        frame.checked_texture.is_some(),
        "checked_texture should be set from checkmark-minimal atlas"
    );
    let path = frame.checked_texture.as_ref().unwrap();
    assert!(
        path.to_lowercase().contains("minimalcheckbox"),
        "checked_texture path should reference minimalcheckbox: {}",
        path
    );
    assert!(
        frame.checked_tex_coords.is_some(),
        "checked_tex_coords should be set from atlas"
    );
}

#[test]
fn minimal_checkbox_disabled_checked_texture_atlas() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbDisChecked", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbDisChecked").unwrap();
    let frame = state.widgets.get(id).unwrap();

    assert!(
        frame.disabled_checked_texture.is_some(),
        "disabled_checked_texture should be set from checkmark-minimal-disabled atlas"
    );
    let path = frame.disabled_checked_texture.as_ref().unwrap();
    assert!(
        path.to_lowercase().contains("minimalcheckbox"),
        "disabled_checked_texture path should reference minimalcheckbox: {}",
        path
    );
    assert!(
        frame.disabled_checked_tex_coords.is_some(),
        "disabled_checked_tex_coords should be set from atlas"
    );
}

#[test]
fn minimal_checkbox_set_checked_visibility() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbVis", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    // CheckedTexture child should exist and start hidden
    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("TestMinCbVis").unwrap();
        let frame = state.widgets.get(id).unwrap();
        let checked_tex_id = frame.children_keys.get("CheckedTexture");
        assert!(
            checked_tex_id.is_some(),
            "CheckedTexture child should exist after template application"
        );
        let tex = state.widgets.get(*checked_tex_id.unwrap()).unwrap();
        assert!(!tex.visible, "CheckedTexture should start hidden");
    }

    // SetChecked(true) should show it
    env.exec("TestMinCbVis:SetChecked(true)").unwrap();
    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("TestMinCbVis").unwrap();
        let frame = state.widgets.get(id).unwrap();
        let checked_tex_id = *frame.children_keys.get("CheckedTexture").unwrap();
        let tex = state.widgets.get(checked_tex_id).unwrap();
        assert!(tex.visible, "CheckedTexture should be visible after SetChecked(true)");
    }

    // SetChecked(false) should hide it
    env.exec("TestMinCbVis:SetChecked(false)").unwrap();
    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("TestMinCbVis").unwrap();
        let frame = state.widgets.get(id).unwrap();
        let checked_tex_id = *frame.children_keys.get("CheckedTexture").unwrap();
        let tex = state.widgets.get(checked_tex_id).unwrap();
        assert!(!tex.visible, "CheckedTexture should be hidden after SetChecked(false)");
    }
}

#[test]
fn minimal_checkbox_quad_batch() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbQuad", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
        cb:Show()
    "#,
    )
    .unwrap();

    let buckets = {
        let mut state = env.state().borrow_mut();
        let _ = state.get_strata_buckets();
        state.strata_buckets.as_ref().unwrap().clone()
    };
    let state = env.state().borrow();
    let batch = build_quad_batch_for_registry(
        &state.widgets,
        (1024.0, 768.0),
        Some("TestMinCbQuad"),
        None,
        None,
        None,
        None,
        None,
        &buckets,
    );

    // Should have texture requests referencing the checkbox atlas path
    let checkbox_requests: Vec<_> = batch
        .texture_requests
        .iter()
        .filter(|r| r.path.to_lowercase().contains("minimalcheckbox"))
        .collect();

    assert!(
        !checkbox_requests.is_empty(),
        "Quad batch should contain texture requests for minimalcheckbox atlas. Got paths: {:?}",
        batch.texture_requests.iter().map(|r| &r.path).collect::<Vec<_>>()
    );
}

#[test]
fn minimal_checkbox_click_toggles_checked() {
    let env = env_with_shared_xml();

    // Create checkbox with an OnClick handler that toggles checked state
    // (WoW CheckButtons have no built-in toggle - addons must wire OnClick)
    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbClick", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
        cb:SetScript("OnClick", function(self, button, down)
            self:SetChecked(not self:GetChecked())
        end)
    "#,
    )
    .unwrap();

    // Starts unchecked
    let checked: bool = env.eval("return TestMinCbClick:GetChecked()").unwrap();
    assert!(!checked, "Should start unchecked");

    // Click() fires OnClick -> toggles to checked
    env.exec("TestMinCbClick:Click()").unwrap();
    let checked: bool = env.eval("return TestMinCbClick:GetChecked()").unwrap();
    assert!(checked, "Should be checked after first Click()");

    // CheckedTexture should now be visible
    assert_checked_texture_visible(&env, "TestMinCbClick", true);

    // Click() again -> toggles to unchecked
    env.exec("TestMinCbClick:Click()").unwrap();
    let checked: bool = env.eval("return TestMinCbClick:GetChecked()").unwrap();
    assert!(!checked, "Should be unchecked after second Click()");

    // CheckedTexture should be hidden again
    assert_checked_texture_visible(&env, "TestMinCbClick", false);
}

fn assert_checked_texture_visible(
    env: &wow_ui_sim::lua_api::WowLuaEnv,
    name: &str,
    expected: bool,
) {
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name(name).unwrap();
    let frame = state.widgets.get(id).unwrap();
    let tex_id = *frame.children_keys.get("CheckedTexture").unwrap();
    let tex = state.widgets.get(tex_id).unwrap();
    assert_eq!(
        tex.visible, expected,
        "CheckedTexture should be {} for {}",
        if expected { "visible" } else { "hidden" },
        name,
    );
}

#[test]
fn minimal_checkbox_mouse_enabled_by_default() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local cb = CreateFrame("CheckButton", "TestMinCbMouse", UIParent, "MinimalCheckboxTemplate")
        cb:SetPoint("CENTER")
    "#,
    )
    .unwrap();

    // CheckButton should have mouse enabled by default (like WoW)
    let mouse_enabled: bool = env
        .eval("return TestMinCbMouse:IsMouseEnabled()")
        .unwrap();
    assert!(
        mouse_enabled,
        "CheckButton should have mouse enabled by default"
    );

    // Also verify via Rust state
    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestMinCbMouse").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(
        frame.mouse_enabled,
        "CheckButton frame.mouse_enabled should be true"
    );
}

#[test]
fn button_mouse_enabled_by_default() {
    let env = env_with_shared_xml();

    env.exec(
        r#"
        local btn = CreateFrame("Button", "TestBtnMouse", UIParent)
        btn:SetPoint("CENTER")
        btn:SetSize(100, 30)
    "#,
    )
    .unwrap();

    let mouse_enabled: bool = env.eval("return TestBtnMouse:IsMouseEnabled()").unwrap();
    assert!(
        mouse_enabled,
        "Button should have mouse enabled by default"
    );
}
