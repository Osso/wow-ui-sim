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
    env.exec(&format!(
        r#"
        assert(select('#', C_QuestLog.IsAccountQuest({quest_id})) == 1)
        assert(C_QuestLog.IsAccountQuest({quest_id}))
        assert(C_QuestLog.IsAccountQuest(tostring({quest_id})))
        assert(C_QuestLog.IsAccountQuest({quest_id} + 0.75))
        assert(C_QuestLog.IsAccountQuest(tostring({quest_id}) .. ".75"))
        assert(not C_QuestLog.IsAccountQuest(-0.75))
        assert(not pcall(C_QuestLog.IsAccountQuest))
        assert(not pcall(C_QuestLog.IsAccountQuest, nil))
        assert(not pcall(C_QuestLog.IsAccountQuest, "not a number"))
        assert(not pcall(C_QuestLog.IsAccountQuest, 2147483648))
        assert(not pcall(C_QuestLog.IsAccountQuest, 0 / 0))
        "#
    ))
    .expect("native account quest conversion and supplied flag should be honored");
}

#[test]
fn user_defaults_batch2_world_map_quest_consumer() {
    let env = keybinding_panel_fixtures::setup_env();
    let quest_id = env.state().borrow().quest_log_entries.entries[0].quest_id;
    env.state().borrow_mut().quest_log_entries.entries[0].is_account_quest = Some(true);
    env.send_key_press("M", None).expect("open world map");
    env.exec(&format!(
        r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        assert(type(QuestUtil.GetAccountQuestText({quest_id})) == 'string')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(type(QuestMapFrame:GetParent():GetMapID()) == 'number')
        QuestLogQuests_Update()
    "#,
    ))
    .expect("original account quest helper and quest-list consumer should run");
}
