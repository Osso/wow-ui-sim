//! User-approved quest difficulty behavior and original world-map consumer.

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn user_defaults_batch2_quest_difficulty_untuned_bands() {
    let env = keybinding_panel_fixtures::setup_env();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.level = 80;
        let base = sim.quest_log_entries.entries[0].clone();
        sim.quest_log_entries.entries = [85, 78, 74]
            .into_iter()
            .enumerate()
            .map(|(index, level)| {
                let mut row = base.clone();
                row.quest_id = 50000 + index as i32;
                row.level = level;
                row
            })
            .collect();
    }
    // WM89 §4: with no ContentTuning record this build supplies comparison level 0, so the
    // difference is minus the player level and the stored quest level is unused. The consumer
    // is the original DifficultyUtil mapping that QuestMapFrame.lua:1751 applies at runtime.
    env.exec(
        r#"
        for i = 1, 3 do
            local difficulty = C_PlayerInfo.GetContentDifficultyQuestForPlayer(49999 + i)
            assert(difficulty == Enum.RelativeContentDifficulty.Trivial, "stored level is unused")
            assert(GetDifficultyColor(difficulty) == QuestDifficultyColors.trivial)
        end
        "#,
    )
    .expect("untuned quest comparison level ignores the stored level");
    for (player_level, expected, color) in [
        (1, "Fair", "difficult"),
        (2, "Fair", "difficult"),
        (3, "Easy", "standard"),
        (4, "Easy", "standard"),
        (5, "Trivial", "trivial"),
    ] {
        env.state().borrow_mut().player.level = player_level;
        env.exec(&format!(
            r#"
            local difficulty = C_PlayerInfo.GetContentDifficultyQuestForPlayer(50000)
            assert(difficulty == Enum.RelativeContentDifficulty.{expected}, "player {player_level}")
            assert(GetDifficultyColor(difficulty) == QuestDifficultyColors.{color}, "player {player_level}")
            "#
        ))
        .expect("player level selects the fixed native band");
    }
}

#[test]
fn user_defaults_batch2_quest_watch_membership() {
    let env = keybinding_panel_fixtures::setup_env();
    let quest_id = env.state().borrow().quest_log_entries.entries[0].quest_id;
    env.exec(&format!(
        r#"
        local id = {quest_id}
        assert(C_QuestLog.GetQuestWatchType(id) == Enum.QuestWatchType.Manual)
        local count = C_QuestLog.GetNumQuestWatches()
        assert(C_QuestLog.RemoveQuestWatch(id))
        assert(C_QuestLog.GetQuestWatchType(id) == nil)
        assert(C_QuestLog.GetNumQuestWatches() == count - 1)
        assert(C_QuestLog.AddQuestWatch(id))
        assert(C_QuestLog.GetQuestWatchType(id) == Enum.QuestWatchType.Manual)
        assert(C_QuestLog.GetNumQuestWatches() == count)
        assert(not C_QuestLog.IsAccountQuest(id))
        assert(not C_QuestLog.IsAccountQuest(0))
        assert(not C_QuestLog.IsAccountQuest(2147483647))
    "#
    ))
    .expect("quest watch membership must follow changes");
    env.state().borrow_mut().quest_log_entries.entries[0].is_account_quest = Some(true);
    env.exec(&format!("assert(C_QuestLog.IsAccountQuest({quest_id}))"))
        .expect("supplied account quest flag takes precedence");
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_account_quest_argument_conversion_matches_native() {
    let env = keybinding_panel_fixtures::setup_env();
    let quest_id = env.state().borrow().quest_log_entries.entries[0].quest_id;
    env.state().borrow_mut().quest_log_entries.entries[0].is_account_quest = Some(true);
    env.exec(&format!(
        r#"
        assert(select('#', C_QuestLog.IsAccountQuest({quest_id})) == 1)
        assert(C_QuestLog.IsAccountQuest({quest_id}))
        assert(C_QuestLog.IsAccountQuest(tostring({quest_id})))
        assert(C_QuestLog.IsAccountQuest({quest_id} + 0.75))
        assert(C_QuestLog.IsAccountQuest(tostring({quest_id}) .. ".75"))
        assert(not C_QuestLog.IsAccountQuest(-0.75))
        assert(not C_QuestLog.IsAccountQuest(2147483647))
        assert(not pcall(C_QuestLog.IsAccountQuest))
        assert(not pcall(C_QuestLog.IsAccountQuest, nil))
        assert(not pcall(C_QuestLog.IsAccountQuest, "not a number"))
        assert(not pcall(C_QuestLog.IsAccountQuest, 2147483648))
        assert(not pcall(C_QuestLog.IsAccountQuest, 0 / 0))
        "#
    ))
    .expect("Forever account quest conversion should match R27 native behavior");
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn quest_faction_group_uses_native_argument_conversion() {
    let env = keybinding_panel_fixtures::setup_env();
    let quest_ids = {
        let state = env.state();
        let sim = state.borrow();
        let entries = &sim.quest_log_entries.entries;
        [
            entries[0].quest_id,
            entries[1].quest_id,
            entries[2].quest_id,
            entries[3].quest_id,
        ]
    };
    let quest_id = quest_ids[0];
    env.exec(&format!(
        r##"
        local query = GetQuestFactionGroup
        local questID = {quest_id}
        assert(type(query) == "function", "Game global is missing")
        assert(select("#", query(questID)) == 0, "unclassified record should return no values")
        assert(select("#", query(tostring(questID))) == 0, "numeric string should parse")
        assert(select("#", query(questID + 0.75)) == 0, "fraction should truncate")
        assert(select("#", query(0)) == 0, "zero ID should return no values")
        assert(select("#", query(-1)) == 0, "negative ID should return no values")
        assert(select("#", query(-0.75)) == 0, "negative fraction should truncate to zero")

        local usage = "Usage: GetQuestFactionGroup(questID"
        local function expect_usage_error(...)
            local ok, message = pcall(query, ...)
            assert(not ok, "expected a usage error")
            assert(message == usage, "unexpected usage error: " .. tostring(message))
        end
        expect_usage_error()
        expect_usage_error(nil)
        expect_usage_error(false)
        expect_usage_error("not a number")

        local function expect_overflow(value, formatted)
            local ok, message = pcall(query, value)
            assert(not ok, "expected overflow for " .. tostring(value))
            local expected = "integer overflow attempting to store " .. formatted
            assert(message == expected, "expected " .. expected .. ", got " .. tostring(message))
        end
        local function expect_nonfinite_overflow(value)
            local ok, message = pcall(query, value)
            assert(not ok, "expected overflow for " .. tostring(value))
            assert(message:find("integer overflow attempting to store ", 1, true) == 1)
        end
        expect_overflow(2147483648, "2147483648.000000")
        expect_overflow(2147483647.75, "2147483647.750000")
        expect_nonfinite_overflow(0 / 0)
        expect_nonfinite_overflow(math.huge)
        expect_nonfinite_overflow(-math.huge)
        "##
    ))
    .expect("GetQuestFactionGroup should match native conversion and errors");

    {
        let mut sim = env.state().borrow_mut();
        for (entry, faction_mask) in
            sim.quest_log_entries
                .entries
                .iter_mut()
                .zip([Some(1), Some(2), Some(3), Some(0)])
        {
            entry.faction_mask = faction_mask;
        }
    }
    env.exec(&format!(
        r##"
        local query = GetQuestFactionGroup
        local allianceID = {alliance_id}
        local hordeID = {horde_id}
        local multipleFactionsID = {multiple_factions_id}
        local unrestrictedID = {unrestricted_id}
        assert(select("#", query(allianceID)) == 1 and query(allianceID) == 1)
        assert(query(tostring(allianceID)) == 1)
        assert(query(allianceID + 0.75) == 1)
        assert(select("#", query(hordeID)) == 1 and query(hordeID) == 2)
        assert(select("#", query(multipleFactionsID)) == 0)
        assert(select("#", query(unrestrictedID)) == 0)
        assert(select("#", query(allianceID + 1000000)) == 0)
        "##,
        alliance_id = quest_ids[0],
        horde_id = quest_ids[1],
        multiple_factions_id = quest_ids[2],
        unrestricted_id = quest_ids[3],
    ))
    .expect("only single-faction quest records should return faction groups");

    // Glue absence follows the no-Glue-installer inference; Game presence is native-verified.
    env.set_screen_mode(wow_ui_sim::screen::ScreenKind::CharacterSelect);
    let hidden_in_glue: bool = env
        .eval("return GetQuestFactionGroup == nil")
        .expect("Glue global visibility should be queryable");
    assert!(hidden_in_glue);

    env.set_screen_mode(wow_ui_sim::screen::ScreenKind::Game);
    let restored_in_game: bool = env
        .eval("return type(GetQuestFactionGroup) == 'function'")
        .expect("Game global visibility should be queryable");
    assert!(restored_in_game);
}

#[cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
#[test]
fn user_defaults_batch2_world_map_quest_consumer() {
    let env = keybinding_panel_fixtures::setup_env();
    let quest_id = env.state().borrow().quest_log_entries.entries[0].quest_id;
    {
        let mut sim = env.state().borrow_mut();
        let entry = &mut sim.quest_log_entries.entries[0];
        entry.is_account_quest = Some(true);
        entry.faction_mask = Some(1);
    }
    env.send_key_press("M", None).expect("open world map");
    env.exec(&format!(
        r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        local accountQuestText = QuestUtil.GetAccountQuestText({quest_id})
        assert(type(accountQuestText) == 'string')
        assert(accountQuestText:find(FACTION_ALLIANCE, 1, true))
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(type(QuestMapFrame:GetParent():GetMapID()) == 'number')
        QuestLogQuests_Update()
    "#,
    ))
    .expect("original account quest helper and quest-list consumer should run");
}
