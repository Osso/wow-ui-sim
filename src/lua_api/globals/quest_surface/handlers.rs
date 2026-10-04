//! Quest surface handler functions registered into the Lua environment.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_string_static,
    create_table, frame_ref, table_set,
};
use crate::lua_api::script_helpers::{get_dispatch_script, get_event_listeners};
use crate::lua_bridge::FromStack;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

use super::data::{Objective, QUEST_LOG, QuestLogEntry, WORLD_QUESTS, WorldQuest};
use super::info_fields::{write_quest_entry_fields, write_quest_header_fields};

#[derive(Clone, Copy)]
pub(super) struct StaticQuest {
    pub(super) title: &'static str,
    pub(super) description: &'static str,
    pub(super) objectives: &'static [Objective],
}

pub fn quest_count() -> i32 {
    QUEST_LOG
        .iter()
        .filter(|entry| matches!(entry, QuestLogEntry::Quest { .. }))
        .count() as i32
}

pub fn find_quest_by_id(quest_id: i32) -> Option<(i32, &'static QuestLogEntry)> {
    QUEST_LOG
        .iter()
        .enumerate()
        .find_map(|(index, entry)| match entry {
            QuestLogEntry::Quest { quest_id: id, .. } if *id == quest_id => {
                Some((index as i32 + 1, entry))
            }
            _ => None,
        })
}

pub fn find_world_quest(quest_id: i32) -> Option<&'static WorldQuest> {
    WORLD_QUESTS.iter().find(|q| q.quest_id == quest_id)
}

pub fn quest_exists(quest_id: i32) -> bool {
    find_quest_by_id(quest_id).is_some()
}

pub fn is_world_quest(quest_id: i32) -> bool {
    find_world_quest(quest_id).is_some()
}

pub(super) fn entry_at(log_index: i32) -> Option<&'static QuestLogEntry> {
    QUEST_LOG.get((log_index - 1) as usize)
}

fn watched_quest_id_at_index(index: i32) -> Option<i32> {
    if index <= 0 {
        return None;
    }
    QUEST_LOG
        .iter()
        .filter_map(|entry| match entry {
            QuestLogEntry::Quest { quest_id, .. } => Some(*quest_id),
            _ => None,
        })
        .nth((index - 1) as usize)
}

fn watch_index_for_quest_id(quest_id: i32) -> Option<i32> {
    QUEST_LOG
        .iter()
        .filter_map(|entry| match entry {
            QuestLogEntry::Quest { quest_id, .. } => Some(*quest_id),
            _ => None,
        })
        .position(|watched_quest_id| watched_quest_id == quest_id)
        .map(|index| index as i32 + 1)
}

pub(super) fn objective_at(log_index: i32, objective_index: i32) -> Option<&'static Objective> {
    match entry_at(log_index) {
        Some(QuestLogEntry::Quest { objectives, .. }) if objective_index > 0 => {
            objectives.get((objective_index - 1) as usize)
        }
        _ => None,
    }
}

pub(super) fn selected_quest_id(state: &LuaState) -> LuaResult<i32> {
    Ok(borrow_state(state)?
        .selected_quest_log_id
        .map(|id| id as i32)
        .unwrap_or(0))
}

pub(super) fn selected_log_index(state: &LuaState) -> LuaResult<i32> {
    Ok(find_quest_by_id(selected_quest_id(state)?)
        .map(|(index, _)| index)
        .unwrap_or(0))
}

pub(super) fn set_selected_quest_id(state: &mut LuaState, quest_id: i32) -> LuaResult<()> {
    borrow_state_mut(state)?.selected_quest_log_id = (quest_id > 0).then_some(quest_id as u32);
    Ok(())
}

pub(super) fn selected_criteria_spell(
    state: &LuaState,
) -> LuaResult<Option<(i32, String, String, bool)>> {
    let sim_state = borrow_state(state)?;
    let Some(quest_id) = sim_state.selected_quest_log_id.map(|id| id as i32) else {
        return Ok(None);
    };
    let Some(entry) = sim_state
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
    else {
        return Ok(None);
    };
    let Some(spell_id) = entry.criteria_spell_id else {
        return Ok(None);
    };
    let spell_name = entry.criteria_spell_name.clone().unwrap_or_default();
    let spell_texture = entry.criteria_spell_texture.clone().unwrap_or_default();
    Ok(Some((
        spell_id,
        spell_name,
        spell_texture,
        entry.criteria_spell_finished,
    )))
}

pub(super) fn quest_list_index(state: &mut LuaState) -> LuaResult<usize> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as isize;
    Ok(index.saturating_sub(1) as usize)
}

fn static_quest_by_id(quest_id: i32) -> Option<StaticQuest> {
    QUEST_LOG.iter().find_map(|entry| match entry {
        QuestLogEntry::Quest {
            quest_id: id,
            title,
            description,
            objectives,
        } if *id == quest_id => Some(StaticQuest {
            title,
            description,
            objectives,
        }),
        _ => None,
    })
}

pub(super) fn selected_static_quest(state: &LuaState) -> LuaResult<Option<StaticQuest>> {
    Ok(static_quest_by_id(selected_quest_id(state)?))
}

pub(super) fn join_objective_text(quest: StaticQuest) -> String {
    quest
        .objectives
        .iter()
        .map(|objective| objective.text)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn reward_items_for_quest(
    state: &LuaState,
    quest_id: i32,
) -> LuaResult<Vec<crate::lua_api::state::QuestRewardItem>> {
    let rewards = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .map(|entry| entry.reward_items.clone())
        .unwrap_or_default();
    Ok(rewards)
}

fn quest_has_rewards(state: &LuaState, quest_id: i32) -> LuaResult<bool> {
    let has_rewards = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .is_some_and(|entry| !entry.reward_items.is_empty() || !entry.currency_rewards.is_empty());
    Ok(has_rewards)
}

pub(super) fn push_optional_i32(state: &mut LuaState, value: Option<i32>) {
    match value {
        Some(value) => state.push(Val::Num(value as f64)),
        None => state.push(Val::Nil),
    }
}

pub(super) fn push_optional_bool(state: &mut LuaState, value: Option<bool>) {
    match value {
        Some(value) => state.push(Val::Bool(value)),
        None => state.push(Val::Nil),
    }
}

pub(super) fn fire_event_with_args(state: &mut LuaState, event_name: &'static str, args: &[Val]) {
    for widget_id in get_event_listeners(state, event_name) {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let mut call_args = Vec::with_capacity(2 + args.len());
        call_args.push(frame);
        call_args.push(create_string_static(state, event_name));
        call_args.extend_from_slice(args);
        let _ = call_function_state(state, handler, &call_args);
    }
}

pub(super) fn set_array_value(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    index: i32,
    value: Val,
) {
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}

pub fn get_num_quest_log_entries(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.quest_log_entries.entries.len() as f64;
    state.push(Val::Num(count));
    state.push(Val::Num(count));
    Ok(2)
}

pub fn get_quest_log_title(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let Some(entry) = entry_at(index) else {
        return Ok(0);
    };

    match entry {
        QuestLogEntry::Header { title } => push_legacy_quest_log_title(
            state,
            LegacyQuestLogTitle {
                title,
                level: 0,
                quest_id: None,
                is_header: true,
                ..LegacyQuestLogTitle::default()
            },
        ),
        QuestLogEntry::Quest {
            quest_id, title, ..
        } => {
            let is_complete = is_completed_quest(state, *quest_id)?;
            push_legacy_quest_log_title(
                state,
                LegacyQuestLogTitle {
                    title,
                    level: 80,
                    quest_id: Some(*quest_id),
                    is_complete,
                    is_on_map: true,
                    ..LegacyQuestLogTitle::default()
                },
            )
        }
    }
}

#[derive(Default)]
struct LegacyQuestLogTitle<'a> {
    title: &'a str,
    level: i32,
    quest_id: Option<i32>,
    is_header: bool,
    is_collapsed: bool,
    is_complete: bool,
    is_on_map: bool,
    has_local_poi: bool,
    is_task: bool,
    is_bounty: bool,
    is_story: bool,
}

fn push_legacy_quest_log_title(
    state: &mut LuaState,
    row: LegacyQuestLogTitle<'_>,
) -> LuaResult<u32> {
    let title = create_string(state, row.title);
    state.push(title);
    state.push(Val::Num(row.level as f64));
    state.push(Val::Nil);
    state.push(Val::Bool(row.is_header));
    state.push(Val::Bool(row.is_collapsed));
    push_legacy_completion_flag(state, row.is_complete);
    state.push(Val::Num(0.0));
    push_optional_number(state, row.quest_id);
    state.push(Val::Bool(false));
    push_optional_number(state, row.quest_id);
    state.push(Val::Bool(row.is_on_map));
    state.push(Val::Bool(row.has_local_poi));
    state.push(Val::Bool(row.is_task));
    state.push(Val::Bool(row.is_bounty));
    state.push(Val::Bool(row.is_story));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    Ok(17)
}

fn push_optional_number(state: &mut LuaState, value: Option<i32>) {
    match value {
        Some(value) => state.push(Val::Num(value as f64)),
        None => state.push(Val::Nil),
    }
}

fn push_legacy_completion_flag(state: &mut LuaState, is_complete: bool) {
    if is_complete {
        state.push(Val::Num(1.0));
    } else {
        state.push(Val::Nil);
    }
}

fn is_completed_quest(state: &LuaState, quest_id: i32) -> LuaResult<bool> {
    let completed = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .map(|entry| entry.is_complete || entry.is_flagged_completed)
        .unwrap_or(false);
    Ok(completed)
}

pub fn get_quest_log_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let Some(entry) = entry_at(index) else {
        return Ok(0);
    };
    let info = create_table(state);
    table_set(state, info, "questLogIndex", Val::Num(index as f64));
    match entry {
        QuestLogEntry::Header { title } => write_quest_header_fields(state, info, title),
        QuestLogEntry::Quest {
            quest_id, title, ..
        } => write_quest_entry_fields(state, info, *quest_id, title),
    }
    state.push(info);
    Ok(1)
}

pub fn get_quest_id_for_log_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let quest_id = match entry_at(index) {
        Some(QuestLogEntry::Quest { quest_id, .. }) => *quest_id,
        _ => 0,
    };
    state.push(Val::Num(quest_id as f64));
    Ok(1)
}

pub fn get_log_index_for_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    match find_quest_by_id(quest_id) {
        Some((index, _)) => {
            state.push(Val::Num(index as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

pub fn get_quest_log_index_by_id(state: &mut LuaState) -> LuaResult<u32> {
    get_log_index_for_quest_id(state)
}

pub fn select_quest_log_entry(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let selected_quest_id = match entry_at(index) {
        Some(QuestLogEntry::Quest { quest_id, .. }) => Some(*quest_id as u32),
        _ => None,
    };
    borrow_state_mut(state)?.selected_quest_log_id = selected_quest_id;
    Ok(0)
}

pub fn get_quest_log_selection(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(selected_log_index(state)? as f64));
    Ok(1)
}

pub fn get_quest_log_selected_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(selected_quest_id(state)? as f64));
    Ok(1)
}

pub fn get_abandon_quest_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(quest) = selected_static_quest(state)? else {
        return Ok(0);
    };
    let title = create_string(state, quest.title);
    state.push(title);
    Ok(1)
}

pub fn can_abandon_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(quest_exists(quest_id)));
    Ok(1)
}

pub fn get_abandon_quest_items(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

pub fn get_quest_log_pushable(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

pub fn get_quest_ui_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let map_id = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .and_then(|entry| entry.map_id)
        .unwrap_or(2248);
    state.push(Val::Num(map_id as f64));
    Ok(1)
}

pub fn get_title_for_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let title = match find_quest_by_id(quest_id) {
        Some((_, QuestLogEntry::Quest { title, .. })) => *title,
        _ => "Quest",
    };
    let title_val = create_string(state, title);
    state.push(title_val);
    Ok(1)
}

pub fn get_quest_link(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let Some((_, QuestLogEntry::Quest { title, .. })) = find_quest_by_id(quest_id) else {
        return Ok(0);
    };
    let link = create_string(
        state,
        &format!("|cffffff00|Hquest:{quest_id}|h[{title}]|h|r"),
    );
    state.push(link);
    Ok(1)
}

pub fn get_num_quest_watches(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(quest_count() as f64));
    Ok(1)
}

pub fn get_quest_id_for_quest_watch_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    match watched_quest_id_at_index(index) {
        Some(quest_id) => {
            state.push(Val::Num(quest_id as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

pub fn get_quest_index_for_watch(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let log_index = watched_quest_id_at_index(index)
        .and_then(|quest_id| find_quest_by_id(quest_id).map(|(log_index, _)| log_index))
        .unwrap_or(0);
    state.push(Val::Num(log_index as f64));
    Ok(1)
}

pub fn quest_poi_get_quest_id_by_visible_index(state: &mut LuaState) -> LuaResult<u32> {
    let visible_index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let (quest_id, log_index) = visible_quest_at_index(visible_index).unwrap_or((0, 0));
    state.push(Val::Num(quest_id as f64));
    state.push(Val::Num(log_index as f64));
    Ok(2)
}

fn visible_quest_at_index(visible_index: i32) -> Option<(i32, i32)> {
    if visible_index <= 0 {
        return None;
    }

    if let Some(QuestLogEntry::Quest { quest_id, .. }) = entry_at(visible_index) {
        return Some((*quest_id, visible_index));
    }

    watched_quest_id_at_index(visible_index)
        .and_then(|quest_id| find_quest_by_id(quest_id).map(|(log_index, _)| (quest_id, log_index)))
}

pub fn get_quest_watch_index(state: &mut LuaState) -> LuaResult<u32> {
    let log_index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let watch_index = entry_at(log_index)
        .and_then(|entry| match entry {
            QuestLogEntry::Quest { quest_id, .. } => watch_index_for_quest_id(*quest_id),
            QuestLogEntry::Header { .. } => None,
        })
        .unwrap_or(0);
    state.push(Val::Num(watch_index as f64));
    Ok(1)
}

pub fn get_quest_sort_index(state: &mut LuaState) -> LuaResult<u32> {
    get_quest_watch_index(state)
}

pub fn is_quest_watched(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let watched = matches!(entry_at(index), Some(QuestLogEntry::Quest { .. }));
    state.push(Val::Bool(watched));
    Ok(1)
}

pub fn is_quest_complete(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(is_completed_quest(state, quest_id)?));
    Ok(1)
}

pub fn is_unit_on_quest(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let on_quest = matches!(entry_at(index), Some(QuestLogEntry::Quest { .. }));
    state.push(Val::Bool(on_quest));
    Ok(1)
}

pub fn get_num_world_quest_watches(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

pub fn get_quest_id_for_world_quest_watch_index(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn return_false(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

pub fn return_nil(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn should_show_quest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(quest_has_rewards(state, quest_id)?));
    Ok(1)
}

pub fn is_world_quest_fn(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(is_world_quest(quest_id)));
    Ok(1)
}

pub fn is_quest_task(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(is_world_quest(quest_id)));
    Ok(1)
}

pub fn is_on_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(quest_exists(quest_id)));
    Ok(1)
}

pub fn get_quest_tag_info(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let info = create_table(state);
    if is_world_quest(quest_id) {
        table_set(state, info, "tagID", Val::Num(2.0));
        let tag_name = create_string_static(state, "World Quest");
        table_set(state, info, "tagName", tag_name);
        table_set(state, info, "worldQuestType", Val::Num(2.0));
        table_set(state, info, "quality", Val::Num(0.0));
        table_set(state, info, "isElite", Val::Bool(false));
        table_set(state, info, "displayExpiration", Val::Bool(true));
    } else {
        table_set(state, info, "tagID", Val::Num(0.0));
        let tag_name = create_string_static(state, "Quest");
        table_set(state, info, "tagName", tag_name);
        table_set(state, info, "quality", Val::Num(1.0));
        table_set(state, info, "isElite", Val::Bool(false));
        table_set(state, info, "displayExpiration", Val::Bool(false));
    }
    state.push(info);
    Ok(1)
}

pub fn get_required_money(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

pub fn return_zero(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

pub fn get_quest_reward_currencies(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let currencies = {
        let sim = borrow_state(state)?;
        sim.quest_log_entries
            .entries
            .iter()
            .find(|entry| entry.quest_id == quest_id)
            .map(|entry| entry.currency_rewards.clone())
            .unwrap_or_default()
    };
    let list_val = create_table(state);
    let Val::Table(list_ref) = list_val else {
        return Ok(0);
    };
    for (zero_based_index, currency) in currencies.into_iter().enumerate() {
        let entry_val = build_currency_reward_table(state, &currency);
        set_array_value(state, list_ref, (zero_based_index + 1) as i32, entry_val);
    }
    state.push(list_val);
    Ok(1)
}

fn build_currency_reward_table(
    state: &mut LuaState,
    currency: &crate::lua_api::state::QuestRewardCurrency,
) -> Val {
    let entry = create_table(state);
    let name_val = create_string(state, &currency.name);
    let texture_val = create_string(state, &currency.texture);
    table_set(
        state,
        entry,
        "currencyID",
        Val::Num(currency.currency_id as f64),
    );
    table_set(state, entry, "name", name_val);
    table_set(state, entry, "texture", texture_val);
    table_set(
        state,
        entry,
        "totalRewardAmount",
        Val::Num(currency.total_reward_amount as f64),
    );
    if let Some(base) = currency.base_reward_amount {
        table_set(state, entry, "baseRewardAmount", Val::Num(base as f64));
    }
    entry
}

pub fn get_quest_log_reward_info(state: &mut LuaState) -> LuaResult<u32> {
    let item_index = i32::from_stack(state, 1)?;
    let quest_id = i32::from_stack(state, 2)?;
    if item_index < 1 {
        return Ok(0);
    }
    let item = {
        let sim = borrow_state(state)?;
        sim.quest_log_entries
            .entries
            .iter()
            .find(|entry| entry.quest_id == quest_id)
            .and_then(|entry| entry.reward_items.get((item_index - 1) as usize).cloned())
    };
    let Some(item) = item else {
        return Ok(0);
    };
    let name_val = create_string(state, &item.name);
    let texture_val = create_string(state, &item.texture);
    state.push(name_val);
    state.push(texture_val);
    state.push(Val::Num(item.count as f64));
    state.push(Val::Num(item.quality as f64));
    state.push(Val::Bool(item.is_usable));
    Ok(5)
}

pub fn get_suggested_group_size(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

pub fn get_next_waypoint_text(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn get_time_allowed(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(2)
}

pub fn set_selected_quest(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    set_selected_quest_id(state, quest_id)?;
    Ok(0)
}

pub fn get_selected_quest(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(selected_quest_id(state)? as f64));
    Ok(1)
}

pub fn request_load_quest_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let success = quest_exists(quest_id) || is_world_quest(quest_id);
    fire_event_with_args(
        state,
        "QUEST_DATA_LOAD_RESULT",
        &[Val::Num(quest_id as f64), Val::Bool(success)],
    );
    Ok(0)
}
