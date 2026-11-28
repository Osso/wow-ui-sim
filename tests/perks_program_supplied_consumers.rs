#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::loader::{find_toc_file, load_addon};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

const SUPPLIED_VENDOR_ITEMS: &str = r#"[
    {"name":"Probe item one","perksVendorCategoryID":1,"description":"First item","timeRemaining":0,"purchased":false,"refundable":false,"subItemsLoaded":true,"isPurchasePending":false,"doesNotExpire":true,"price":100,"originalPrice":null,"showSaleBanner":false,"perksVendorItemID":101,"itemID":1001,"iconTexture":"Interface/Icons/INV_Misc_QuestionMark","mountID":0,"mountTypeName":"","speciesID":0,"transmogSetID":0,"itemModifiedAppearanceID":0,"subItems":[],"uiGroupInfo":null,"invType":"","quality":1},
    {"name":"Probe item two","perksVendorCategoryID":1,"description":"Second item","timeRemaining":0,"purchased":false,"refundable":false,"subItemsLoaded":true,"isPurchasePending":false,"doesNotExpire":true,"price":200,"originalPrice":null,"showSaleBanner":false,"perksVendorItemID":202,"itemID":1002,"iconTexture":"Interface/Icons/INV_Misc_QuestionMark","mountID":0,"mountTypeName":"","speciesID":0,"transmogSetID":0,"itemModifiedAppearanceID":0,"subItems":[],"uiGroupInfo":null,"invType":"","quality":1}
]"#;

const SUPPLIED_CHEST_REWARDS: &str = r#"[
    {"rewardTypeID":1,"perksVendorItemID":101,"rewardAmount":1,"monthRewarded":"2026-11","activityMonthID":11,"thresholdOrderIndex":1},
    {"rewardTypeID":1,"perksVendorItemID":202,"rewardAmount":1,"monthRewarded":"2026-11","activityMonthID":11,"thresholdOrderIndex":2}
]"#;

const SECOND_CHEST_REWARD: &str = r#"[
    {"rewardTypeID":1,"perksVendorItemID":202,"rewardAmount":1,"monthRewarded":"2026-11","activityMonthID":11,"thresholdOrderIndex":2}
]"#;

const MONTHLY_ACTIVITIES: &str = r#"{
    "activePerksMonth":11,"displayMonthName":"November","secondsRemaining":3600,
    "activities":[{"ID":901,"activityName":"Completed activity","description":"",
        "thresholdContributionAmount":200,"completed":true,"inProgress":false,
        "supersedes":0,"uiPriority":0,"areAllConditionsMet":true,"conditions":[],
        "requirementsList":[],"criteriaList":[],"tagNames":[],"chatLink":""}],
    "thresholds":[
        {"thresholdOrderIndex":1,"currencyAwardAmount":100,"requiredContributionAmount":100,"pendingReward":false},
        {"thresholdOrderIndex":2,"currencyAwardAmount":200,"requiredContributionAmount":200,"pendingReward":false}],
    "tags":[],"pendingIDs":[],"themePrefix":"","trackedIDs":[]
}"#;

fn supply_perks_program_catalog(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.SetPerksProgramVendorItems([==[{SUPPLIED_VENDOR_ITEMS}]==])"
    ))
    .expect("A_Admin should accept the supplied Perks Program catalog");
}

fn supply_pending_chest_rewards(env: &WowLuaEnv) {
    supply_pending_chest_reward_snapshot(env, SUPPLIED_CHEST_REWARDS);
}

fn supply_pending_chest_reward_snapshot(env: &WowLuaEnv, rewards: &str) {
    env.exec(&format!(
        "A_Admin.SetPerksProgramPendingChestRewards([==[{rewards}]==])"
    ))
    .expect("A_Admin should accept supplied pending chest rewards");
}

fn load_monthly_activities_consumer() -> WowLuaEnv {
    let ui = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_EncounterJournal"))
        .expect("Encounter Journal TOC should resolve");
    let toc = TocFile::from_file(&toc_path).expect("Encounter Journal TOC should parse");
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(toc.files.iter().any(|file| {
        file == std::path::Path::new("Mainline/Blizzard_MonthlyActivities.lua")
    }));

    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.set_screen_size(1024.0, 768.0);
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    load_addon(&env.loader_env(), &toc_path).expect("selected Encounter Journal should load");
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn pending_reward_glow_count(env: &WowLuaEnv) -> i64 {
    env.eval(
        r#"
        local count = 0
        for _, threshold in ipairs(MonthlyActivitiesFrame.thresholdFrames) do
            if threshold.RewardCurrency.PendingGlow:IsShown() then
                count = count + 1
            end
        end
        return count
        "#,
    )
    .expect("original threshold glow count should be readable")
}

#[cfg(feature = "client-retail")]
#[test]
fn supplied_catalog_and_chest_state_are_visible() {
    let env = crate::common::panel_fixtures::setup_env();
    env.exec("A_Admin.SetPerksProgramClientAvailable(true)")
        .expect("A_Admin should mark the Perks Program client available");
    let addons = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let toc = addons.join("Blizzard_PerksProgram/Blizzard_PerksProgram.toc");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc)
        .expect("the original Perks Program addon should load");
    env.exec("ShowUIPanel(PerksProgramFrame)")
        .expect("the original Perks Program panel should open through ShowUIPanel");
    supply_perks_program_catalog(&env);
    supply_pending_chest_rewards(&env);
    env.exec("A_Admin.SetPerksProgramCurrencyAmount(777)")
        .expect("A_Admin should supply Trader's Tender");

    let observed: (i64, bool, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "local requestReturns = select('#', C_PerksProgram.RequestPendingChestRewards())\n\
             local frame = PerksProgramFrame\n\
             local currency = frame.ProductsFrame.PerksProgramCurrencyFrame\n\
             return requestReturns, frame:IsShown(), frame.vendorItemIDs[1], frame.vendorItemIDs[2],\n\
                    C_PerksProgram.GetVendorItemInfo(101).price,\n\
                    C_PerksProgram.GetVendorItemInfo(202).price,\n\
                    currency.currencyAmount, #currency.pendingChestRewards",
        )
        .expect("original Perks Program frame should read the supplied state");
    assert_eq!(observed, (0, true, 101, 202, 100, 200, 777, 2));
}

#[cfg(feature = "client-wowforever")]
#[test]
fn supplied_catalog_and_chest_state_are_visible() {
    let env = crate::common::panel_fixtures::setup_env();
    supply_perks_program_catalog(&env);
    supply_pending_chest_rewards(&env);
    env.exec(
        "A_Admin.SetPerksProgramFeatureEnabled(true); A_Admin.SetPerksProgramClientAvailable(true)",
    )
    .expect("A_Admin should enable the Forever Perks Program client");

    let observed: (i64, i64, i64, i64, i64, i64, i64, i64) = env
        .eval(
            "local requestReturns = select('#', C_PerksProgram.RequestPendingChestRewards())\n\
             local ids = C_PerksProgram.GetAvailableVendorItemIDs()\n\
             local rewards = C_PerksProgram.GetPendingChestRewards()\n\
             return requestReturns, ids[1], ids[2], C_PerksProgram.GetVendorItemInfo(101).price,\n\
                    C_PerksProgram.GetVendorItemInfo(202).price,\n\
                    C_PerksProgram.GetCurrencyAmount(), #rewards, rewards[2].thresholdOrderIndex",
        )
        .expect("Perks Program should read the supplied state on Forever");
    assert_eq!(observed, (0, 101, 202, 100, 200, 0, 2, 2));
}

#[test]
fn pending_chest_rewards_drive_original_monthly_activity_glows() {
    let env = load_monthly_activities_consumer();
    env.exec(&format!("A_Admin.SetPerksActivities([==[{MONTHLY_ACTIVITIES}]==])"))
        .expect("A_Admin should accept the supplied activities and thresholds");
    env.exec("SetCVar('perksActivitiesLastPoints', '200'); MonthlyActivitiesFrame:Show()")
        .expect("original Monthly Activities frame should open");
    supply_pending_chest_rewards(&env);
    assert_eq!(pending_reward_glow_count(&env), 2);

    supply_pending_chest_reward_snapshot(&env, SECOND_CHEST_REWARD);
    assert_eq!(pending_reward_glow_count(&env), 1);
    let source: String = env
        .eval("return debug.getinfo(MonthlyActivitiesFrame:GetScript('OnEvent'), 'S').source")
        .expect("original event handler source should be readable");
    assert!(source.contains("Blizzard_MonthlyActivities.lua"));
}
