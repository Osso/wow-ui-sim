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
fn toy_catalog_identity_is_consistent_across_toy_box_and_tooltip() {
    let env = env();
    env.exec(
        r#"
        local itemID, name, icon = C_ToyBox.GetToyInfo(86575)
        assert(itemID == 86575 and name == 'Chalice of Secrets' and icon == 305294)
        GameTooltip:SetToyByItemID(86575)
        assert(GameTooltipTextLeft1:GetText() == name)
        local _, piccolo, piccoloIcon = C_ToyBox.GetToyInfo(13379)
        assert(piccolo == 'Piccolo of the Flaming Fire' and piccoloIcon == 133942)
        "#,
    )
    .unwrap();
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
