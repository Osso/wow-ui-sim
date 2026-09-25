use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::TargetInfo;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn tooltip_spell_identity_survives_missing_local_metadata() {
    let env = env();
    env.state().borrow_mut().action_bars.insert(5, 45524);
    env.exec(
        r#"
        for _, id in ipairs({45524, 19750}) do
            local getters = {
                C_TooltipInfo.GetSpellByID(id),
                C_TooltipInfo.GetSpell(id),
                C_TooltipInfo.GetHyperlink("|Hspell:" .. id .. "|h[Spell]|h"),
                C_TooltipInfo.GetMountBySpellID(id),
            }
            for _, data in ipairs(getters) do
                assert(data.type == Enum.TooltipDataType.Spell, "wrong tooltip type for spell " .. id)
                assert(data.id == id, "spell tooltip lost supplied identity " .. id)
            end
        end
        local action = C_TooltipInfo.GetAction(5)
        assert(action.id == 45524, "action tooltip lost spell identity")
        assert(#action.lines > 0, "action binding line should remain present")
        assert(C_TooltipInfo.GetAction(999) == nil)
        "#,
    )
    .expect("supplied spell identity must not depend on local spell metadata");
}

#[test]
fn hyperlink_unit_guid_uses_modeled_player_and_target_tooltips() {
    let env = env();
    let target_guid = "Creature-0-0-0-0-12345-0000000000";
    {
        let mut sim = env.state().borrow_mut();
        sim.player.name = "Uther".into();
        sim.current_target = Some(TargetInfo {
            unit_id: "target".into(),
            name: "Quilboar".into(),
            guid: target_guid.into(),
            level: 10,
            creature_type: "Humanoid".into(),
            class_index: 1,
            health: 100,
            health_max: 100,
            power: 0,
            power_max: 0,
            power_type: 0,
            power_type_name: "MANA".into(),
            is_player: false,
            is_enemy: true,
            classification: "normal".into(),
            reaction: 2,
            interaction: Default::default(),
        });
    }
    env.exec(
        r#"
        local player = C_TooltipInfo.GetHyperlink("unit:Player-1-00000001")
        assert(player.type == Enum.TooltipDataType.Unit and player.guid == "Player-1-00000001")
        assert(player.lines[1].leftText == "Uther")
        local target = C_TooltipInfo.GetHyperlink("unit:Creature-0-0-0-0-12345-0000000000")
        assert(target.type == Enum.TooltipDataType.Unit)
        assert(target.guid == UnitGUID("target"))
        assert(target.lines[1].leftText == "Quilboar")
        assert(target.lines[2].leftText == "Level 10")
        "#,
    )
    .expect("unit hyperlinks should resolve modeled GUIDs before consumer first-line access");
}

#[test]
fn missing_unit_hyperlink_returns_nil_for_att_shaped_retry() {
    let env = env();
    env.exec(
        r#"
        local guid = "Creature-0-0-0-0-98765-0000000000"
        local function readName()
            local tooltipData = C_TooltipInfo.GetHyperlink("unit:" .. guid)
            if tooltipData and tooltipData.lines and tooltipData.lines[1] then
                return tooltipData.lines[1].leftText
            end
        end
        assert(readName() == nil)
        assert(readName() == nil)
        "#,
    )
    .expect("missing GUID must not fabricate a tooltip on retry");
}

#[test]
fn hyperlink_unsupported_and_malformed_links_return_nil_without_changing_item_spell() {
    let env = env();
    env.exec(
        r#"
        for _, link in ipairs({"unit:", "unit:not-a-guid", "unit:Creature-0-0-0-0-12-0000000000-extra", "quest:42", "|Hfoo:1|h[foo]|h", "|Hitem:bad|h[x]|h"}) do
            assert(C_TooltipInfo.GetHyperlink(link) == nil, link)
        end
        local item = C_TooltipInfo.GetHyperlink("|Hitem:6948|h[Hearthstone]|h")
        assert(item.type == Enum.TooltipDataType.Item and item.lines[1].leftText == "Hearthstone")
        local spell = C_TooltipInfo.GetHyperlink("|Hspell:19750|h[Flash of Light]|h")
        assert(spell.type == Enum.TooltipDataType.Spell and spell.lines[1].leftText == "Flash of Light")
        "#,
    )
    .expect("hyperlink presence and item/spell payloads follow the modeled source");
}

#[test]
fn tooltip_item_and_toy_payloads_retain_modeled_identity() {
    let env = env();
    env.exec(
        r#"
        local item = C_TooltipInfo.GetItemByID(6948)
        assert(item.id == 6948 and item.lines[1].leftText == "Hearthstone")
        local itemByLink = C_TooltipInfo.GetTooltipDataForItem("item:6948")
        assert(itemByLink.id == 6948)
        local toy = C_TooltipInfo.GetToyByItemID(166779)
        assert(toy.id == 166779 and toy.lines[1].leftText == "Hearthstone Game Table")
        local mount = C_TooltipInfo.GetMountBySpellID(23338)
        assert(mount.id == 23338 and mount.lines[1].leftText == "Swift Palomino")
        "#,
    )
    .expect("identified item, toy and mount payloads retain their modeled identity");
}

#[test]
fn c_tooltip_info_item_source_aliases_delegate_to_existing_paths() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local baseline = C_TooltipInfo.GetItemByID(6948)
            local bagItem = C_TooltipInfo.GetBagItem(0, 1)
            local viaBagLocation = C_TooltipInfo.GetItem({ bagID = 0, slotIndex = 1 })
            local viaItemId = C_TooltipInfo.GetItem(6948)
            local viaItemLink = C_TooltipInfo.GetTooltipDataForItem("item:6948")

            local equipped = C_TooltipInfo.GetInventoryItem("player", 1)
            local viaEquipmentLocation = C_TooltipInfo.GetItem({ equipmentSlotIndex = 1 })

            local spellBaseline = C_TooltipInfo.GetSpellByID(19750)
            local spellByLink = C_TooltipInfo.GetSpell(GetSpellLink(19750))

            if bagItem.lines[1].leftText ~= baseline.lines[1].leftText then
                return "bag_item_should_match_item_by_id"
            end
            if viaBagLocation.lines[1].leftText ~= baseline.lines[1].leftText then
                return "item_location_should_match_bag_item"
            end
            if viaItemId.lines[1].leftText ~= baseline.lines[1].leftText then
                return "numeric_item_should_match_item_by_id"
            end
            if viaItemLink.lines[1].leftText ~= baseline.lines[1].leftText then
                return "tooltip_data_for_item_should_match_item_by_id"
            end
            if viaEquipmentLocation.lines[1].leftText ~= equipped.lines[1].leftText then
                return "equipment_location_should_match_inventory_item"
            end
            if spellByLink.lines[1].leftText ~= spellBaseline.lines[1].leftText then
                return "spell_alias_should_match_spell_by_id"
            end

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "Tooltip item/spell source aliases should reuse the existing item and spell tooltip paths"
    );
}
