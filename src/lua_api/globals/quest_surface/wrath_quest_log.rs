//! Wrath legacy reads over supplied flat rich rows, without synthetic headers.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::{QuestLogEntry, SimState};
use crate::lua_bridge::FromStack;
use rilua::{LuaResult, Val, vm::state::LuaState};

pub(super) const LEGACY_READS: &[(&str, super::SurfaceFn)] = &[
    (
        "GetNumQuestLeaderBoards",
        super::wrath_objectives::get_num_quest_leaderboards,
    ),
    (
        "GetQuestLogLeaderBoard",
        super::wrath_objectives::get_quest_log_leaderboard,
    ),
    (
        "GetQuestLogQuestText",
        super::wrath_objectives::get_quest_log_quest_text,
    ),
    ("GetQuestLogGroupNum", get_quest_log_group_num),
    ("GetQuestGreenRange", get_quest_green_range),
    ("GetDailyQuestsCompleted", get_daily_quests_completed),
    ("GetMaxDailyQuests", get_max_daily_quests),
    ("GetNumQuestLogRewards", get_num_quest_log_rewards),
    ("GetQuestLogRewardInfo", get_quest_log_reward_info),
    ("GetQuestLogRewardSpell", get_quest_log_reward_spell),
    (
        "GetQuestLogRewardArenaPoints",
        get_quest_log_reward_arena_points,
    ),
    ("GetQuestLogRewardTalents", get_quest_log_reward_talents),
    (
        "ProcessQuestLogRewardFactions",
        process_quest_log_reward_factions,
    ),
    (
        "GetNumQuestLogRewardFactions",
        get_num_quest_log_reward_factions,
    ),
    (
        "GetQuestLogRewardFactionInfo",
        get_quest_log_reward_faction_info,
    ),
];

fn supplied_index(state: &LuaState) -> LuaResult<usize> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as usize;
    Ok(index.wrapping_sub(1))
}

pub(super) fn get_abandon_quest_name(state: &mut LuaState) -> LuaResult<u32> {
    let title = {
        let sim = borrow_state(state)?;
        sim.abandon_quest_id.and_then(|id| {
            sim.quest_log_entries
                .entries
                .iter()
                .find(|row| row.quest_id as u32 == id)
                .map(|row| row.title.clone())
        })
    };
    let Some(title) = title else {
        return Ok(0);
    };
    let title = create_string(state, &title);
    state.push(title);
    Ok(1)
}

pub(super) fn get_quest_log_title(state: &mut LuaState) -> LuaResult<u32> {
    let index = supplied_index(state)?;
    let row = borrow_state(state)?
        .quest_log_entries
        .entries
        .get(index)
        .cloned();
    let Some(row) = row else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let title = create_string(state, &row.title);
    let completion = if row.is_failed {
        Val::Num(-1.0)
    } else if row.is_complete {
        Val::Num(1.0)
    } else {
        Val::Nil
    };
    // WotLK: title, level, tag, group, header, collapsed, complete, daily, ID, displayID.
    for value in [
        title,
        Val::Num(row.level as f64),
        Val::Nil,
        Val::Nil,
        Val::Nil,
        Val::Nil,
        completion,
        Val::Nil,
        Val::Num(row.quest_id as f64),
        Val::Nil,
    ] {
        state.push(value);
    }
    Ok(10)
}

pub(super) fn select_quest_log_entry(state: &mut LuaState) -> LuaResult<u32> {
    let index = supplied_index(state)?;
    let quest_id = borrow_state(state)?
        .quest_log_entries
        .entries
        .get(index)
        .map(|row| row.quest_id)
        .unwrap_or(0);
    super::handlers::set_selected_quest_id(state, quest_id)?;
    Ok(0)
}

pub(super) fn get_quest_log_group_num(state: &mut LuaState) -> LuaResult<u32> {
    let group_size = {
        let sim = borrow_state(state)?;
        selected_entry(&sim)
            .and_then(|row| row.suggested_group_size)
            .unwrap_or(0)
    };
    state.push(Val::Num(group_size as f64));
    Ok(1)
}

pub(super) fn get_quest_green_range(state: &mut LuaState) -> LuaResult<u32> {
    let level = borrow_state(state)?.player.level;
    state.push(Val::Num(f64::from(
        crate::c_api::quest_difficulty::quest_green_range(level),
    )));
    Ok(1)
}

pub(super) fn get_daily_quests_completed(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .quest_log_entries
        .daily_quest_ids
        .iter()
        .filter(|id| **id != 0)
        .count();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(super) fn get_max_daily_quests(state: &mut LuaState) -> LuaResult<u32> {
    let limit = borrow_state(state)?.quest_log_entries.daily_quest_ids.len();
    state.push(Val::Num(limit as f64));
    Ok(1)
}

fn selected_entry(sim: &SimState) -> Option<&QuestLogEntry> {
    let id = sim.selected_quest_log_id?;
    sim.quest_log_entries
        .entries
        .iter()
        .find(|row| row.quest_id as u32 == id)
}

pub(super) fn get_num_quest_log_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let count = selected_entry(&*borrow_state(state)?).map_or(0, |row| row.reward_items.len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(super) fn get_quest_log_reward_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = supplied_index(state)?;
    let item =
        selected_entry(&*borrow_state(state)?).and_then(|row| row.reward_items.get(index).cloned());
    super::handlers::push_quest_reward_item(state, item)
}

pub(super) fn get_quest_log_reward_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell = selected_entry(&*borrow_state(state)?).and_then(|row| row.reward_spell.clone());
    let Some(spell) = spell else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let texture = create_string(state, &spell.texture);
    let name = create_string(state, &spell.name);
    for value in [
        texture,
        name,
        Val::Bool(spell.is_trade_skill),
        Val::Bool(spell.is_learned),
    ] {
        state.push(value);
    }
    Ok(4)
}

pub(super) fn get_quest_log_reward_arena_points(state: &mut LuaState) -> LuaResult<u32> {
    let points = selected_entry(&*borrow_state(state)?).map_or(0, |row| row.reward_arena_points);
    state.push(Val::Num(points as f64));
    Ok(1)
}

pub(super) fn get_quest_log_reward_talents(state: &mut LuaState) -> LuaResult<u32> {
    let talents = selected_entry(&*borrow_state(state)?).map_or(0, |row| row.reward_talents);
    state.push(Val::Num(talents as f64));
    Ok(1)
}

pub(super) fn process_quest_log_reward_factions(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let selected = sim.selected_quest_log_id;
    sim.quest_log_entries.process_reward_factions(selected);
    Ok(0)
}

pub(super) fn get_num_quest_log_reward_factions(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .quest_log_entries
        .processed_reward_factions
        .len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(super) fn get_quest_log_reward_faction_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = supplied_index(state)?;
    let (id, amount) = borrow_state(state)?
        .quest_log_entries
        .processed_reward_factions
        .get(index)
        .copied()
        .unwrap_or((0, 0));
    state.push(Val::Num(id as f64));
    state.push(Val::Num(amount as f64));
    Ok(2)
}

pub(super) fn get_quest_log_selection(state: &mut LuaState) -> LuaResult<u32> {
    let index = {
        let sim = borrow_state(state)?;
        sim.selected_quest_log_id
            .and_then(|id| {
                sim.quest_log_entries
                    .entries
                    .iter()
                    .position(|row| row.quest_id as u32 == id)
            })
            .map_or(0, |index| index + 1)
    };
    state.push(Val::Num(index as f64));
    Ok(1)
}
