#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn world_quest_tracker_renders_world_tasks_and_skips_non_world_task() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker"],
        &[],
        |env: &WowLuaEnv, loaded| assert_tracker_blocks(env, loaded),
    );
}

fn assert_tracker_blocks(env: &WowLuaEnv, loaded: &[String]) {
    assert!(loaded.iter().any(|addon| addon == "Blizzard_ObjectiveTracker"));
    supply_task_rows(env);
    let result: String = env
                .eval(
                    r#"
                    local taskIDs = {}
                    for _, questID in ipairs(GetTasksTable()) do
                      taskIDs[questID] = true
                    end
                    if not taskIDs[80000] then return "supplied_task_missing" end
                    WorldQuestObjectiveTracker:Update(2000, false)
                    local shown = {}
                    for _, questID in ipairs(WorldQuestObjectiveTracker.currentContentList) do
                      shown[questID] = true
                    end
                    local blocks = WorldQuestObjectiveTracker.usedBlocks[WorldQuestObjectiveTracker.blockTemplate]
                    if not blocks[90001] then return "first_task_block_missing" end
                    if not blocks[90002] then return "second_task_block_missing" end
                    if shown[80000] then return "non_world_task_shown" end
                    return "ok"
                    "#,
                )
                .expect("world-quest tracker layout should run");
    assert_eq!(result, "ok", "tracker result: {result}");
}

fn supply_task_rows(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state
        .quest_log_entries
        .completed_quest_ids
        .extend([90101, 90102, 90103, 90104, 90003, 90004, 90005, 90006, 90007, 90008]);
    state.quest_log_entries.entries[0].info_flags.is_task = true;
    for (quest_id, text) in [(90001, "Geodes collected"), (90002, "Rift sealed")] {
        let progress = state
            .quest_log_entries
            .objective_progress
            .get_mut(&quest_id)
            .expect("seeded world quest progress exists");
        progress.declared_objective_count = None;
        progress.objectives.push(wow_ui_sim::c_api::quest_objectives::QuestObjective {
            text: text.to_string(),
            objective_type: "item".to_string(),
            finished: false,
            num_fulfilled: 1,
            num_required: 3,
            criteria_id: None,
        });
    }
}
