#![cfg(feature = "client-mists")]

use wow_ui_sim::c_api::c_trade_skill_quality::CraftingQualityInfo;

#[path = "common/blizzard_addon_closure.rs"]
mod closure;

#[test]
fn auction_consumer_observes_two_supplied_reagent_qualities() {
    let ui = closure::blizzard_ui_dir();
    let (env, loaded) =
        closure::build_blizzard_addon_closure_env(&ui, &["Blizzard_AuctionHouseUI"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_AuctionHouseUI"));
    for (item, quality, icon) in [(900001, 1, "first-chat"), (900002, 2, "second-chat")] {
        env.state()
            .borrow_mut()
            .recipe_quality_inputs
            .reagent_item_quality
            .insert(
                item,
                CraftingQualityInfo {
                    quality,
                    icon_chat: icon.into(),
                    icon_quest_objective: "not-in-mists".into(),
                    ..Default::default()
                },
            );
    }
    env.exec(r#"
        local update = AuctionHouseTableCellItemDisplayMixin.UpdateDisplay
        local source = debug.getinfo(update, 'S').source
        assert(source:find('Blizzard_AuctionHouseTableBuilder.lua', 1, true), source)
        local function text() return { SetText = function(self, value) self.value = value end, SetPoint = function() end } end
        local cell = { rowData = { totalQuantity = 1 }, Text = text(), ExtraInfo = text(),
            Icon = { SetTexture = function() end, Show = function() end, SetAlpha = function() end },
            HandleItemNameTruncation = function() end }
        local info = { itemName = 'Reagent', quality = 1, iconFileID = 123 }
        update(cell, { itemID = 900001 }, info)
        assert(cell.ExtraInfo.value:find('first-chat', 1, true))
        update(cell, { itemID = 900002 }, info)
        assert(cell.ExtraInfo.value:find('second-chat', 1, true))
        local q = C_TradeSkillUI.GetItemReagentQualityInfo(900001)
        assert(q.quality == 1 and q.iconQuestObjective == nil)
    "#).expect("original auction consumer must read supplied quality records");
}
