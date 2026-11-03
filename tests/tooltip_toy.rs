use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn c_tooltip_info_toy_by_item_id_uses_seeded_toy_state() {
    let env = env();
    let (tooltip_type, name): (i32, String) = env
        .eval(
            r#"
            local toyTip = C_TooltipInfo.GetToyByItemID(166779)
            return toyTip.type, toyTip.lines[1].leftText
            "#,
        )
        .unwrap();

    assert_eq!(tooltip_type, 0);
    assert_eq!(name, "Transmorpher Beacon");
}

#[test]
fn game_tooltip_set_toy_by_item_id_populates_item_lines() {
    let env = env();
    env.exec("GameTooltip:SetToyByItemID(166779)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let tooltip = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist");
    assert_eq!(tooltip.lines[0].left_text, "Transmorpher Beacon");
}
