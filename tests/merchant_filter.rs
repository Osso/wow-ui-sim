#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, new_blizzard_addon_env};
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_startup_addons_for_screen, load_startup_addon,
    register_load_with_addons,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::MerchantCostRecord;
use wow_ui_sim::screen::ScreenKind;

fn merchant_game_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    register_load_with_addons(&env.loader_env(), &discover_all_blizzard_addons(&ui));
    let startup = discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game);
    for addon in startup {
        let loaded = env.state().borrow().addons.iter().any(|entry| {
            entry.folder_name == addon.name && entry.is_loaded_in_environment(entry.use_secure_env)
        });
        if !loaded {
            load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
                .unwrap_or_else(|error| panic!("load startup {}: {error}", addon.name));
        }
        if addon.name == "Blizzard_EnvironmentCleanup" {
            env.restore_post_cleanup_globals();
        }
    }
    env.sync_string_metatable_to_global_string();
    env
}

fn supply_merchant_offers(env: &WowLuaEnv) {
    use wow_ui_sim::lua_api::globals::real::merchant_item_link::{
        MerchantLinkOffer, MerchantLinks,
    };

    let mut sim = env.state().borrow_mut();
    sim.merchant_items = vec![6948, 1728];
    sim.merchant_offer_costs.insert(
        0,
        MerchantCostRecord {
            price: 123,
            ..MerchantCostRecord::default()
        },
    );
    sim.merchant_offer_costs.insert(
        1,
        MerchantCostRecord {
            price: 456,
            ..MerchantCostRecord::default()
        },
    );
    sim.merchant_links = MerchantLinks::default();
    for slot in 0..2 {
        sim.merchant_links.offers.insert(
            slot,
            MerchantLinkOffer::Item {
                fields: Some(vec![0, 80, 0, 7]),
            },
        );
    }
    sim.merchant_frame_open = true;
    sim.player.present = true;
}

#[test]
fn merchant_dropdown_filters_rendered_rows_by_boe() {
    let env = merchant_game_env();
    supply_merchant_offers(&env);
    env.exec(
        r#"
        MerchantFrame.selectedTab = 1
        MerchantFrame.lastTab = 1
        MerchantFrame.page = 1
        MerchantFrame_Update()
        assert(GetMerchantNumItems() == 2, "all filter must show both supplied offers")
        assert(MerchantItem1Name:GetText() == "Hearthstone")
        assert(MerchantItem1ItemButton.price == 123)
        assert(MerchantItem1ItemButton.link == GetMerchantItemLink(1))
        assert(MerchantItem1ItemButton.link:find("|Hitem:6948:", 1, true))
        assert(MerchantItem2Name:GetText() == "Teebu's Blazing Longsword")
        assert(MerchantItem2ItemButton.price == 456)
        assert(MerchantItem2ItemButton.link == GetMerchantItemLink(2))
        assert(MerchantItem2ItemButton.link:find("|Hitem:1728:", 1, true))
        assert(C_MerchantFrame.IsMerchantItemRefundable(1) == false)
        MerchantFrame.FilterDropdown:Show()
        MerchantFrame.FilterDropdown:GenerateMenu()
        MerchantFrame.FilterDropdown:OpenMenu()
        local menu = Menu.GetManager():GetOpenMenu()
        assert(menu, "original merchant filter menu must open")
        local selected
        for _, child in ipairs({menu:GetChildren()}) do
            local text = child.fontString and child.fontString:GetText()
            if text == ITEM_BIND_ON_EQUIP then selected = child end
        end
        assert(selected, "original merchant menu must include the BoE filter")
        selected:Click()
        assert(GetMerchantFilter() == LE_LOOT_FILTER_BOE, "dropdown must select BoE")
        assert(GetMerchantNumItems() == 1, "BoE filter must reduce the visible row count")
        assert(C_MerchantFrame.GetItemInfo(1).name == "Teebu's Blazing Longsword")
        assert(C_MerchantFrame.GetItemInfo(1).price == 456)
        assert(GetMerchantItemID(1) == 1728)
        assert(GetMerchantItemLink(1):find("|Hitem:1728:", 1, true))
        assert(C_MerchantFrame.IsMerchantItemRefundable(1) == false)
        assert(MerchantItem1Name:GetText() == "Teebu's Blazing Longsword")
        assert(MerchantItem1ItemButton.name == "Teebu's Blazing Longsword")
        assert(MerchantItem1ItemButton.price == 456)
        assert(MerchantItem1ItemButton.link == GetMerchantItemLink(1))
        "#,
    )
    .expect("original merchant dropdown must render the selected filtered offer");
}
