#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_custom_word_wrap_min_width_expands_wrapped_only_tooltip_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "WrapMinWidthOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetCustomWordWrapMinWidth(350)
        GameTooltip:AddLine("This is a wrapped-only tooltip body that should honor the custom minimum width instead of collapsing down to padding width.", 1, 1, 1, true)
        GameTooltip:Show()
    "#,
    )
    .unwrap();

    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    if matches!(
        wow_ui_sim::client_profile::ACTIVE,
        wow_ui_sim::client_profile::ClientProfile::Retail
            | wow_ui_sim::client_profile::ClientProfile::Ptr
    ) {
        // WM63 L23:16-22: the custom width caps the candidate; fitting may shrink it.
        let content_width = frame.width - 20.48;
        let line_id = state.tooltips[&gt_id].left_line_ids[0];
        let line_width = state.widgets.get(line_id).unwrap().width;
        assert!(content_width > 0.0 && content_width <= 350.0);
        assert!((line_width - content_width).abs() < 0.01);
    } else {
        assert!(
            frame.width >= 374.0,
            "SetCustomWordWrapMinWidth(350) should expand wrapped tooltip width, got {}",
            frame.width
        );
    }
}

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}
