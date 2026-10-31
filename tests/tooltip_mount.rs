use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn c_tooltip_info_mount_by_spell_id_uses_seeded_mount_state() {
    let env = env();
    let (tooltip_type, name, spell_id): (i32, String, i32) = env
        .eval(
            r#"
            local mountTip = C_TooltipInfo.GetMountBySpellID(458)
            return mountTip.type, mountTip.lines[1].leftText, mountTip.id
            "#,
        )
        .unwrap();

    assert_eq!(tooltip_type, 1);
    assert_eq!(name, "Brown Horse");
    assert_eq!(spell_id, 458);
}

#[test]
fn game_tooltip_set_mount_by_spell_id_populates_mount_lines() {
    let env = env();
    env.exec("GameTooltip:SetMountBySpellID(72286)").unwrap();

    let state = env.state().borrow();
    let gt_id = state.widgets.get_id_by_name("GameTooltip").unwrap();
    let tooltip = state
        .tooltips
        .get(&gt_id)
        .expect("tooltip data should exist");
    assert_eq!(tooltip.lines[0].left_text, "Invincible");
}
