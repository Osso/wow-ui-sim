//! `C_QuestLog` probe surface backed by `SimState.quest_log_entries`.
//!
//! Migrates 18 entries off the namespace stub tables:
//!
//! - `GetBountySetInfoForMapID(mapID)` — nil (no bounty board data).
//! - `GetInfo(logIndex)` — returns QuestInfo table or nil.
//! - `GetNextWaypoint(questID)` — returns (x, y) or nothing.
//! - `GetQuestDetailsTheme(questID)` — returns theme string or nil.
//! - `GetQuestTagInfo(questID)` — returns QuestTagInfo table or nil.
//! - `GetWorldQuestInfo(questID)` — returns WorldQuestInfo table or nil.
//! - `GetAllCompletedQuestIDs()` — returns array of completed quest IDs.
//! - `GetLogIndexForQuestID(questID)` — returns 1-based index or nil.
//! - `GetMaxNumQuestsCanAccept()` — returns the quest accept cap.
//! - `GetNumQuestLogEntries()` — returns (shownCount, totalCount).
//! - `IsComplete(questID)` — returns bool.
//! - `IsFailed(questID)` — returns bool.
//! - `IsMetaQuest(questID)` — returns bool.
//! - `IsOnMap(questID)` — returns bool.
//! - `IsOnQuest(questID)` — returns bool.
//! - `IsQuestFlaggedCompleted(questID)` — returns bool.
//! - `IsQuestReplayable(questID)` — returns bool.
//! - `IsWorldQuest(questID)` — returns bool.

use super::{ensure_namespace, set_table_array};
use crate::c_api::quest_objectives;
use crate::lua_api::globals::quest_surface::data::{
    QUEST_LOG, QuestLogEntry as SeededQuestLogEntry, WORLD_QUESTS,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::sim_substates::{QuestLogEntry, QuestLogState};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const QUEST_LOG_HEADER_TITLE: &str = "Khaz Algar";

pub(super) fn register_quest_log_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_QuestLog")?;
    for (name, func) in C_QUEST_LOG_METHODS {
        table_set_rust_fn_static(state, ns, name, *func)?;
    }
    quest_objectives::register(state)?;
    let globals = state.global;
    table_set_rust_fn_static(state, globals, "GetQuestPOIs", get_quest_pois)?;
    Ok(())
}

const C_QUEST_LOG_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    ("GetBountySetInfoForMapID", get_bounty_set_info_for_map_id),
    ("GetInfo", get_info),
    ("GetMapForQuestPOIs", get_map_for_quest_pois),
    ("GetMaxNumQuests", get_max_num_quests),
    ("GetMaxNumQuestsCanAccept", get_max_num_quests),
    ("GetNextWaypoint", get_next_waypoint),
    ("GetNextWaypointForMap", get_next_waypoint_for_map),
    ("GetNumQuestObjectives", get_num_quest_objectives),
    ("GetQuestDetailsTheme", get_quest_details_theme),
    ("GetQuestTagInfo", get_quest_tag_info),
    ("GetQuestsOnMap", get_quests_on_map),
    ("GetWorldQuestInfo", get_world_quest_info),
    ("GetAllCompletedQuestIDs", get_all_completed_quest_ids),
    ("GetQuestIDForLogIndex", get_quest_id_for_log_index),
    ("GetLogIndexForQuestID", get_log_index_for_quest_id),
    ("GetMaxNumQuestsCanAccept", get_max_num_quests_can_accept),
    ("GetNumQuestLogEntries", get_num_quest_log_entries),
    ("IsComplete", is_complete),
    ("IsFailed", is_failed),
    ("IsMetaQuest", is_meta_quest),
    ("IsOnMap", is_on_map),
    ("IsOnQuest", is_on_quest),
    ("IsQuestFlaggedCompleted", is_quest_flagged_completed),
    ("IsQuestReplayable", is_quest_replayable),
    ("IsThreatQuest", is_threat_quest),
    ("IsWorldQuest", is_world_quest),
    ("SetMapForQuestPOIs", set_map_for_quest_pois),
];

fn get_bounty_set_info_for_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let _map_id = i32::from_stack(state, 1)?;
    // No bounty board data in the sim.
    state.push(Val::Nil);
    Ok(1)
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let log_index = i32::from_stack(state, 1)?;
    if log_index < 1 {
        return Ok(0);
    }
    let kind = {
        let sim = borrow_state(state)?;
        classify_quest_log_index(sim.quest_log_entries.entries.len(), log_index)
    };
    let Some(kind) = kind else { return Ok(0) };

    let t = create_table(state);
    match kind {
        QuestLogIndexKind::Header => {
            write_header_identity_fields(state, t, log_index);
            write_header_constant_fields(state, t);
        }
        QuestLogIndexKind::Quest(quest_entry_index) => {
            let entry = borrow_state(state)?
                .quest_log_entries
                .entries
                .get(quest_entry_index)
                .cloned();
            let Some(entry) = entry else { return Ok(0) };
            write_quest_identity_fields(state, t, &entry, log_index);
            write_quest_classification_flags(state, t, &entry);
            write_quest_constant_stub_fields(state, t, false);
            let campaign = borrow_state(state)?
                .quest_log_entries
                .campaign_ids
                .get(&entry.quest_id)
                .copied()
                .unwrap_or(0);
            table_set(state, t, "campaignID", Val::Num(f64::from(campaign)));
            let auto_complete = crate::c_api::quest_objectives::ENABLED
                && borrow_state(state)?
                    .quest_log_entries
                    .readiness
                    .auto_complete
                    .contains(&entry.quest_id);
            table_set(state, t, "isAutoComplete", Val::Bool(auto_complete));
        }
    }

    state.push(t);
    Ok(1)
}

#[derive(Clone, Copy)]
enum QuestLogIndexKind {
    Header,
    Quest(usize),
}

fn classify_quest_log_index(entry_count: usize, log_index: i32) -> Option<QuestLogIndexKind> {
    if entry_count == 0 || log_index < 1 {
        return None;
    }
    if log_index == 1 {
        return Some(QuestLogIndexKind::Header);
    }
    let quest_index = (log_index - 2) as usize;
    (quest_index < entry_count).then_some(QuestLogIndexKind::Quest(quest_index))
}

fn write_header_identity_fields(state: &mut LuaState, t: Val, log_index: i32) {
    table_set(state, t, "questID", Val::Num(0.0));
    let title = create_string(state, QUEST_LOG_HEADER_TITLE);
    table_set(state, t, "title", title);
    table_set(state, t, "level", Val::Num(0.0));
    table_set(state, t, "questLogIndex", Val::Num(log_index as f64));
    table_set(state, t, "difficultyLevel", Val::Num(0.0));
    table_set(state, t, "isComplete", Val::Bool(false));
    table_set(state, t, "isFailed", Val::Bool(false));
}

fn write_quest_identity_fields(
    state: &mut LuaState,
    t: Val,
    entry: &QuestLogEntry,
    log_index: i32,
) {
    table_set(state, t, "questID", Val::Num(entry.quest_id as f64));
    let title = create_string(state, &entry.title);
    table_set(state, t, "title", title);
    table_set(state, t, "level", Val::Num(entry.level as f64));
    table_set(state, t, "questLogIndex", Val::Num(log_index as f64));
    table_set(state, t, "difficultyLevel", Val::Num(entry.level as f64));
    table_set(state, t, "isComplete", Val::Bool(entry.is_complete));
    table_set(state, t, "isFailed", Val::Bool(entry.is_failed));
}

fn write_quest_classification_flags(state: &mut LuaState, t: Val, entry: &QuestLogEntry) {
    table_set(state, t, "isMeta", Val::Bool(entry.is_meta));
    table_set(state, t, "isWorldQuest", Val::Bool(entry.is_world_quest));
    table_set(state, t, "isTask", Val::Bool(entry.is_world_quest));
    table_set(state, t, "isOnMap", Val::Bool(entry.map_id.is_some()));
}

/// Fields that the sim always reports with a constant value — headers,
/// bounty/story categories, POI flags, sort/scaling toggles, and the
/// campaign/suggestedGroup identifiers we don't model.
fn write_quest_constant_stub_fields(state: &mut LuaState, t: Val, is_header: bool) {
    table_set(state, t, "isHeader", Val::Bool(is_header));
    table_set(state, t, "isCollapsed", Val::Bool(false));
    table_set(state, t, "isBounty", Val::Bool(false));
    table_set(state, t, "isStory", Val::Bool(false));
    table_set(state, t, "hasLocalPOI", Val::Bool(false));
    table_set(state, t, "isHidden", Val::Bool(false));
    table_set(state, t, "overridesSortOrder", Val::Bool(false));
    table_set(state, t, "startEvent", Val::Bool(false));
    table_set(state, t, "isScaling", Val::Bool(false));
    table_set(state, t, "readyForTranslation", Val::Bool(false));
    table_set(state, t, "suggestedGroup", Val::Num(0.0));
}

fn write_header_constant_fields(state: &mut LuaState, t: Val) {
    table_set(state, t, "isAutoComplete", Val::Bool(false));
    table_set(state, t, "isMeta", Val::Bool(false));
    table_set(state, t, "isWorldQuest", Val::Bool(false));
    table_set(state, t, "isTask", Val::Bool(false));
    table_set(state, t, "isOnMap", Val::Bool(false));
    write_quest_constant_stub_fields(state, t, true);
}

fn get_next_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let waypoint = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|e| e.quest_id == quest_id)
        .and_then(|e| e.waypoint);
    match waypoint {
        Some((x, y)) => {
            state.push(Val::Num(x));
            state.push(Val::Num(y));
            Ok(2)
        }
        None => Ok(0),
    }
}

fn get_map_for_quest_pois(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = borrow_state(state)?.quest_poi_map_id.unwrap_or(0);
    state.push(Val::Num(map_id as f64));
    Ok(1)
}

fn get_next_waypoint_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let map_id = i32::from_stack(state, 2)?;
    let waypoint = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id && entry.map_id == Some(map_id))
        .and_then(|entry| entry.waypoint);
    match waypoint {
        Some((x, y)) => {
            state.push(Val::Num(x));
            state.push(Val::Num(y));
            Ok(2)
        }
        None => Ok(0),
    }
}

fn get_num_quest_objectives(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let count =
        objective_count_for_quest(&borrow_state(state)?.quest_log_entries, quest_id).unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_quest_details_theme(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let theme = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|e| e.quest_id == quest_id)
        .and_then(|e| e.details_theme.clone());
    match theme {
        Some(t) => {
            let s = create_string(state, &t);
            state.push(s);
            Ok(1)
        }
        None => Ok(0),
    }
}

fn get_quest_tag_info(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|e| e.quest_id == quest_id)
        .cloned();
    let Some(entry) = entry else {
        return Ok(0);
    };
    let t = create_table(state);
    let tag_id = entry.tag_id.unwrap_or(0);
    table_set(state, t, "tagID", Val::Num(tag_id as f64));
    let tag_name = if entry.is_world_quest {
        "World Quest"
    } else {
        "Quest"
    };
    let tag_name_val = create_string(state, tag_name);
    table_set(state, t, "tagName", tag_name_val);
    if entry.is_world_quest {
        table_set(state, t, "worldQuestType", Val::Num(2.0));
        table_set(state, t, "quality", Val::Num(0.0));
        table_set(state, t, "displayExpiration", Val::Bool(true));
    } else {
        table_set(state, t, "quality", Val::Num(1.0));
        table_set(state, t, "displayExpiration", Val::Bool(false));
    }
    table_set(state, t, "isElite", Val::Bool(false));
    state.push(t);
    Ok(1)
}

fn get_world_quest_info(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|e| e.quest_id == quest_id && e.is_world_quest)
        .cloned();
    let Some(entry) = entry else { return Ok(0) };

    let t = create_table(state);
    table_set(state, t, "questID", Val::Num(entry.quest_id as f64));
    let map_id = entry.map_id.unwrap_or(0);
    table_set(state, t, "mapID", Val::Num(map_id as f64));
    if let Some((x, y)) = entry.waypoint {
        table_set(state, t, "x", Val::Num(x));
        table_set(state, t, "y", Val::Num(y));
    }
    let count = if quest_objectives::ENABLED {
        quest_objectives::objective_count(&borrow_state(state)?.quest_log_entries, quest_id)
    } else {
        1
    };
    table_set(state, t, "numObjectives", Val::Num(f64::from(count)));
    table_set(state, t, "isElite", Val::Bool(false));
    table_set(state, t, "tradeskillLineIndex", Val::Num(0.0));
    state.push(t);
    Ok(1)
}

fn get_quests_on_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let quests = {
        let sim = borrow_state(state)?;
        let log = &sim.quest_log_entries;
        log.entries
            .iter()
            .filter(|entry| entry.map_id == Some(map_id))
            .filter_map(|entry| quest_poi_map_info_for_entry(log, entry))
            .collect::<Vec<_>>()
    };

    let array = create_table(state);
    for (index, quest) in quests.into_iter().enumerate() {
        let info = create_table(state);
        write_quest_poi_map_info(state, info, quest);
        set_table_array(state, array, index as i64 + 1, info);
    }
    state.push(array);
    Ok(1)
}

fn get_all_completed_quest_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<i32> = borrow_state(state)?
        .quest_log_entries
        .completed_quest_ids
        .iter()
        .copied()
        .collect();
    let array = create_table(state);
    for (i, id) in ids.into_iter().enumerate() {
        set_table_array(state, array, i as i64 + 1, Val::Num(id as f64));
    }
    state.push(array);
    Ok(1)
}

fn get_quest_id_for_log_index(state: &mut LuaState) -> LuaResult<u32> {
    let log_index = i32::from_stack(state, 1)?;
    let kind = {
        let sim = borrow_state(state)?;
        classify_quest_log_index(sim.quest_log_entries.entries.len(), log_index)
    };
    match kind {
        Some(QuestLogIndexKind::Header) => {
            state.push(Val::Num(0.0));
            Ok(1)
        }
        Some(QuestLogIndexKind::Quest(quest_entry_index)) => {
            let quest_id = borrow_state(state)?
                .quest_log_entries
                .entries
                .get(quest_entry_index)
                .map(|entry| entry.quest_id)
                .unwrap_or(0);
            state.push(Val::Num(quest_id as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

fn get_log_index_for_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let idx = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .enumerate()
        .find_map(|(i, e)| {
            if e.quest_id == quest_id {
                Some(i as i32 + 2)
            } else {
                None
            }
        });
    match idx {
        Some(i) => {
            state.push(Val::Num(i as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

fn get_max_num_quests_can_accept(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(25.0));
    Ok(1)
}

fn get_num_quest_log_entries(state: &mut LuaState) -> LuaResult<u32> {
    let quest_count = borrow_state(state)?.quest_log_entries.entries.len() as f64;
    let total_entries = if quest_count > 0.0 {
        quest_count + 1.0
    } else {
        0.0
    };
    state.push(Val::Num(total_entries));
    state.push(Val::Num(quest_count));
    Ok(2)
}

fn quest_bool_field(
    state: &mut LuaState,
    f: fn(&crate::lua_api::sim_substates::QuestLogEntry) -> bool,
) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let val = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|e| e.quest_id == quest_id)
        .map(f)
        .unwrap_or(false);
    state.push(Val::Bool(val));
    Ok(1)
}

fn is_complete(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.is_complete)
}

fn is_failed(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.is_failed)
}

fn is_meta_quest(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.is_meta)
}

fn is_on_map(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.map_id.is_some())
}

fn is_on_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let found = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .any(|e| e.quest_id == quest_id);
    state.push(Val::Bool(found));
    Ok(1)
}

fn is_quest_flagged_completed(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let flagged = is_completed_quest_id(&borrow_state(state)?.quest_log_entries, quest_id);
    state.push(Val::Bool(flagged));
    Ok(1)
}

fn is_completed_quest_id(quest_log: &QuestLogState, quest_id: i32) -> bool {
    quest_log.completed_quest_ids.contains(&quest_id)
}

fn is_quest_replayable(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.is_replayable)
}

fn is_threat_quest(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_world_quest(state: &mut LuaState) -> LuaResult<u32> {
    quest_bool_field(state, |e| e.is_world_quest)
}

fn get_max_num_quests(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(25.0));
    Ok(1)
}

fn get_quest_pois(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(out_table) = stack_val(state, 1) else {
        return Ok(0);
    };

    let pois = {
        let sim = borrow_state(state)?;
        sim.quest_log_entries
            .entries
            .iter()
            .filter_map(|entry| {
                let map_id = entry.map_id?;
                let (x, y) = entry.waypoint?;
                Some((entry.quest_id, map_id, x, y))
            })
            .collect::<Vec<_>>()
    };

    for (index, (quest_id, map_id, x, y)) in pois.into_iter().enumerate() {
        let info = create_table(state);
        table_set(state, info, "questID", Val::Num(quest_id as f64));
        table_set(state, info, "mapID", Val::Num(map_id as f64));
        table_set(state, info, "x", Val::Num(x));
        table_set(state, info, "y", Val::Num(y));
        set_table_array(state, Val::Table(out_table), index as i64 + 1, info);
    }

    Ok(0)
}

fn set_map_for_quest_pois(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.quest_poi_map_id = Some(map_id);
    Ok(0)
}

#[derive(Clone, Copy)]
struct QuestPoiMapInfo {
    quest_id: i32,
    map_id: i32,
    x: f64,
    y: f64,
    num_objectives: i32,
    is_meta: bool,
    in_progress: bool,
}

fn quest_poi_map_info_for_entry(
    log: &QuestLogState,
    entry: &QuestLogEntry,
) -> Option<QuestPoiMapInfo> {
    let map_id = entry.map_id?;
    let (x, y) = entry.waypoint?;
    let num_objectives = objective_count_for_quest(log, entry.quest_id).unwrap_or(0);
    Some(QuestPoiMapInfo {
        quest_id: entry.quest_id,
        map_id,
        x,
        y,
        num_objectives,
        is_meta: entry.is_meta,
        in_progress: !entry.is_complete,
    })
}

fn objective_count_for_quest(log: &QuestLogState, quest_id: i32) -> Option<i32> {
    if quest_objectives::ENABLED {
        return Some(quest_objectives::objective_count(log, quest_id));
    }
    if let Some(quest) = WORLD_QUESTS.iter().find(|quest| quest.quest_id == quest_id) {
        return Some(quest.num_objectives);
    }

    QUEST_LOG.iter().find_map(|entry| match entry {
        SeededQuestLogEntry::Quest {
            quest_id: seeded_quest_id,
            objectives,
            ..
        } if *seeded_quest_id == quest_id => Some(objectives.len() as i32),
        _ => None,
    })
}

fn write_quest_poi_map_info(state: &mut LuaState, info: Val, quest: QuestPoiMapInfo) {
    table_set(state, info, "questID", Val::Num(quest.quest_id as f64));
    table_set(
        state,
        info,
        "numObjectives",
        Val::Num(quest.num_objectives as f64),
    );
    table_set(state, info, "mapID", Val::Num(quest.map_id as f64));
    table_set(state, info, "x", Val::Num(quest.x));
    table_set(state, info, "y", Val::Num(quest.y));
    table_set(state, info, "isQuestStart", Val::Bool(false));
    table_set(state, info, "isDaily", Val::Bool(false));
    table_set(state, info, "isCombatAllyQuest", Val::Bool(false));
    table_set(state, info, "isMeta", Val::Bool(quest.is_meta));
    table_set(state, info, "inProgress", Val::Bool(quest.in_progress));
    table_set(state, info, "isMapIndicatorQuest", Val::Bool(false));
}
