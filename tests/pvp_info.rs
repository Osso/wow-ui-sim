use wow_ui_sim::lua_api::WowLuaEnv;

const PVP_INFO_SCRIPT: &str = r#"
    local worldOne = C_PvP.GetWorldPVPAreaInfo(1)
    if not worldOne or worldOne.name ~= "Wintergrasp" or worldOne.canEnter ~= true or worldOne.canQueue ~= true or worldOne.isActive ~= true or worldOne.minLevel ~= 80 or worldOne.startTime ~= 900 then
        return "wrong_world_area_one"
    end

    local worldTwo = C_PvP.GetWorldPVPAreaInfo(2)
    if not worldTwo or worldTwo.name ~= "Tol Barad" or worldTwo.canQueue ~= false or worldTwo.isActive ~= false then
        return "wrong_world_area_two"
    end

    if C_PvP.GetWorldPVPAreaInfo(99) ~= nil then
        return "unexpected_world_area"
    end

    if GetNumBattlegroundTypes() < 5 then
        return "missing_battleground_rows"
    end

    -- The legacy tuple is absent on modern retail; its existing assertions
    -- remain active on profiles retaining the global.
    if GetBattlegroundInfo ~= nil then
    local name, canEnter, isHoliday, isRandom, bgID, description, mapID, maxPlayers, gameType, iconTexture, _, _, hasControllingHoliday = GetBattlegroundInfo(1)
    if name ~= "Wintergrasp" or canEnter ~= true or isRandom ~= false or bgID ~= 571 or mapID ~= 571 or maxPlayers ~= 40 or hasControllingHoliday ~= 1 then
        return "wrong_world_battleground_row"
    end

    name, canEnter, isHoliday, isRandom, bgID, description, mapID, maxPlayers, gameType, iconTexture, _, _, hasControllingHoliday = GetBattlegroundInfo(3)
    if name ~= "Warsong Gulch" or canEnter ~= true or isRandom ~= false or bgID ~= 2 or mapID ~= 489 or maxPlayers ~= 10 or hasControllingHoliday ~= 0 then
        return "wrong_specific_battleground_row"
    end

    if GetBattlegroundInfo(99) ~= nil then
        return "unexpected_battleground_row"
    end
    end

    local holiday = C_PvP.GetHolidayBGInfo()
    if not holiday or holiday.bgID ~= 108 or holiday.bgIndex ~= 2 or holiday.name ~= "Warsong Scramble" or holiday.canQueue ~= true or holiday.minLevel ~= 10 then
        return "wrong_holiday_bg"
    end

    if C_PvP.GetLocklistMap(1) ~= 0 or C_PvP.GetLocklistMapName(1) ~= nil then
        return "locklist_not_empty"
    end

    C_PvP.SetLocklistMap(566)
    if C_PvP.GetLocklistMap(1) ~= 566 or C_PvP.GetLocklistMapName(1) ~= "Eye of the Storm" then
        return "wrong_first_locklist_entry"
    end

    C_PvP.SetLocklistMap(727)
    if C_PvP.GetLocklistMap(2) ~= 727 or C_PvP.GetLocklistMapName(2) ~= "Silvershard Mines" then
        return "wrong_second_locklist_entry"
    end

    C_PvP.SetLocklistMap(566)
    if C_PvP.GetLocklistMap(1) ~= 566 or C_PvP.GetLocklistMap(2) ~= 727 then
        return "duplicate_locklist_inserted"
    end

    C_PvP.ClearLocklistMap(566)
    if C_PvP.GetLocklistMap(1) ~= 727 or C_PvP.GetLocklistMapName(1) ~= "Silvershard Mines" or C_PvP.GetLocklistMap(2) ~= 0 or C_PvP.GetLocklistMapName(2) ~= nil then
        return "wrong_locklist_after_clear"
    end

    C_PvP.ClearLocklistMap(727)
    if C_PvP.GetLocklistMap(1) ~= 0 or C_PvP.GetLocklistMap(2) ~= 0 then
        return "locklist_not_cleared"
    end

    return "ok"
"#;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn pvp_info_methods_use_seeded_world_and_locklist_state() {
    let env = env();
    let result: String = env
        .eval(PVP_INFO_SCRIPT)
        .expect("C_PvP methods should be queryable");
    assert_eq!(result, "ok");
}

#[test]
fn get_personal_rated_info_returns_retail_shaped_inert_tuple() {
    let env = env();

    let (count, rating, season_best, weekly_best, has_won, pvp_tier_is_nil): (
        i32,
        i32,
        i32,
        i32,
        bool,
        bool,
    ) = env
        .eval(
            r##"
            local rating, seasonBest, weeklyBest, seasonPlayed, seasonWon,
                  weeklyPlayed, weeklyWon, lastWeeksBest, hasWon, pvpTier,
                  ranking, roundsSeasonPlayed, roundsSeasonWon,
                  roundsWeeklyPlayed, roundsWeeklyWon = GetPersonalRatedInfo(1)
            return select("#", GetPersonalRatedInfo(1)),
                   rating,
                   seasonBest,
                   weeklyBest,
                   hasWon,
                   pvpTier == nil
            "##,
        )
        .unwrap();

    assert_eq!(count, 15);
    assert_eq!((rating, season_best, weekly_best), (0, 0, 0));
    assert!(!has_won);
    assert!(pvp_tier_is_nil);
}
