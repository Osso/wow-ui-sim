#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_allow_show_with_no_lines_keeps_zero_line_tooltip_renderable() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ZeroLineTooltipOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetAllowShowWithNoLines(true)
        GameTooltip:ClearLines()
        GameTooltip:Show()
    "#,
    )
    .unwrap();

    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let tooltip_data = wow_ui_sim::iced_app::tooltip::collect_tooltip_data(&state);
    let render_data = tooltip_data.get(&gt_id);

    assert!(
        render_data.is_some(),
        "SetAllowShowWithNoLines(true) should keep a zero-line tooltip renderable"
    );

    let frame = state.widgets.get(gt_id).unwrap();
    assert!(
        frame.width < 200.0 && frame.height < 100.0,
        "Zero-line tooltip should be remeasured instead of keeping the default size, got {}x{}",
        frame.width,
        frame.height
    );
}

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}
