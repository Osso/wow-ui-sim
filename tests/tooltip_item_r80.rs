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

#[test]
fn item_r80_original_consumers_use_final_host_amounts_and_metadata() {
    use wow_ui_sim::c_api::item_tooltip::{ItemTooltipInput, ItemTooltipStat};
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    let level = wow_ui_sim::items::get_item(229181).unwrap().item_level;
    for amount in [37, -9] {
        env.state().borrow_mut().item_tooltip_inputs.insert(
            (229181, level),
            ItemTooltipInput {
                stats: vec![ItemTooltipStat {
                    label: "Strength".into(),
                    amount,
                }],
                guid: Some("Item-1-00000080".into()),
                hyperlink: Some("item:229181".into()),
                override_item_level: Some(700),
                is_corrupted_item: Some(true),
                is_azerite_empowered_item: Some(false),
                is_azerite_item: Some(true),
            },
        );
        env.exec(&format!(r#"
            local expected = '{amount:+} Strength'
            local function check(data)
                assert(data.id == 229181)
                assert(data.guid == 'Item-1-00000080' and data.hyperlink == 'item:229181')
                assert(data.overrideItemLevel == 700)
                assert(data.isCorruptedItem == true and data.isAzeriteEmpoweredItem == false and data.isAzeriteItem == true)
                local line = data.lines[#data.lines]
                assert(line.leftText == expected, 'host amount, not level/class estimate')
                assert(line.wrapText == nil, 'R80 Q1: unwrapped lines omit wrapText')
                assert(line.rightText == nil and line.rightColor == nil)
                assert(line.guid == nil and line.isCorruptedItem == nil, 'metadata is top-level')
            end
            check(C_TooltipInfo.GetItemByID(229181))
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetItemByID(229181)
            check(GameTooltip:GetPrimaryTooltipData())
            assert(GameTooltip:GetLeftLine(GameTooltip:NumLines()):GetText() == expected)
            GameTooltip:SetHyperlink('item:229181')
            check(GameTooltip:GetPrimaryTooltipData())
        "#)).unwrap();
    }
    env.state().borrow_mut().item_tooltip_inputs.clear();
    env.exec(
        r#"
        local data = C_TooltipInfo.GetItemByID(229181)
        assert(data.guid == nil and data.isCorruptedItem == nil and data.overrideItemLevel == nil)
        for _,line in ipairs(data.lines) do assert(line.leftText ~= '-9 Strength') end
    "#,
    )
    .unwrap();
}
