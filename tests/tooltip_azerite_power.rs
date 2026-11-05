//! WM94 §5: the original `SetAzeritePower` accessor selects the power record's
//! spell and returns the common spell composition for that spell.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    #[test]
    fn tooltip_azerite_power_original_accessor_selects_the_power_spell() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            local link = '|cffa335ee|Hitem:168332::::::::80:70:::::::::|h[Titanic 385]|h|r'
            for _, case in ipairs({
                {1, 55564, 'Increase Versatility 15'},
                {7, 41695, 'Agility 20'},
            }) do
                local power_id, spell_id, name = case[1], case[2], case[3]
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetAzeritePower(168332, 300, power_id, link)
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetAzeritePower', 'azerite-power accessor missing')
                assert(data and data.type == Enum.TooltipDataType.Spell and data.id == spell_id,
                    'power record must select its spell identity')
                assert(#data.lines > 0, 'power spell must compose the common spell lines')
                assert(data.lines[1].leftText == name, 'power spell name')
                assert(GameTooltip:GetLeftLine(1):GetText() == name)
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_azerite_power_unknown_power_is_a_miss() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(
            r#"
            assert(select('#', C_TooltipInfo.GetAzeritePower(168332, 300, 999999)) == 0,
                'unknown power must take the missing route')
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetAzeritePower(168332, 300, 999999)
            assert(GameTooltip:GetPrimaryTooltipData() == nil, 'unknown power must not show a tooltip')
            assert(select('#', C_TooltipInfo.GetAzeritePower(168332, 300, -1)) == 0,
                'negative power must not resolve')
            "#,
        )
        .unwrap();
    }
}
