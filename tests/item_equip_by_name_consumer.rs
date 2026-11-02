//! Original SecureTemplates item action drives the WM57 equip request.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

prefork_full_ui_case! {
fn secure_item_action_requests_equip_and_exposes_the_lock(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert((0, 3), BagItem {
            item_id: 7005, stack_count: 1, hyperlink: None,
            durability: None, applied_transmog: None,
            owned_identity: None,
        });
        sim.equippable_items.insert(7005);
    }
    env.exec(r#"
        local source = debug.getinfo(SecureActionButton_OnClick, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_FrameXML/SecureTemplates.lua', 1, true), source)
        local listener = CreateFrame('Frame')
        local events = {}
        listener:RegisterEvent('ITEM_LOCK_CHANGED')
        listener:RegisterEvent('ITEM_LOCKED')
        listener:SetScript('OnEvent', function(_, name, bag, slot)
            events[#events + 1] = name .. ',' .. tostring(bag) .. ',' .. tostring(slot)
        end)
        local button = CreateFrame('Button', nil, UIParent)
        button:SetAttribute('type', 'item')
        button:SetAttribute('item', 'item:7005')
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(table.concat(events, ';') == 'ITEM_LOCK_CHANGED,0,3;ITEM_LOCKED,0,3', table.concat(events, ';'))
        assert(C_Container.GetContainerItemInfo(0, 3).isLocked)
        assert(C_Item.IsLocked(ItemLocation:CreateFromBagAndSlot(0, 3)))
        assert(C_Container.GetContainerItemID(0, 3) == 7005)
        assert(GetCursorInfo() == nil)
        -- Another request does not fire another lock transition.
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(#events == 2)
    "#).unwrap();
}
}
