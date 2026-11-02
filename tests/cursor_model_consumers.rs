//! Original Retail consumers of the shared cursor model: MainMenuBarBagManager
//! reads CURSOR_CHANGED's new cursor type, and ActionBarMixin follows the grid.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

prefork_full_ui_case! {
fn cursor_type_and_grid_reach_bag_bar_and_action_bar(env: &WowLuaEnv) {
    env.state().borrow_mut().bag_items.insert((0, 1), BagItem {
        item_id: 6948,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    });
    env.exec(r#"
        local source = debug.getinfo(MainMenuBarBagManager.OnCursorChanged, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_MainMenuBarBagButtons/Shared/MainMenuBarBagManager.lua', 1, true), source)
        source = debug.getinfo(MainActionBar.SetShowGrid, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_ActionBar/Shared/ActionBar.lua', 1, true), source)
        local function grid()
            return bit.band(MainActionBar.showAllButtons or 0, ACTION_BUTTON_SHOW_GRID_REASON_EVENT) ~= 0
        end
        local function expanded()
            return MainMenuBarBagManager.expandBarAuto == true
        end

        ClearCursor()
        C_Container.PickupContainerItem(0, 1)
        assert(expanded(), 'an owned item expands the bag bar')
        assert(not grid(), 'owned items do not show the grid')
        C_Item.PickupItem(6948)
        assert(not expanded(), 'an ActionBar cursor is not a bag cursor')
        assert(grid(), 'an ActionBar pickup shows the grid')
        PickupMerchantItem(2)
        assert(expanded(), 'a merchant cursor expands the bag bar')
        assert(not grid(), 'clearing the ActionBar cursor hides the grid')
        C_Spell.PickupSpell(853)
        assert(not expanded() and grid())
        ClearCursor()
        assert(not expanded() and not grid())
    "#).unwrap();
}
}
