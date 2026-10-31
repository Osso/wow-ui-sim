#![cfg(any(feature = "client-retail", feature = "client-ptr", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::state::EquippedItem;

fn item(item_id: u32) -> EquippedItem {
    EquippedItem { item_id, enchant_id: 0, gem_ids: [0; 3] }
}

#[test]
fn paper_doll_item_level_color_tracks_lowest_equipped_quality() {
    test_timeout! {
        let env = crate::common::panel_fixtures::setup_env();
        env.state().borrow_mut().player.level = 80;
        env.state().borrow_mut().player.equipped_items.clear();
        env.exec(r#"
            ToggleCharacter("PaperDollFrame")
            function AssertItemLevelColor(quality)
                PaperDollFrame_UpdateStats()
                local r, g, b = CharacterStatsPane.ItemLevelFrame.Value:GetTextColor()
                local expected = ITEM_QUALITY_COLORS[quality]
                assert(math.abs(r - expected.r) < 0.001 and math.abs(g - expected.g) < 0.001 and math.abs(b - expected.b) < 0.001, 'item-level color must match equipped quality ' .. quality)
            end
            AssertItemLevelColor(1)
        "#).unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.player.equipped_items.insert(16, item(159645)); // rare weapon
            state.player.equipped_items.insert(17, item(268211)); // epic weapon
        }
        env.exec("assert(GetInventoryItemQuality('player', 16) == 3); assert(GetInventoryItemQuality('player', 17) == 4); AssertItemLevelColor(3)").unwrap();
        env.state().borrow_mut().player.equipped_items.remove(&16);
        env.exec("AssertItemLevelColor(4)").unwrap();
        env.state().borrow_mut().player.equipped_items.insert(4, item(159645));
        env.exec("AssertItemLevelColor(4)").unwrap();
    }
}
