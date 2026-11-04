//! Original GroupLootFrame icon hover over existing roll-ID keyed item links.
#![cfg(any(feature = "profile-retail", feature = "client-ptr", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_loot_roll_original_hover_selects_roll_link() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    for (roll, item_id) in [(71, 1728), (92, 127830)] {
        let item = wow_ui_sim::items::get_item(item_id).expect("generated catalog item");
        env.exec(&format!(
            r#"
            A_Admin.StartLootRoll({roll}, 60000, {name:?}, '', 4, 0, 'item:{item_id}')
            GroupLootFrame1.rollID = {roll}
            local icon = GroupLootFrame1.IconFrame
            assert(icon and icon:GetScript('OnEnter'), 'original loot icon hover must load')
            icon:GetScript('OnEnter')(icon)
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetLootRollItem', 'loot-roll accessor missing')
            assert(data and data.id == {item_id}, 'roll must select its own item link')
            assert(data.type == Enum.TooltipDataType.Item and #data.lines > 0)
            assert(data.lines[1].leftText == {name:?}, 'title must come from generated catalog')
            assert(GameTooltip:GetLeftLine(1):GetText() == {name:?})
            "#,
            name = item.name,
        ))
        .unwrap();
    }
    env.exec(
        r#"
        A_Admin.EndLootRoll(71)
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetLootRollItem(71)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'removed roll must not reuse a prior item')
        assert(select('#', C_TooltipInfo.GetLootRollItem(71)) == 0, 'EMULATED shared miss arity')
        GameTooltip:SetLootRollItem(9999)
        assert(GameTooltip:GetPrimaryTooltipData() == nil, 'unknown roll must not fabricate data')
        "#,
    )
    .unwrap();
}
