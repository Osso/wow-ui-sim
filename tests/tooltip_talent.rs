use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn c_tooltip_info_talent_reuses_spell_tooltips() {
    let env = env();
    // R81 Q3: preserve description index 3 with explicit alternate helper text.
    env.state()
        .borrow_mut()
        .spell_tooltip_inputs
        .entry(467911)
        .or_default()
        .alternate_cast_text = Some("host alternate".into());
    let (talent_name, talent_desc, spell_name, spell_desc): (String, String, String, String) = env
        .eval(
            r#"
            local talentTip = C_TooltipInfo.GetTalent(122583)
            local spellTip = C_TooltipInfo.GetSpellByID(467911)
            return talentTip.lines[1].leftText,
                   talentTip.lines[3].leftText,
                   spellTip.lines[1].leftText,
                   spellTip.lines[3].leftText
            "#,
        )
        .unwrap();

    assert_eq!(talent_name, spell_name);
    assert_eq!(talent_desc, spell_desc);
}

#[test]
fn game_tooltip_set_talent_populates_spell_lines() {
    let env = env();
    env.exec("GameTooltip:SetTalent(122583)").unwrap();

    let expected_name: String = env
        .eval(
            r#"
            local spellTip = C_TooltipInfo.GetSpellByID(467911)
            return spellTip.lines[1].leftText
            "#,
        )
        .unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let tooltip = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist");
    assert_eq!(tooltip.lines[0].left_text, expected_name);
}
