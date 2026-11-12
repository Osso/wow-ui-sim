//! R80 Q2: ordinary item-mod amounts remain UNKNOWN; do not emit estimated stats.
#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn item_r80_original_consumers_omit_unsupplied_stat_amounts() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    env.exec(
        r#"
        local function check(data)
            assert(data and data.id == 229181, 'selected item')
            for _, line in ipairs(data.lines) do
                assert(not string.match(line.leftText or '', '^%+%d+ '),
                    'R80 Q2: ordinary item stats have no verified amount producer')
            end
        end
        check(C_TooltipInfo.GetItemByID(229181))
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetItemByID(229181)
        check(GameTooltip:GetPrimaryTooltipData())
        assert(GameTooltip:GetPrimaryTooltipInfo().getterName == 'GetItemByID')
        GameTooltip:SetHyperlink('item:229181')
        check(GameTooltip:GetPrimaryTooltipData())
        assert(GameTooltip:GetPrimaryTooltipInfo().getterName == 'GetHyperlink')
        "#,
    )
    .unwrap();
}
