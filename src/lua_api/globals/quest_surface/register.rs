//! Method-table constants and namespace registration for the quest surface.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::SurfaceFn;
use super::handlers::*;
use super::legacy_globals::*;
use super::task_quest::*;

pub const QUEST_LOG_METHODS: &[(&str, SurfaceFn)] = &[
    ("GetNumQuestLogEntries", get_num_quest_log_entries),
    ("GetInfo", get_quest_log_info),
    ("GetQuestIDForLogIndex", get_quest_id_for_log_index),
    ("GetLogIndexForQuestID", get_log_index_for_quest_id),
    ("GetTitleForQuestID", get_title_for_quest_id),
    ("GetNumQuestWatches", get_num_quest_watches),
    (
        "GetQuestIDForQuestWatchIndex",
        get_quest_id_for_quest_watch_index,
    ),
    ("GetNumWorldQuestWatches", get_num_world_quest_watches),
    (
        "GetQuestIDForWorldQuestWatchIndex",
        get_quest_id_for_world_quest_watch_index,
    ),
    ("AddQuestWatch", noop),
    ("RemoveQuestWatch", noop),
    ("SortQuestWatches", noop),
    ("IsQuestFlaggedCompleted", return_false),
    ("IsComplete", return_false),
    ("ReadyForTurnIn", return_false),
    ("IsFailed", return_false),
    ("IsQuestDisabledForSession", return_false),
    ("IsPushableQuest", return_false),
    ("IsRepeatableQuest", return_false),
    ("IsImportantQuest", return_false),
    ("IsMetaQuest", return_false),
    ("IsOnMap", return_false),
    ("IsOnQuest", is_on_quest),
    ("IsWorldQuest", is_world_quest_fn),
    ("IsQuestTask", is_quest_task),
    ("IsQuestBounty", return_false),
    ("GetQuestRewardCurrencies", get_quest_reward_currencies),
    ("GetQuestTagInfo", get_quest_tag_info),
    ("GetRequiredMoney", get_required_money),
    ("GetSuggestedGroupSize", get_suggested_group_size),
    ("ShouldShowQuestRewards", should_show_quest_rewards),
    ("QuestHasWarModeBonus", return_false),
    ("QuestCanHaveWarModeBonus", return_false),
    ("QuestHasQuestSessionBonus", return_false),
    ("GetNextWaypointText", get_next_waypoint_text),
    ("GetTimeAllowed", get_time_allowed),
    ("GetQuestDetailsTheme", return_nil),
    ("RequestLoadQuestByID", request_load_quest_by_id),
    ("SetSelectedQuest", set_selected_quest),
    ("GetSelectedQuest", get_selected_quest),
];

pub const TASK_QUEST_METHODS: &[(&str, SurfaceFn)] = &[
    ("IsActive", task_quest_is_active),
    (
        "DoesMapShowTaskQuestObjectives",
        does_map_show_task_quest_objectives,
    ),
    ("GetQuestsOnMap", build_task_quest_info),
    #[cfg(not(feature = "retail-12-0-0"))]
    ("GetQuestsForPlayerByMapID", build_task_quest_info),
    (
        "GetQuestUIWidgetSetByType",
        resolve_quest_ui_widget_set_by_type,
    ),
    ("GetQuestInfoByQuestID", task_quest_get_quest_info_by_id),
    ("GetQuestLocation", task_quest_get_quest_location),
    ("GetQuestTimeLeftMinutes", task_quest_time_left_minutes),
    ("GetQuestTimeLeftSeconds", task_quest_time_left_seconds),
];

pub const GLOBAL_QUEST_FUNCTIONS: &[(&str, SurfaceFn)] = &[
    ("AddQuestWatch", noop),
    ("CollapseQuestHeader", noop),
    ("ExpandQuestHeader", noop),
    ("GetNumQuestLeaderBoards", get_num_quest_leaderboards),
    ("CanAbandonQuest", can_abandon_quest),
    ("GetAbandonQuestItems", get_abandon_quest_items),
    ("GetAbandonQuestName", get_abandon_quest_name),
    ("GetQuestLogIndexByID", get_quest_log_index_by_id),
    ("GetQuestLogPushable", get_quest_log_pushable),
    ("GetQuestLogRequiredMoney", get_required_money),
    ("GetQuestLogSelection", get_quest_log_selection),
    ("GetQuestLogSelectedID", get_quest_log_selected_id),
    ("GetQuestLogTitle", get_quest_log_title),
    ("GetQuestUiMapID", get_quest_ui_map_id),
    ("GetNumQuestLogEntries", get_num_quest_log_entries),
    ("GetQuestLink", get_quest_link),
    ("GetQuestLogLeaderBoard", get_quest_log_leaderboard),
    ("GetQuestLogQuestText", get_quest_log_quest_text),
    ("GetQuestLogTimeLeft", get_quest_log_time_left),
    ("GetQuestIndexForWatch", get_quest_index_for_watch),
    ("GetQuestSortIndex", get_quest_sort_index),
    ("GetQuestWatchIndex", get_quest_watch_index),
    ("GetNumQuestWatches", get_num_quest_watches),
    ("GetNumQuestLogRewards", return_zero),
    ("GetNumQuestLogChoices", return_zero),
    ("GetQuestLogRewardInfo", get_quest_log_reward_info),
    ("GetQuestLogRewardMoney", return_zero),
    (
        "GetQuestLogRewardSkillPoints",
        get_reward_skill_points_nil_triplet,
    ),
    ("GetQuestLogRewardXP", return_zero),
    ("GetQuestLogRewardArtifactXP", return_zero),
    ("GetQuestLogRewardHonor", return_zero),
    ("GetQuestLogRewardTitle", return_nil),
    ("QuestHasPOIInfo", return_false),
    ("GetNumQuestRewards", get_num_quest_rewards),
    ("GetNumQuestChoices", return_zero),
    ("GetQuestItemInfo", get_quest_item_info),
    ("GetQuestItemInfoLootType", get_quest_item_info_loot_type),
    ("GetGreetingText", get_greeting_text),
    ("GetNumActiveQuests", get_num_active_quests),
    ("GetNumAvailableQuests", get_num_available_quests),
    ("GetActiveTitle", get_active_title),
    ("GetActiveQuestID", get_active_quest_id),
    ("IsActiveQuestTrivial", is_active_quest_trivial),
    ("GetAvailableTitle", get_available_title),
    ("GetAvailableQuestInfo", get_available_quest_info),
    ("SelectActiveQuest", select_active_quest),
    ("SelectAvailableQuest", select_available_quest),
    ("GetQuestID", get_quest_id),
    ("GetTitleText", get_title_text),
    ("GetQuestText", get_quest_text),
    ("GetObjectiveText", get_objective_text),
    ("GetRewardText", get_reward_text),
    ("GetRewardMoney", return_zero),
    ("GetRewardSkillPoints", get_reward_skill_points_nil_triplet),
    ("GetRewardXP", return_zero),
    ("GetRewardArtifactXP", return_zero),
    ("GetRewardHonor", return_zero),
    ("GetRewardTitle", return_nil),
    ("GetSuggestedGroupSize", get_suggested_group_size),
    ("GetQuestPOIBlobCount", get_quest_poi_blob_count),
    ("HaveQuestData", have_quest_data),
    ("HaveQuestRewardData", have_quest_data),
    ("IsCurrentQuestFailed", is_current_quest_failed),
    ("IsQuestComplete", is_quest_complete),
    ("IsQuestWatched", is_quest_watched),
    ("IsQuestSequenced", is_quest_sequenced),
    ("IsUnitOnQuest", is_unit_on_quest),
    (
        "QuestPOIGetQuestIDByVisibleIndex",
        quest_poi_get_quest_id_by_visible_index,
    ),
    ("RemoveQuestWatch", noop),
    ("SelectQuestLogEntry", select_quest_log_entry),
    ("GetQuestLogCompletionText", get_quest_log_completion_text),
    ("GetQuestLogCriteriaSpell", get_quest_log_criteria_spell),
    ("GetQuestProgressBarPercent", get_quest_progress_bar_percent),
    ("GetCriteriaSpell", get_criteria_spell),
    ("QuestMapUpdateAllQuests", quest_map_update_all_quests),
    (
        "QuestMapFrame_GetFocusedQuestID",
        quest_map_frame_get_focused_quest_id,
    ),
    (
        "GetQuestLogSpecialItemInfo",
        get_quest_log_special_item_info,
    ),
    ("SortQuestWatches", noop),
    ("SortQuestSortTypes", noop),
    ("SortQuests", noop),
];

pub fn register_quest_info_handlers(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = super::ensure_global_table(state, "C_QuestLog");
    for (name, func) in QUEST_LOG_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *func)?;
    }
    Ok(())
}

pub fn register_task_quest_handlers(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = super::ensure_global_table(state, "C_TaskQuest");
    for (name, func) in TASK_QUEST_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *func)?;
    }
    Ok(())
}

pub fn register_quest_classification_handler(state: &mut LuaState) -> LuaResult<()> {
    fn get_quest_classification(state: &mut LuaState) -> LuaResult<u32> {
        use crate::lua_bridge::FromStack;
        let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
        let classification = if super::is_world_quest(quest_id) {
            10.0
        } else {
            7.0
        };
        state.push(Val::Num(classification));
        Ok(1)
    }

    let table_ref = super::ensure_global_table(state, "C_QuestInfoSystem");
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetQuestClassification",
        get_quest_classification,
    )?;
    Ok(())
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    for (name, func) in GLOBAL_QUEST_FUNCTIONS {
        LuaApiMut::register_function(lua, name, *func)?;
    }
    let state = lua.state_mut();
    register_quest_info_handlers(state)?;
    register_task_quest_handlers(state)?;
    register_quest_classification_handler(state)?;
    Ok(())
}
