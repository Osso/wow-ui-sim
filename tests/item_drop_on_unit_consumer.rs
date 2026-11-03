//! Original SecureTemplates target action drops the cursor item into trade (WM68).
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

fn owned(item_id: u32, identity: u64) -> BagItem {
    BagItem {
        owned_identity: Some(identity),
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}

prefork_full_ui_case! {
fn secure_target_action_fills_ordinary_trade_slots_in_order(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.active_trade = None;
        for slot in 1..=8 {
            sim.bag_items.insert((0, slot), owned(6948 + slot as u32, 100 + slot as u64));
        }
    }
    env.exec(r#"
        local source = debug.getinfo(SecureActionButton_OnClick, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_FrameXML/SecureTemplates.lua', 1, true), source)
        local button = CreateFrame('Button', nil, UIParent)
        button:SetAttribute('type', 'target')
        button:SetAttribute('unit', 'player')
        local function action()
            SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        end
        tradeEvents = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('TRADE_PLAYER_ITEM_CHANGED')
        listener:SetScript('OnEvent', function(_, _, ...)
            tradeEvents[#tradeEvents + 1] = select('#', ...) .. ':' .. tostring((...))
        end)
        InitiateTrade('player')
        for slot = 1, 6 do
            C_Container.PickupContainerItem(0, slot)
            action()
            assert(not CursorHasItem(), 'slot ' .. slot)
        end
        assert(table.concat(tradeEvents, ',') == '1:1,1:2,1:3,1:4,1:5,1:6', table.concat(tradeEvents, ','))
        -- No ordinary slot is free: the cursor stays and no event fires.
        C_Container.PickupContainerItem(0, 7)
        action()
        assert(CursorHasItem())
        assert(#tradeEvents == 6)
    "#).unwrap();
    let sim = env.state().borrow();
    let trade = sim.active_trade.as_ref().unwrap();
    assert_eq!(trade.player_slots, [6949, 6950, 6951, 6952, 6953, 6954, 0]);
}
}

prefork_full_ui_case! {
fn drop_requires_an_owned_item_and_the_trade_target(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.active_trade = None;
        sim.bag_items.insert((0, 1), owned(6948, 201));
    }
    env.exec(r#"
        InitiateTrade('player')
        -- An item-ID cursor (mode 6) supplies no owned item.
        C_Item.PickupItem(6948)
        C_Item.DropItemOnUnit('player')
        assert(select(2, GetCursorInfo()) == 6948, 'item-ID cursor retained')
        ClearCursor()
        -- A unit that does not resolve is not the trade target.
        C_Container.PickupContainerItem(0, 1)
        C_Item.DropItemOnUnit('party4')
        assert(CursorHasItem(), 'unresolved unit retains the cursor')
        C_Item.DropItemOnUnit('player')
        assert(not CursorHasItem(), 'trade target takes the item')
    "#).unwrap();
    assert_eq!(
        env.state().borrow().active_trade.as_ref().unwrap().player_slots[0],
        6948
    );
}
}
