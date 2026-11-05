//! Original GroupLootFrame icon hover over existing roll-ID keyed item links.
#![cfg(any(feature = "profile-retail", feature = "client-ptr", feature = "client-wowforever"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_loot_roll_original_hover_resolves_stored_descriptor() {
    let env = tooltip_full_env_helpers::load_full_env_without_post_load();
    // WM97 DQ7: the ingress decodes the host link once; the getter reads the stored
    // descriptor. Pinned list 591 adds 4 levels to catalog level 29 on the profiles
    // whose record set is modelled (Retail/PTR). Forever publishes the same getters
    // but has no pinned classic record set here, so it resolves the catalog item.
    let level = if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        29
    } else {
        33
    };
    env.exec(&format!(
        r#"
        A_Admin.StartLootRoll(93, 60000, 'Descriptor Sword', '', 4, 0,
            'item:1728:0:0:0:0:0:0:0:0:0:0:0:1:591:0:0:0:0::0')
        GroupLootFrame1.rollID = 93
        local icon = GroupLootFrame1.IconFrame
        assert(icon and icon:GetScript('OnEnter'), 'original loot icon hover must load')
        icon:GetScript('OnEnter')(icon)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and data.id == 1728, 'roll selects its stored descriptor item ID')
        assert(data.lines[2].leftText == 'Item Level {level}', 'stored descriptor must reach the level line')
        assert(GameTooltip:GetLeftLine(2):GetText() == 'Item Level {level}')
        "#,
    ))
    .unwrap();
    // A roll without a descriptor-bearing link stays a miss; no catalog item is invented.
    env.exec(
        r#"
        A_Admin.StartLootRoll(94, 60000, 'No Link', '', 4, 0)
        assert(select('#', C_TooltipInfo.GetLootRollItem(94)) == 0, 'descriptor-free roll is a miss')
        "#,
    )
    .unwrap();
}

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
