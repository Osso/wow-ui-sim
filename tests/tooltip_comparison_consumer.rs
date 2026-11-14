//! R88n.6: drive the original Retail comparison manager through real frames.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

use wow_ui_sim::c_api::item_tooltip::{ItemTooltipInput, ItemTooltipStat};
use wow_ui_sim::lua_api::WowLuaEnv;

fn helm_links(env: &WowLuaEnv) -> (String, String) {
    env.eval(
        r#"
        for slot = 1, 19 do A_Admin.UnequipItem(slot) end
        A_Admin.EquipItem(1, 211993)
        local first = GetInventoryItemLink("player", 1)
        A_Admin.EquipItem(1, 221125)
        local second = GetInventoryItemLink("player", 1)
        assert(first and second and first ~= second, "distinct catalog helms")
        return first, second
        "#,
    )
    .expect("equip catalog fixtures in the real inventory")
}

#[test]
fn tooltip_comparison_consumer_selects_equipped_item_and_displays_distinct_deltas() {
    let env = tooltip_full_env_helpers::setup_full_env();
    let (first, second) = helm_links(&env);
    for (id, hyperlink, amount) in [(211993, first, 41), (221125, second, 13)] {
        let item = wow_ui_sim::items::get_item(id).expect("catalog helm");
        env.state().borrow_mut().item_tooltip_inputs.insert(
            (id, item.item_level),
            ItemTooltipInput {
                stats: vec![ItemTooltipStat {
                    label: "Stamina".to_owned(),
                    amount,
                }],
                hyperlink: Some(hyperlink),
                ..Default::default()
            },
        );
    }

    env.exec(
        r#"
        local manager = assert(TooltipComparisonManager, "load pinned SharedXMLGame TOC")
        local function inspect(sourceID, equippedID)
            manager:Clear()
            -- Capture a genuine linked item from the standard inventory fixture,
            -- then equip the other helm in the same slot before comparison.
            A_Admin.EquipItem(1, sourceID)
            local sourceLink = GetInventoryItemLink("player", 1)
            A_Admin.EquipItem(1, equippedID)
            local equippedLink = GetInventoryItemLink("player", 1)
            assert(sourceLink and equippedLink and sourceLink ~= equippedLink)
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:SetHyperlink(sourceLink)
            local source = manager:CreateComparisonItem(GameTooltip:GetPrimaryTooltipData())
            assert(source, "real item tooltip supplies the comparison descriptor")

            -- Invoke the original consumer directly: public FrameScript spelling
            -- is not established by the native callback evidence.
            manager:CompareItem(source, GameTooltip)
            local info = assert(manager.compareInfo, "comparison manager received no record")
            local candidate = assert(info.item, "comparison record must select an item")
            local data = assert(manager:GetComparisonItemData(candidate), "resolve selected item")
            local selectedID = data.id or (data.hyperlink and
                tonumber(data.hyperlink:match("item:(%d+)")))
            assert(selectedID == equippedID, "Info must select the supplied equipped helm")
            local shopping = GameTooltip.shoppingTooltips[1]
            assert(shopping and shopping:IsShown(), "original consumer shows selected tooltip")
            local rendered = {}
            local summary = false
            for i = 1, shopping:NumLines() do
                local row = _G[shopping:GetName() .. "TextLeft" .. i]
                local str = row and row:GetText()
                if str == ITEM_DELTA_DESCRIPTION then
                    summary = true
                elseif summary and str and str:find("%S") then
                    rendered[#rendered + 1] = str
                end
            end
            assert(summary and #rendered > 0, "original consumer emitted no delta rows")
            return table.concat(rendered, "\n")
        end
        local forward = inspect(211993, 221125)
        local reverse = inspect(221125, 211993)
        assert(forward ~= reverse, "a constant Delta result cannot follow swapped source and equipment")
        "#,
    )
    .expect("pinned tooltip comparison manager renders both supplied comparison pairs");
}
