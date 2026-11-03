#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_set_shrink_to_fit_wrapped_false_keeps_wrapped_line_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ShrinkWrappedOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:SetShrinkToFitWrapped(false)
        GameTooltip:AddLine("This is a wrapped-only tooltip body that should keep its natural width when shrink-to-fit-wrapped is disabled.", 1, 1, 1, true)
        GameTooltip:Show()
    "#,
    )
    .unwrap();

    prepare_tooltip_layout(&env);

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let frame = state.widgets.get(gt_id).unwrap();

    assert!(
        frame.width > 200.0,
        "SetShrinkToFitWrapped(false) should let wrapped lines contribute to tooltip width, got {}",
        frame.width
    );
}

fn prepare_tooltip_layout(env: &WowLuaEnv) {
    use wow_ui_sim::render::font::WowFontSystem;

    let mut font_sys = WowFontSystem::new();
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::prepare_tooltip_layout(&mut state, &mut font_sys);
}
