//! Original SecureTemplates item action drives the WM57 equip request.
#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::item_spell::item_equip::OwnedItemEquipInputs;
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

prefork_full_ui_case! {
fn paper_doll_cursor_changed_highlights_compatible_item(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.bag_items.insert((0, 2), BagItem {
            item_id: 25, stack_count: 1, hyperlink: None,
            durability: None, applied_transmog: None, owned_identity: Some(51),
        });
    }
    env.exec(r#"
        local source = debug.getinfo(PaperDollItemSlotButton_OnEvent, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua', 1, true), source)
        local button = { GetID = function() return 16 end,
            LockHighlight = function(self) self.highlighted = true end,
            UnlockHighlight = function(self) self.highlighted = false end }
        PaperDollItemSlotButton_OnEvent(button, 'CURSOR_CHANGED')
        assert(button.highlighted == false)
        C_Container.PickupContainerItem(0, 2)
        PaperDollItemSlotButton_OnEvent(button, 'CURSOR_CHANGED')
        assert(button.highlighted == true)
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn binding_confirmation_reaches_the_original_game_event_consumer(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.clear();
        sim.player.equipped_items.clear();
        sim.bag_items.insert((0, 2), BagItem {
            item_id: 25, stack_count: 1, hyperlink: None,
            durability: None, applied_transmog: None, owned_identity: Some(51),
        });
        sim.item_equip.inputs.insert(51, OwnedItemEquipInputs {
            confirmation_eligible: Some(true),
            ..Default::default()
        });
    }
    env.exec(r#"
        local source = debug.getinfo(GameEvent.HandleEquipBindConfirm, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_Game/Mainline/EventImplementation.lua', 1, true), source)
        local previous = StaticPopup_Show
        bindCaptures = {}
        StaticPopup_Show = function(which, text1, text2, data)
            if which == 'EQUIP_BIND' then
                bindCaptures[#bindCaptures + 1] = data
                return nil
            end
            return previous(which, text1, text2, data)
        end
        C_Item.EquipItemByName(25)
        assert(#bindCaptures == 1)
        local data = bindCaptures[1]
        assert(data.slot == 0)
        assert(data.itemLocation.bagID == 0 and data.itemLocation.slotIndex == 2)
        assert(data.itemLocation:IsBagAndSlot())
        assert(CursorHasItem() and not C_Container.GetContainerItemInfo(0, 2).isLocked)
    "#).unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let item = sim.bag_items.remove(&(0, 2)).unwrap();
        sim.bag_items.insert((0, 3), item);
    }
    env.exec(r#"
        ClearCursor()
        C_Item.PickupItem(6948)
        C_Item.EquipItemByName(25, 16)
        assert(#bindCaptures == 2)
        local data = bindCaptures[2]
        assert(data.slot == 1)
        assert(data.itemLocation.bagID == 0 and data.itemLocation.slotIndex == 3)
        assert(select(2, GetCursorInfo()) == 6948)
    "#).unwrap();
    assert!(env.state().borrow().item_equip.requests.is_empty());
}
}
