use wow_ui_sim::c_api::c_pvp::catalog::BattlegroundInfo;

/// Proves simulator compatibility globals preserve their tested behavior.
#[test]
fn simulator_legacy_compat_globals_preserve_tested_behavior() {
    let env = load_full_game_ui_with_all_lod();
    env.state().borrow_mut().pvp_catalog.battlegrounds = vec![
        BattlegroundInfo {
            name: "Wintergrasp".into(),
            icon: Some(0),
            game_type: "wintergrasp".into(),
            short_description: "short".into(),
            long_description: "long".into(),
            map_description: "map".into(),
            max_players: 40,
            battleground_id: Some(571),
            lfg_dungeon_id: Some(117),
            map_id: Some(571),
            is_holiday: false,
            is_random: false,
            can_enter: true,
            is_training_ground: false,
        },
        BattlegroundInfo {
            name: "Arena".into(),
            icon: Some(1300),
            game_type: "arena".into(),
            short_description: "arena-short".into(),
            long_description: "arena-long".into(),
            map_description: "arena-map".into(),
            max_players: 10,
            battleground_id: Some(559),
            lfg_dungeon_id: Some(120),
            map_id: Some(559),
            is_holiday: true,
            is_random: true,
            can_enter: false,
            is_training_ground: true,
        },
    ];
    let result: String = env
        .eval(
            r#"
            local required = {
                "FindBaseSpellByID",
                "FindFlyoutSlotBySpellID",
                "FindSpellOverrideByID",
                "PlaySound",
                "strtrim",
            }
            for _, name in ipairs(required) do
                if type(rawget(_G, name)) ~= "function" then
                    return "missing=" .. name
                end
            end

            local oldBase = C_SpellBook.FindBaseSpellByID
            local baseID
            C_SpellBook.FindBaseSpellByID = function(value)
                baseID = value
                return "base-sentinel"
            end
            local baseOK, baseResult = pcall(FindBaseSpellByID, 101)
            C_SpellBook.FindBaseSpellByID = oldBase
            if not baseOK or baseID ~= 101 or baseResult ~= "base-sentinel" then
                return "base-wrapper"
            end

            local oldFlyout = C_SpellBook.FindFlyoutSlotBySpellID
            local flyoutID
            C_SpellBook.FindFlyoutSlotBySpellID = function(value)
                flyoutID = value
                return "flyout-sentinel"
            end
            local flyoutOK, flyoutResult = pcall(FindFlyoutSlotBySpellID, 202)
            C_SpellBook.FindFlyoutSlotBySpellID = oldFlyout
            if not flyoutOK or flyoutID ~= 202 or flyoutResult ~= "flyout-sentinel" then
                return "flyout-wrapper"
            end

            local oldOverride = C_SpellBook.FindSpellOverrideByID
            local overrideID
            C_SpellBook.FindSpellOverrideByID = function(value)
                overrideID = value
                return "override-sentinel"
            end
            local overrideOK, overrideResult = pcall(FindSpellOverrideByID, 303)
            C_SpellBook.FindSpellOverrideByID = oldOverride
            if not overrideOK or overrideID ~= 303 or overrideResult ~= "override-sentinel" then
                return "override-wrapper"
            end

            if GetBattlegroundInfo ~= nil then return "battleground-global" end
            local first = C_PvP.GetBattlegroundInfo("1.9")
            if type(first) ~= "table" or first.name ~= "Wintergrasp" or first.icon ~= nil
                or first.gameType ~= "wintergrasp" or first.shortDescription ~= "short"
                or first.longDescription ~= "long" or first.mapDescription ~= "map"
                or first.maxPlayers ~= 40 or first.battlegroundID ~= 571
                or first.lfgDungeonID ~= 117 or first.mapID ~= 571
                or first.isHoliday ~= false or first.isRandom ~= false
                or first.canEnter ~= true or first.isTrainingGround ~= false then
                return "battleground-row"
            end
            local second = C_PvP.GetBattlegroundInfo(2.9)
            if type(second) ~= "table" or second.name ~= "Arena" or second.icon ~= 1300 then
                return "battleground-fractional-index"
            end
            if C_PvP.GetBattlegroundInfo(0) ~= nil or C_PvP.GetBattlegroundInfo(999) ~= nil then
                return "battleground-miss"
            end
            if select('#', C_PvP.GetBattlegroundInfo(0)) ~= 1 then
                return "battleground-miss-arity"
            end
            if pcall(C_PvP.GetBattlegroundInfo) or pcall(C_PvP.GetBattlegroundInfo, nil) then
                return "battleground-missing-index"
            end
            for _, invalid in ipairs({ false, {}, "not numeric", -1, 4294967296 }) do
                if pcall(C_PvP.GetBattlegroundInfo, invalid) then
                    return "battleground-invalid-index"
                end
            end

            local soundOK = pcall(PlaySound, 861)
            if not soundOK then
                return "play-sound"
            end

            if strtrim("  hello \t") ~= "hello" or strtrim("--hello--", "-") ~= "hello" then
                return "strtrim"
            end
            return "ok"
            "#,
        )
        .expect("legacy compatibility probe succeeds");

    assert_eq!(result, "ok");
}
