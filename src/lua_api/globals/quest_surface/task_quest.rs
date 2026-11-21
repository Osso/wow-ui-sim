//! Quest map and C_TaskQuest handlers.

use crate::c_api::quest_objectives;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

use super::data::{SEEDED_WORLD_QUEST_TIME_LEFT_MINUTES, WORLD_QUESTS};
use super::handlers::{find_world_quest, set_array_value};

/// Seed catalog availability minus this environment's recorded completions.
/// Metadata lookup stays catalog-backed; active/map/time projections use this set.
fn read_available_world_quests(
    state: &LuaState,
) -> LuaResult<Vec<&'static super::data::WorldQuest>> {
    let sim = borrow_state(state)?;
    let completed = &sim.quest_log_entries.completed_quest_ids;
    Ok(WORLD_QUESTS
        .iter()
        .filter(|quest| !completed.contains(&quest.quest_id))
        .collect())
}

pub fn quest_map_frame_get_focused_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let selected_quest_log_id = borrow_state(state)?.selected_quest_log_id;
    match selected_quest_log_id {
        Some(quest_id) => {
            state.push(Val::Num(quest_id as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

/// `QuestMapUpdateAllQuests()` — retail returns the number of POIs that
/// were found on the current world map. The sim mirrors the total quest-log
/// entry count so the seeded quest-map tests stay aligned with the same
/// surface that backs `C_QuestLog.GetNumQuestLogEntries()`.
pub fn quest_map_update_all_quests(state: &mut LuaState) -> LuaResult<u32> {
    let entry_count = borrow_state(state)?.quest_log_entries.entries.len() as f64;
    state.push(Val::Num(entry_count));
    Ok(1)
}

/// `GetQuestLogTimeLeft()` — retail returns seconds remaining on a
/// time-limited quest, or `nil` when the currently selected quest isn't
/// timed. The seeded quest-log has no time-limited entries, so we only
/// return a value when the selected quest resolves to one of the seeded
/// world quests.
pub fn get_quest_log_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let selected = borrow_state(state)?.selected_quest_log_id;
    match selected {
        Some(quest_id)
            if read_available_world_quests(state)?
                .iter()
                .any(|quest| quest.quest_id == quest_id as i32) =>
        {
            state.push(Val::Num(SEEDED_WORLD_QUEST_TIME_LEFT_MINUTES as f64 * 60.0));
            Ok(1)
        }
        _ => Ok(0),
    }
}

pub fn get_quest_log_special_item_info(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn get_reward_skill_points_nil_triplet(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(3)
}

pub(crate) fn build_tasks_table(state: &mut LuaState) -> LuaResult<u32> {
    let available = read_available_world_quests(state)?;
    let mut task_ids: HashSet<i32> = available.iter().map(|quest| quest.quest_id).collect();
    let task_entries: Vec<_> = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .filter(|entry| entry.info_flags.is_task)
        .map(|entry| entry.quest_id)
        .filter(|quest_id| task_ids.insert(*quest_id))
        .collect();
    let task_ids = available
        .into_iter()
        .map(|quest| quest.quest_id)
        .chain(task_entries);
    let result = create_table(state);
    let Val::Table(result_ref) = result else {
        unreachable!("create_table must return a table");
    };
    for (index, quest_id) in (1..).zip(task_ids) {
        set_array_value(state, result_ref, index, Val::Num(f64::from(quest_id)));
    }
    state.push(result);
    Ok(1)
}

pub fn build_task_quest_info(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let result = create_table(state);
    let Val::Table(result_ref) = result else {
        unreachable!("create_table must return a table");
    };

    let available = read_available_world_quests(state)?;
    let matching_world_quests = available.into_iter().filter(|quest| quest.map_id == map_id);
    for (out_index, quest) in (1..).zip(matching_world_quests) {
        let info = create_table(state);
        table_set(state, info, "questID", Val::Num(quest.quest_id as f64));
        table_set(state, info, "x", Val::Num(quest.x));
        table_set(state, info, "y", Val::Num(quest.y));
        table_set(state, info, "mapID", Val::Num(quest.map_id as f64));
        let count = if quest_objectives::ENABLED {
            quest_objectives::objective_count(
                &borrow_state(state)?.quest_log_entries,
                quest.quest_id,
            )
        } else {
            quest.num_objectives
        };
        table_set(state, info, "numObjectives", Val::Num(f64::from(count)));
        table_set(state, info, "isMapIndicatorQuest", Val::Bool(false));
        set_array_value(state, result_ref, out_index, info);
    }

    state.push(result);
    Ok(1)
}

pub fn task_quest_is_active(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let active = read_available_world_quests(state)?
        .iter()
        .any(|quest| quest.quest_id == quest_id);
    state.push(Val::Bool(active));
    Ok(1)
}

pub fn does_map_show_task_quest_objectives(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let shows_objectives = read_available_world_quests(state)?
        .iter()
        .any(|quest| quest.map_id == map_id);
    state.push(Val::Bool(shows_objectives));
    Ok(1)
}

pub fn task_quest_get_quest_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let Some(quest) = find_world_quest(quest_id) else {
        return Ok(0);
    };
    let title = create_string(state, quest.title);
    state.push(title);
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    Ok(4)
}

pub fn task_quest_get_quest_location(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let (x, y) = find_world_quest(quest_id)
        .map(|q| (q.x, q.y))
        .unwrap_or((0.0, 0.0));
    state.push(Val::Num(x));
    state.push(Val::Num(y));
    Ok(2)
}

pub fn task_quest_time_left_minutes(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    if read_available_world_quests(state)?
        .iter()
        .any(|quest| quest.quest_id == quest_id)
    {
        state.push(Val::Num(SEEDED_WORLD_QUEST_TIME_LEFT_MINUTES as f64));
        return Ok(1);
    }
    Ok(0)
}

pub fn task_quest_time_left_seconds(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    if read_available_world_quests(state)?
        .iter()
        .any(|quest| quest.quest_id == quest_id)
    {
        state.push(Val::Num((SEEDED_WORLD_QUEST_TIME_LEFT_MINUTES * 60) as f64));
        return Ok(1);
    }
    Ok(0)
}

pub fn resolve_quest_ui_widget_set_by_type(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<i32>::from_stack(state, 1)?.unwrap_or(0);
    let widget_type = Option::<i32>::from_stack(state, 2)?.unwrap_or(0);
    let set_id = borrow_state(state)?
        .ui_widget_inputs
        .quest_sets
        .get(&(quest_id, widget_type))
        .copied();
    match set_id {
        Some(set_id) => {
            state.push(Val::Num(set_id));
            Ok(1)
        }
        None => Ok(0),
    }
}
