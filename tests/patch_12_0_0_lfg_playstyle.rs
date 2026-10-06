use super::patch_12_0_0_struct_shapes::assert_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn lfg_general_playstyle_roundtrip_and_parent_shapes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        ListingInput = {activityIDs = {1195, 1240}, questID = 901,
            isAutoAccept = true, isCrossFactionListing = true, isPrivateGroup = true,
            newPlayerFriendly = true, playstyle = Enum.LFGEntryPlaystyle.Standard,
            generalPlaystyle = Enum.LFGEntryGeneralPlaystyle.Learning,
            requiredDungeonScore = 1234, requiredItemLevel = 600, requiredPvpRating = 1700}
        assert(C_LFGList.CreateListing(ListingInput))
        FirstEntry = C_LFGList.GetActiveEntryInfo()
        assert(FirstEntry.generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Learning)
        assert(FirstEntry.activityIDs[2] == 1240 and FirstEntry.questID == 901)
        assert(FirstEntry.autoAccept and FirstEntry.isCrossFactionListing and FirstEntry.privateGroup)
        assert(FirstEntry.newPlayerFriendly and FirstEntry.requiredDungeonScore == 1234)
        assert(FirstEntry.requiredItemLevel == 600 and FirstEntry.requiredPvpRating == 1700)
        ListingInput.generalPlaystyle = Enum.LFGEntryGeneralPlaystyle.Expert
        assert(C_LFGList.UpdateListing(ListingInput))
        local second = C_LFGList.GetActiveEntryInfo()
        assert(second.generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Expert)
        assert(FirstEntry.generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Learning)
        FirstEntry.activityIDs[1] = -1
        assert(second.activityIDs[1] == 1195)
        ListingInput.generalPlaystyle = 1.5
        assert(not pcall(C_LFGList.UpdateListing, ListingInput), 'fractional enum must reject')
        ListingInput.generalPlaystyle = 99
        assert(not pcall(C_LFGList.UpdateListing, ListingInput), 'unknown enum must reject')
        assert(C_LFGList.GetActiveEntryInfo().generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Expert)
        ListingInput.generalPlaystyle = Enum.LFGEntryGeneralPlaystyle.Expert
    "#).unwrap();
    env.process_timers().unwrap();
    assert_shape(&env, "LFGListInfoDocumentation.lua", "LfgListingCreateData", "return ListingInput");
    assert_shape(&env, "LFGListInfoDocumentation.lua", "LfgEntryData", "return C_LFGList.GetActiveEntryInfo()");
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.world.premade_listings[0].metadata = wow_ui_sim::c_api::c_lfg_list_search::SearchMetadata {
            has_self: true,
            dungeon_scores: vec![wow_ui_sim::c_api::c_lfg_list_search::DungeonScore {
                map_score: 250.5, map_name: "Fixture dungeon".into(), best_run_level: 12,
                finished_success: true, best_run_duration_ms: 900000.0, best_level_increment: 2,
            }],
            pvp_ratings: vec![wow_ui_sim::c_api::c_lfg_list_search::PvpRating {
                bracket: 2, rating: 1700, activity_name: "Fixture arena".into(), tier: 3,
            }],
        };
        state.world.premade_listings[0].general_playstyle = 1;
        state.world.premade_listings[1].general_playstyle = 4;
        state.world.premade_listings[2].general_playstyle = 0;
    }
    assert_shape(&env, "LFGListInfoDocumentation.lua", "LfgSearchResultData", "return C_LFGList.GetSearchResultInfo(1)");
    env.exec(r#"
        local first = C_LFGList.GetSearchResultInfo(1)
        local second = C_LFGList.GetSearchResultInfo(2)
        assert(first.hasSelf == true and second.hasSelf == false)
        assert(first.leaderDungeonScoreInfo[1].mapScore == 250.5)
        assert(first.leaderDungeonScoreInfo[1].mapName == 'Fixture dungeon')
        assert(first.leaderDungeonScoreInfo[1].bestRunLevel == 12)
        assert(first.leaderDungeonScoreInfo[1].finishedSuccess == true)
        assert(first.leaderDungeonScoreInfo[1].bestRunDurationMs == 900000)
        assert(first.leaderDungeonScoreInfo[1].bestLevelIncrement == 2)
        assert(first.leaderPvpRatingInfo[1].bracket == 2 and first.leaderPvpRatingInfo[1].rating == 1700)
        assert(first.leaderPvpRatingInfo[1].activityName == 'Fixture arena' and first.leaderPvpRatingInfo[1].tier == 3)
        assert(#second.leaderDungeonScoreInfo == 0 and #second.leaderPvpRatingInfo == 0)
        first.leaderDungeonScoreInfo[1].mapName = 'mutation'
        assert(C_LFGList.GetSearchResultInfo(1).leaderDungeonScoreInfo[1].mapName == 'Fixture dungeon')
        assert(first.generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Learning)
        assert(second.generalPlaystyle == Enum.LFGEntryGeneralPlaystyle.Expert)
        first.generalPlaystyle = 3
        assert(C_LFGList.GetSearchResultInfo(1).generalPlaystyle == 1)
        assert(second.generalPlaystyle == 4)
        assert(C_LFGList.GetSearchResultInfo(3).generalPlaystyle == nil)
        assert(C_LFGList.GetSearchResultInfo(99999) == nil)
        assert(C_LFGList.UpdateListing({activityIDs = {1195}}))
        assert(C_LFGList.GetActiveEntryInfo().generalPlaystyle == nil)
    "#).unwrap();
}
