#![cfg(feature = "gui")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::font::WowFontSystem;

#[test]
fn test_tooltip_width_with_colors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local owner = CreateFrame("Frame", "ColorTestOwner", UIParent)
        GameTooltip:SetOwner(owner, "ANCHOR_NONE")
        GameTooltip:AddLine("|cffff0000|cff00ff00|cff0000ffStrength|r|r|r")
        GameTooltip:Show()
        "#,
    )
    .unwrap();

    let mut font_system = WowFontSystem::new();
    let width_with_markup = measure_tooltip_width(&env, &mut font_system);
    env.exec(
        r#"
        GameTooltip:ClearLines()
        GameTooltip:AddLine("Strength")
        GameTooltip:Show()
        "#,
    )
    .unwrap();
    let width_without_markup = measure_tooltip_width(&env, &mut font_system);

    assert!(width_without_markup > 30.0, "visible text must contribute width");
    assert!(
        (width_with_markup - width_without_markup).abs() < 1.0,
        "nested color markup must not change tooltip width: {width_with_markup} vs {width_without_markup}"
    );
}

fn measure_tooltip_width(env: &WowLuaEnv, font_system: &mut WowFontSystem) -> f32 {
    let mut state = env.state().borrow_mut();
    wow_ui_sim::iced_app::tooltip::update_tooltip_sizes(&mut state, font_system);
    let id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    state.widgets.get(id).unwrap().width
}
