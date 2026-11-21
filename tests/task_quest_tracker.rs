#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

const TRACKER_PROBE: &str = r#"
local taskIDs = {}
for _, questID in ipairs(GetTasksTable()) do
  taskIDs[questID] = true
end
if not taskIDs[80000] then return "supplied_task_missing" end
if not taskIDs[90002] then return "second_task_missing_from_table" end
if not QuestUtils_IsQuestWorldQuest(90002) then return "second_task_not_world" end
local localArea, localMap, count = GetTaskInfo(90002)
if not localArea or not localMap then return "local_task_map_flags" end
if count ~= 1 then return "second_task_wrong_count" end
local distantArea, distantMap = GetTaskInfo(80000)
if distantArea or distantMap then return "distant_task_map_flags" end
WorldQuestObjectiveTracker:Update(2000, false)
local shown = {}
for _, questID in ipairs(WorldQuestObjectiveTracker.currentContentList) do
  shown[questID] = true
end
local worldBlocks = WorldQuestObjectiveTracker.usedBlocks[WorldQuestObjectiveTracker.blockTemplate]
if not worldBlocks[90001] then return "first_task_block_missing" end
if not worldBlocks[90002] then return "second_task_block_missing" end
if not shown[90002] then return "second_task_not_in_content_list" end
if shown[80000] then return "non_world_task_shown" end
BonusObjectiveTracker:Update(2000, false)
local bonusBlocks = BonusObjectiveTracker.usedBlocks[BonusObjectiveTracker.blockTemplate] or {}
if bonusBlocks[80000] then return "distant_bonus_task_shown" end
return "ok"
"#;

#[test]
fn objective_trackers_render_local_tasks_and_skip_distant_task() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker"],
        &[],
        |env: &WowLuaEnv, loaded| assert_tracker_blocks(env, loaded),
    );
}

fn assert_tracker_blocks(env: &WowLuaEnv, loaded: &[String]) {
    assert!(
        loaded
            .iter()
            .any(|addon| addon == "Blizzard_ObjectiveTracker")
    );
    supply_task_rows(env);
    let result: String = env
        .eval(TRACKER_PROBE)
        .expect("world-quest tracker layout should run");
    assert_eq!(result, "ok", "tracker result: {result}");
}

fn supply_task_rows(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.quest_log_entries.entries[0].info_flags.is_task = true;
    state.quest_log_entries.entries[0].map_id = Some(2024);
    state.quest_watch_types.remove(&80000);
    supply_second_world_quest(&mut state);
    for (quest_id, text) in [(90001, "Geodes collected"), (90002, "Rift sealed")] {
        let progress = state
            .quest_log_entries
            .objective_progress
            .get_mut(&quest_id)
            .expect("seeded world quest progress exists");
        progress.declared_objective_count = None;
        progress
            .objectives
            .push(wow_ui_sim::c_api::quest_objectives::QuestObjective {
                text: text.to_string(),
                objective_type: "item".to_string(),
                finished: false,
                num_fulfilled: 1,
                num_required: 3,
                criteria_id: None,
            });
    }
    drop(state);
    env.exec("C_Map.SetMapForQuestLog(2025)")
        .expect("set the player's current map");
}

fn supply_second_world_quest(state: &mut wow_ui_sim::lua_api::SimState) {
    let mut entry = state
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == 90001)
        .expect("first seeded world quest entry exists")
        .clone();
    entry.quest_id = 90002;
    entry.title = "Temporal Rift Collapse".to_string();
    entry.is_world_quest = true;
    entry.info_flags.is_task = true;
    state.quest_log_entries.entries.push(entry);
}
