//! Legacy global quest handlers shared by classic profile quest panels.

use crate::c_api::quest_objectives;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_string_static,
};
use crate::lua_bridge::FromStack;
use crate::quest_poi_blobs;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::data::QuestLogEntry;
use super::handlers::{
    entry_at, find_quest_by_id, fire_event_with_args, is_world_quest, join_objective_text,
    objective_at, push_optional_bool, push_optional_i32, quest_exists, quest_list_index,
    reward_items_for_quest, selected_criteria_spell, selected_log_index, selected_quest_id,
    selected_static_quest, set_selected_quest_id,
};

const SEEDED_REWARD_ITEM_ID: i32 = 6948;
const QUEST_GREETING_TEXT: &str = "How can I help you, adventurer?";
const QUEST_REWARD_TEXT: &str = "You will receive:";

pub fn get_greeting_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = create_string_static(state, QUEST_GREETING_TEXT);
    state.push(text);
    Ok(1)
}

pub fn get_num_active_quests(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.gossip.active_quests.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub fn get_num_available_quests(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.gossip.available_quests.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub fn get_active_title(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let row = borrow_state(state)?
        .gossip
        .active_quests
        .get(index)
        .cloned();
    let Some(row) = row else {
        return Ok(0);
    };
    let title = create_string(state, &row.title);
    state.push(title);
    let ready = if quest_objectives::ENABLED {
        let sim = borrow_state(state)?;
        crate::c_api::quest_readiness::is_ready(&sim, row.quest_id as i32)
    } else {
        row.is_complete.unwrap_or(false)
    };
    state.push(Val::Bool(ready));
    Ok(2)
}

pub fn get_active_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let quest_id = borrow_state(state)?
        .gossip
        .active_quests
        .get(index)
        .map(|row| row.quest_id)
        .unwrap_or(0);
    state.push(Val::Num(quest_id as f64));
    Ok(1)
}

pub fn is_active_quest_trivial(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let is_trivial = borrow_state(state)?
        .gossip
        .active_quests
        .get(index)
        .is_some_and(|row| row.is_trivial);
    state.push(Val::Bool(is_trivial));
    Ok(1)
}

pub fn get_available_title(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let title = borrow_state(state)?
        .gossip
        .available_quests
        .get(index)
        .map(|row| row.title.clone())
        .unwrap_or_default();
    let title = create_string(state, &title);
    state.push(title);
    Ok(1)
}

pub fn get_available_quest_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let row = borrow_state(state)?
        .gossip
        .available_quests
        .get(index)
        .cloned();
    let Some(row) = row else {
        return Ok(0);
    };
    state.push(Val::Bool(row.is_trivial));
    push_optional_i32(state, row.frequency);
    push_optional_bool(state, row.repeatable);
    state.push(Val::Bool(row.is_legendary));
    state.push(Val::Num(row.quest_id as f64));
    state.push(Val::Bool(row.is_important));
    state.push(Val::Bool(row.is_meta));
    state.push(Val::Num(row.quest_info_id as f64));
    Ok(8)
}

pub fn select_active_quest(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let row = borrow_state(state)?
        .gossip
        .active_quests
        .get(index)
        .cloned();
    let Some(row) = row else {
        return Ok(0);
    };
    if quest_objectives::ENABLED {
        return crate::c_api::quest_readiness::select_active_dialog(state, row.quest_id);
    }
    set_selected_quest_id(state, row.quest_id as i32)?;
    let event = if row.is_complete.unwrap_or(false) {
        "QUEST_COMPLETE"
    } else {
        "QUEST_PROGRESS"
    };
    fire_event_with_args(state, event, &[]);
    Ok(0)
}

pub fn select_available_quest(state: &mut LuaState) -> LuaResult<u32> {
    let index = quest_list_index(state)?;
    let row = borrow_state(state)?
        .gossip
        .available_quests
        .get(index)
        .cloned();
    let Some(row) = row else {
        return Ok(0);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        let offer = sim
            .quest_log_entries
            .snapshot_gossip_offer(&row)
            .map_err(|_| rilua::runtime_error("selected quest ID exceeds the rich-log range"))?;
        sim.quest_log_entries.readiness.dialog = None;
        sim.pending_quest_offer = Some(offer);
        sim.selected_quest_log_id = Some(row.quest_id);
    }
    fire_event_with_args(state, "QUEST_DETAIL", &[]);
    Ok(0)
}

pub fn get_quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = selected_quest_id(state)?;
    state.push(Val::Num(quest_id as f64));
    Ok(1)
}

pub fn get_title_text(state: &mut LuaState) -> LuaResult<u32> {
    let title = selected_static_quest(state)?
        .map(|quest| quest.title)
        .unwrap_or("Quest");
    let title = create_string(state, title);
    state.push(title);
    Ok(1)
}

pub fn get_quest_text(state: &mut LuaState) -> LuaResult<u32> {
    let description = selected_static_quest(state)?
        .map(|quest| quest.description)
        .unwrap_or("");
    let description = create_string(state, description);
    state.push(description);
    Ok(1)
}

pub fn get_objective_text(state: &mut LuaState) -> LuaResult<u32> {
    let objective_text = if quest_objectives::ENABLED {
        quest_objectives::read_selected_objective_text(state)?
    } else {
        selected_static_quest(state)?
            .map(join_objective_text)
            .unwrap_or_default()
    };
    let objective_text = create_string(state, &objective_text);
    state.push(objective_text);
    Ok(1)
}

/// Legacy API: unsupplied material is nil; the original QuestFrame chooses Parchment.
pub fn get_quest_background_material(state: &mut LuaState) -> LuaResult<u32> {
    let material = {
        let sim = borrow_state(state)?;
        sim.selected_quest_log_id
            .and_then(|id| i32::try_from(id).ok())
            .and_then(|id| sim.quest_log_entries.background_materials.get(&id))
            .cloned()
    };
    let value = material
        .map(|material| create_string(state, &material))
        .unwrap_or(Val::Nil);
    state.push(value);
    Ok(1)
}

pub fn get_reward_text(state: &mut LuaState) -> LuaResult<u32> {
    let reward_text = create_string_static(state, QUEST_REWARD_TEXT);
    state.push(reward_text);
    Ok(1)
}

pub fn get_num_quest_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = selected_quest_id(state)?;
    let count = reward_items_for_quest(state, quest_id)?.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub fn get_quest_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let reward_type = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let reward_index = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0) as usize;
    if reward_type != "reward" || reward_index == 0 {
        return Ok(0);
    }
    let quest_id = selected_quest_id(state)?;
    let item = reward_items_for_quest(state, quest_id)?
        .get(reward_index - 1)
        .cloned();
    let Some(item) = item else {
        return Ok(0);
    };
    let name = create_string(state, &item.name);
    let texture = create_string(state, &item.texture);
    state.push(name);
    state.push(texture);
    state.push(Val::Num(item.count as f64));
    state.push(Val::Num(item.quality as f64));
    state.push(Val::Bool(item.is_usable));
    state.push(Val::Num(SEEDED_REWARD_ITEM_ID as f64));
    Ok(6)
}

pub fn get_quest_item_info_loot_type(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

pub fn get_num_quest_leaderboards(state: &mut LuaState) -> LuaResult<u32> {
    if quest_objectives::ENABLED {
        return quest_objectives::get_legacy_objective_count(state);
    }
    let log_index = Option::<f64>::from_stack(state, 1)?
        .map(|index| index as i32)
        .unwrap_or(selected_log_index(state)?);
    let count = match entry_at(log_index) {
        Some(QuestLogEntry::Quest { objectives, .. }) => objectives.len() as i32,
        _ => 0,
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub fn get_quest_log_leaderboard(state: &mut LuaState) -> LuaResult<u32> {
    if quest_objectives::ENABLED {
        return quest_objectives::get_legacy_leaderboard(state);
    }
    let objective_index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let log_index = Option::<f64>::from_stack(state, 2)?
        .map(|index| index as i32)
        .unwrap_or(selected_log_index(state)?);
    let Some(objective) = objective_at(log_index, objective_index) else {
        return Ok(0);
    };
    let text = create_string(state, objective.text);
    let obj_type = create_string(state, objective.obj_type);
    state.push(text);
    state.push(obj_type);
    state.push(Val::Bool(objective.finished));
    Ok(3)
}

pub fn get_quest_log_quest_text(state: &mut LuaState) -> LuaResult<u32> {
    if quest_objectives::ENABLED {
        return quest_objectives::get_legacy_quest_text(state);
    }
    let quest_id = selected_quest_id(state)?;
    let Some((
        _,
        QuestLogEntry::Quest {
            description,
            objectives,
            ..
        },
    )) = find_quest_by_id(quest_id)
    else {
        let empty1 = create_string_static(state, "");
        state.push(empty1);
        let empty2 = create_string_static(state, "");
        state.push(empty2);
        return Ok(2);
    };

    let objective_lines = objectives
        .iter()
        .map(|o| o.text)
        .collect::<Vec<_>>()
        .join("\n");
    let description_val = create_string(state, description);
    let objectives_val = create_string(state, &objective_lines);
    state.push(description_val);
    state.push(objectives_val);
    Ok(2)
}

pub fn get_quest_poi_blob_count(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as u32;
    state.push(Val::Num(
        quest_poi_blobs::get_quest_blobs(quest_id).len() as f64
    ));
    Ok(1)
}

pub fn have_quest_data(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    state.push(Val::Bool(
        quest_exists(quest_id) || is_world_quest(quest_id),
    ));
    Ok(1)
}

pub fn is_current_quest_failed(state: &mut LuaState) -> LuaResult<u32> {
    let is_failed = {
        let sim_state = borrow_state(state)?;
        sim_state
            .selected_quest_log_id
            .map(|id| id as i32)
            .and_then(|quest_id| {
                sim_state
                    .quest_log_entries
                    .entries
                    .iter()
                    .find(|entry| entry.quest_id == quest_id)
                    .map(|entry| entry.is_failed)
            })
            .unwrap_or(false)
    };
    state.push(Val::Bool(is_failed));
    Ok(1)
}

pub fn is_quest_sequenced(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

pub fn get_quest_log_completion_text(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub fn get_quest_log_criteria_spell(state: &mut LuaState) -> LuaResult<u32> {
    push_selected_criteria_spell(state)
}

pub fn get_criteria_spell(state: &mut LuaState) -> LuaResult<u32> {
    push_selected_criteria_spell(state)
}

fn push_selected_criteria_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some((spell_id, spell_name, spell_texture, finished)) = selected_criteria_spell(state)?
    else {
        return Ok(0);
    };
    let spell_name = create_string(state, &spell_name);
    let spell_texture = create_string(state, &spell_texture);
    state.push(Val::Num(spell_id as f64));
    state.push(spell_name);
    state.push(spell_texture);
    state.push(Val::Bool(finished));
    Ok(4)
}

pub fn get_quest_progress_bar_percent(state: &mut LuaState) -> LuaResult<u32> {
    if quest_objectives::ENABLED {
        return quest_objectives::get_percentage(state);
    }
    state.push(Val::Num(0.0));
    Ok(1)
}

/// Classic `GetQuestGreenRange`: one number derived from the current player level.
///
/// The classic `Blizzard_UIParent` TOCs select the `Vanilla`/`Mists` body whose
/// `GetRelativeDifficultyColor` calls this global (`era` cache
/// `Blizzard_UIParent/Vanilla/UIParent.lua:1419`, `mists` cache
/// `Blizzard_UIParent/Mists/UIParent.lua:1628`). No pinned
/// `Blizzard_APIDocumentationGenerated` declares it, so the signature comes from
/// that consumer: no arguments, one number.
///
/// The level/5 band table is the source-derived Wrath 12340 table
/// (`docs/specs/wrath-quest-abandon.md`). Native Mists/Era/Anniversary bands are
/// UNKNOWN; applying the classic-family table there is an EMULATED default that
/// only moves the displayed quest-difficulty color.
pub(super) fn get_quest_green_range(state: &mut LuaState) -> LuaResult<u32> {
    const QUEST_GREEN_RANGES: [u8; 20] =
        [4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];
    let level = borrow_state(state)?.player.level;
    let band = (level.max(0) as usize / 5).min(QUEST_GREEN_RANGES.len() - 1);
    state.push(Val::Num(f64::from(QUEST_GREEN_RANGES[band])));
    Ok(1)
}
