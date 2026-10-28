//! User-approved quest difficulty behavior.

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
