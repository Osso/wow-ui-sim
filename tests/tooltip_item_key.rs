//! WM94 §2: the original `SetItemKey` accessor applies the displayed-level
//! override and the item-key required-level control over the shared catalog
//! item producer.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    const LINE_TYPE_REQUIREMENT: f64 = 5.0;

    fn requirement_line(env: &wow_ui_sim::lua_api::WowLuaEnv, key: &str) -> (f64, f64) {
        env.eval(&format!(
            r#"
            local data = C_TooltipInfo.GetItemKey({key})
            for _, line in ipairs(data.lines) do
                if line.type == {LINE_TYPE_REQUIREMENT} then
                    return tonumber(line.leftText:match('(%d+)$')), line.level
                end
            end
            return 0, 0
            "#,
        ))
        .unwrap()
    }

    #[test]
    fn tooltip_item_key_applies_displayed_level_override() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetItemKey(1728, 40, 0, 0)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetItemKey', 'item-key accessor missing')
            assert(data and data.type == Enum.TooltipDataType.Item and data.id == 1728)
            assert(data.overrideItemLevel == 40, 'nonzero level must publish the override')
            assert(data.lines[1].leftText == "Teebu's Blazing Longsword")
            assert(data.lines[2].leftText == 'Item Level 40', 'level override must reach the level line')
            assert(GameTooltip:GetLeftLine(2):GetText() == 'Item Level 40')

            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetItemKey(1728, 0, 0, 0)
            local plain = GameTooltip:GetPrimaryTooltipData()
            assert(plain.id == 1728 and plain.overrideItemLevel == nil, 'zero level keeps the normal calculation')
            assert(plain.lines[2].leftText == 'Item Level 29')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_item_key_clamps_the_working_level_but_publishes_the_input() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            local data = C_TooltipInfo.GetItemKey(1728, 5000, 0, 0)
            assert(data.overrideItemLevel == 5000, 'published override keeps the supplied value')
            assert(data.lines[2].leftText == 'Item Level 1300', 'working level is bounded to 1300')
            local low = C_TooltipInfo.GetItemKey(1728, 1, 0, 0)
            assert(low.overrideItemLevel == 1 and low.lines[2].leftText == 'Item Level 1')
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_item_key_required_level_combines_with_the_item_requirement() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        assert_eq!(requirement_line(&env, "1728, 0, 0, 80"), (80.0, 80.0));
        assert_eq!(requirement_line(&env, "1728, 0, 0, 20"), (25.0, 25.0));
        assert_eq!(requirement_line(&env, "1728, 0, 0, 0"), (0.0, 0.0));
        assert_eq!(requirement_line(&env, "221125, 0, 0, 0"), (0.0, 0.0));
        assert_eq!(requirement_line(&env, "221125, 0, 0, 90"), (90.0, 90.0));
    }

    #[test]
    fn tooltip_item_key_suffix_without_a_record_keeps_the_base_name() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            local data = C_TooltipInfo.GetItemKey(1728, 0, 1234, 0)
            assert(data.lines[1].leftText == "Teebu's Blazing Longsword",
                'no suffix record is modelled, so the base name is published')
            assert(data.itemSuffix == nil)
            assert(select('#', C_TooltipInfo.GetItemKey(999999, 0, 0, 0)) == 0,
                'missing catalog item must take the missing route')
            "#,
        )
        .unwrap();
    }
}
