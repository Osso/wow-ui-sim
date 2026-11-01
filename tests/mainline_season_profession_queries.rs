//! Mainline-family queries reached by Blizzard_EncounterJournal, the housing
//! editor and tutorials, and the profession frames.
#![cfg(feature = "profile-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const PROBE: &str = r#"
    return table.concat({
        tostring(C_SeasonInfo.GetCurrentDisplaySeasonID()),
        tostring(C_Housing.IsHousingMarketEnabled()),
        tostring(C_QuestLog.IsQuestFlaggedCompletedOnAccount(91877)),
        tostring(C_CraftingOrders.HasFavoriteCustomerOptions()),
        tostring(C_TradeSkillUI.IsAnyRecipeFromSource(1)),
    }, "|")
"#;

#[test]
fn mainline_queries_report_configured_season_market_and_account_completion() {
    let env = WowLuaEnv::new().expect("create mainline query environment");
    let before: String = env.eval(PROBE).expect("query defaults");
    assert_eq!(before, "0|false|false|false|false");

    {
        let mut sim = env.state().borrow_mut();
        sim.mythic_plus.display_season_id = 17;
        sim.housing.market_enabled = true;
        sim.quest_log_entries.completed_quest_ids.insert(91877);
    }
    let after: String = env.eval(PROBE).expect("query configured state");
    assert_eq!(after, "17|true|true|false|false");
}
