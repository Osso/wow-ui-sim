#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn world_quest_tracker_renders_world_tasks_and_skips_non_world_task() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker"],
        &[],
        |env: &WowLuaEnv, loaded| {
            assert!(loaded.iter().any(|addon| addon == "Blizzard_ObjectiveTracker"));
            env.state()
                .borrow_mut()
                .quest_log_entries
                .entries[0]
                .info_flags
                .is_task = true;

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
        },
    );
}
