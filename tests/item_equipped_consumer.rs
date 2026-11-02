//! Original secure item action consumer for equipped-item state.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::EquippedItem;

prefork_full_ui_case! {
fn secure_item_action_uses_equipped_item_from_inventory_link(env: &WowLuaEnv) {
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.equipped_items.clear();
        sim.player.equipped_items.insert(16, EquippedItem {
            item_id: 6948,
            enchant_id: 0,
            gem_ids: [0; 3],
            applied_transmog: None,
            owned_identity: None,
        });
        sim.equippable_items.insert(6948);
    }
    env.exec(r#"
        local link = GetInventoryItemLink('player', 16)
        local name = SecureCmdItemParse(link)
        assert(type(name) == 'string' and name:find('item:6948', 1, true))
        assert(C_Item.IsEquippableItem(name))
        local used = 0
        SecureCmdUseItem = function(item) assert(item == link); used = used + 1 end
        local button = CreateFrame('Button', 'EquippedItemConsumerProbe', UIParent)
        button:SetAttribute('type', 'item')
        button:SetAttribute('item', link)
        SecureActionButton_OnClick(button, 'LeftButton', false, false, true)
        assert(used == 1)
    "#).unwrap();
    {
        let state = env.state();
        state.borrow_mut().player.equipped_items.get_mut(&16).unwrap().item_id = 6949;
    }
    env.exec("assert(not C_Item.IsEquippedItem('item:6948'))").unwrap();
}
}
