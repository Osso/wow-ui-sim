//! User-approved quest difficulty behavior and original world-map consumer.

#[path = "common/keybinding_panel_fixtures.rs"]
mod keybinding_panel_fixtures;

#[test]
fn user_defaults_batch2_quest_difficulty_boundaries() {
    let env = keybinding_panel_fixtures::setup_env();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.player.level = 80;
        let base = sim.quest_log_entries.entries[0].clone();
        sim.quest_log_entries.entries = [85, 84, 83, 82, 76, 75, 72, 71]
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
    env.exec(r#"
        local expected = { 'Impossible', 'Difficult', 'Difficult', 'Fair', 'Fair', 'Easy', 'Easy', 'Trivial' }
        for i, name in ipairs(expected) do
            assert(C_PlayerInfo.GetContentDifficultyQuestForPlayer(49999 + i) == Enum.RelativeContentDifficulty[name], name)
        end
    "#).expect("quest levels must select each difficulty boundary");
}

#[test]
fn user_defaults_batch2_world_map_quest_consumer() {
    let env = keybinding_panel_fixtures::setup_env();
    env.send_key_press("M", None).expect("open world map");
    env.exec(
        r#"
        local source = debug.getinfo(QuestLogQuests_Update, 'S').source:gsub('\\', '/')
        assert(source:find('Mainline/QuestMapFrame.lua', 1, true))
        assert(type(QuestMapFrame:GetParent():GetMapID()) == 'number')
        QuestLogQuests_Update()
    "#,
    )
    .expect("real quest list consumes quest difficulty");
}
