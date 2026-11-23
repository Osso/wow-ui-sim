#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const SUPPLIED_VENDOR_ITEMS: &str = r#"[
    {"name":"Probe item one","perksVendorCategoryID":1,"description":"First item","timeRemaining":0,"purchased":false,"refundable":false,"subItemsLoaded":true,"isPurchasePending":false,"doesNotExpire":true,"price":100,"originalPrice":null,"showSaleBanner":false,"perksVendorItemID":101,"itemID":1001,"iconTexture":"Interface/Icons/INV_Misc_QuestionMark","mountID":0,"mountTypeName":"","speciesID":0,"transmogSetID":0,"itemModifiedAppearanceID":0,"subItems":[],"uiGroupInfo":null,"invType":"","quality":1},
    {"name":"Probe item two","perksVendorCategoryID":1,"description":"Second item","timeRemaining":0,"purchased":false,"refundable":false,"subItemsLoaded":true,"isPurchasePending":false,"doesNotExpire":true,"price":200,"originalPrice":null,"showSaleBanner":false,"perksVendorItemID":202,"itemID":1002,"iconTexture":"Interface/Icons/INV_Misc_QuestionMark","mountID":0,"mountTypeName":"","speciesID":0,"transmogSetID":0,"itemModifiedAppearanceID":0,"subItems":[],"uiGroupInfo":null,"invType":"","quality":1}
]"#;

const SUPPLIED_CHEST_REWARDS: &str = r#"[
    {"rewardTypeID":1,"perksVendorItemID":101,"rewardAmount":1,"monthRewarded":"2026-11","activityMonthID":11,"thresholdOrderIndex":1},
    {"rewardTypeID":1,"perksVendorItemID":202,"rewardAmount":1,"monthRewarded":"2026-11","activityMonthID":11,"thresholdOrderIndex":2}
]"#;

fn supply_perks_program_state(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.SetPerksProgramVendorItems([==[{SUPPLIED_VENDOR_ITEMS}]==])\n\
         A_Admin.SetPerksProgramPendingChestRewards([==[{SUPPLIED_CHEST_REWARDS}]==])"
    ))
    .expect("A_Admin should accept supplied Perks Program catalog and rewards");
}

#[cfg(feature = "client-retail")]
#[test]
fn supplied_catalog_and_chest_state_are_visible() {
    let env = crate::common::panel_fixtures::setup_env();
    supply_perks_program_state(&env);
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let toc = addons.join("Blizzard_PerksProgram/Blizzard_PerksProgram.toc");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
        .expect("the original Perks Program addon should load");
    env.exec("ShowUIPanel(PerksProgramFrame)")
        .expect("the original Perks Program panel should open through ShowUIPanel");
    env.exec("A_Admin.SetPerksProgramCurrencyAmount(777)")
        .expect("A_Admin should supply Trader's Tender");

    let observed: (bool, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "local frame = PerksProgramFrame\n\
             local currency = frame.ProductsFrame.PerksProgramCurrencyFrame\n\
             return frame:IsShown(), frame.vendorItemIDs[1], frame.vendorItemIDs[2],\n\
                    C_PerksProgram.GetVendorItemInfo(101).price,\n\
                    C_PerksProgram.GetVendorItemInfo(202).price,\n\
                    currency.currencyAmount, #currency.pendingChestRewards",
        )
        .expect("original Perks Program frame should read the supplied state");
    assert_eq!(observed, (true, 101, 202, 100, 200, 777, 2));
}

#[cfg(feature = "client-wowforever")]
#[test]
fn supplied_catalog_and_chest_state_are_visible() {
    let env = crate::common::panel_fixtures::setup_env();
    supply_perks_program_state(&env);

    let observed: (i64, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "local ids = C_PerksProgram.GetAvailableVendorItemIDs()\n\
             local rewards = C_PerksProgram.GetPendingChestRewards()\n\
             return ids[1], ids[2], C_PerksProgram.GetVendorItemInfo(101).price,\n\
                    C_PerksProgram.GetVendorItemInfo(202).price,\n\
                    C_PerksProgram.GetCurrencyAmount(), #rewards, rewards[2].thresholdOrderIndex",
        )
        .expect("Perks Program should read the supplied state on Forever");
    assert_eq!(observed, (101, 202, 100, 200, 0, 2, 2));
}
