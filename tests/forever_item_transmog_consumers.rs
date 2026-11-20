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
            IconSelectorPopupFrameTemplateMixin.OkayButton_OnClick = function() end
            local popup = { mode = IconSelectorPopupFrameModes.New, BorderBox = {
                SelectedIconArea = { SelectedIconButton = { GetIconTexture = function() return 123 end } },
                IconSelectorEditBox = { GetText = function() return "Copper" end },
            } }
            TransmogOutfitPopupMixin.OkayButton_OnClick(popup)
            popup.BorderBox.SelectedIconArea.SelectedIconButton.GetIconTexture = function() return 456 end
            popup.BorderBox.IconSelectorEditBox.GetText = function() return "Silver" end
            TransmogOutfitPopupMixin.OkayButton_OnClick(popup)
            local copper = C_TransmogOutfitInfo.GetOutfitInfoByName("Copper")
            local silver = C_TransmogOutfitInfo.GetOutfitInfoByName("Silver")
            if not copper or copper.name ~= "Copper" or copper.icon ~= 123 then return "first" end
            if not silver or silver.name ~= "Silver" or silver.icon ~= 456 then return "second" end
            return "ok"
        "#).expect("pinned TransmogOutfitPopupMixin consumer should run");
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
