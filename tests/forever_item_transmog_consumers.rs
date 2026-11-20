#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

#[test]
fn transmog_outfit_consumer_refreshes_from_two_created_outfits() {
    common::with_timeout(240, || {
        let ui = blizzard_ui_dir();
        let env = new_blizzard_addon_env(&ui);
        let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_Transmog"], &[]);
        assert!(loaded.iter().any(|name| name == "Blizzard_Transmog"));
        env.apply_post_load_workarounds();
        let result: String = env.eval(r#"
            local refreshed
            local frame = { OutfitCollection = { Refresh = function(_, provider, select)
                refreshed = { provider = provider, select = select }
            end } }
            TransmogFrameMixin.RefreshOutfits(frame, false)
            C_TransmogOutfitInfo.AddNewOutfit("Copper", 123)
            C_TransmogOutfitInfo.AddNewOutfit("Silver", 456)
            TransmogFrameMixin.RefreshOutfits(frame, true)
            local collection = refreshed.provider:GetCollection()
            if refreshed.select ~= true or #collection ~= 2 then return "count" end
            if collection[1].name ~= "Copper" or collection[1].icon ~= 123 then return "first" end
            if collection[2].name ~= "Silver" or collection[2].icon ~= 456 then return "second" end
            return "ok"
        "#).expect("pinned TransmogFrameMixin consumer should run");
        assert_eq!(result, "ok");
    });
}

fn owned(item_id: u32, identity: u64) -> BagItem {
    BagItem {
        owned_identity: Some(identity), item_id, stack_count: 1, hyperlink: None,
        durability: None, applied_transmog: None,
    }
}

prefork_full_ui_case! {
fn secure_target_consumer_drops_two_items_into_trade(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.insert((0, 1), owned(6949, 101));
        sim.bag_items.insert((0, 2), owned(6950, 102));
    }
    env.exec(r#"
        local button = CreateFrame("Button", nil, UIParent)
        button:SetAttribute("type", "target")
        button:SetAttribute("unit", "player")
        InitiateTrade("player")
        for slot = 1, 2 do
            C_Container.PickupContainerItem(0, slot)
            SecureActionButton_OnClick(button, "LeftButton", false, false, true)
            assert(not CursorHasItem())
        end
    "#).unwrap();
    assert_eq!(env.state().borrow().active_trade.as_ref().unwrap().player_slots[..2], [6949, 6950]);
}
}
