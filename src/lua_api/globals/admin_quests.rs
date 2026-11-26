//! Rilua A_Admin handlers — quest NPC / gossip dialog seeding.

use crate::lua_api::globals::quest_surface::data::{QUEST_LOG, QuestLogEntry};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, table_get};
use crate::lua_api::sim_substates::{GossipQuestRow, GossipState, QuestRewardItem};
use crate::lua_api::state::QuestLogEntry as RichQuestLogEntry;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
#[cfg(not(feature = "client-wrath"))]
use rilua::Val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

const DEFAULT_QUEST_ID: u32 = 80000;
const DEFAULT_QUEST_TITLE: &str = "The Lost Expedition";
const COMPLETED_REWARD_QUEST_ID: u32 = 80001;
const INCOMPLETE_ACTIVE_QUEST_ID: u32 = 80000;
const AVAILABLE_MULTI_QUEST_ID: u32 = 80002;
const COMPLETED_REWARD_NAME: &str = "Earthen Lockbox";
const COMPLETED_REWARD_TEXTURE: &str = "Interface\\Icons\\INV_Box_01";
const QUEST_GREETING_TEXT: &str = "How can I help you, adventurer?";
const SET_QUEST_LOG_ENTRY_USAGE: &str =
    "Usage: A_Admin.SetQuestLogEntry(questID, title, isComplete)";
#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
const SET_QUEST_LOG_REWARD_ITEMS_ERROR: &str =
    "SetQuestLogRewardItems: expected an existing quest and valid reward rows";

pub(super) fn register_quests(b: TableBuilder) -> LuaResult<TableBuilder> {
    #[cfg(all(feature = "profile-retail", feature = "retail-12-0-5"))]
    let b = b.set_function(
        "RequestQuestAcceptConfirmation",
        request_quest_accept_confirmation,
    )?;
    let b = register_quest_state_inputs(b)?;
    let b = if crate::c_api::quest_objectives::ENABLED {
        b.set_function(
            "SetQuestObjectives",
            super::admin_quest_objectives::set_quest_objectives,
        )?
        .set_function(
            "SetQuestReadiness",
            super::real::quest_dialog::set_quest_readiness,
        )?
        .set_function(
            "SetQuestProgressRequirements",
            super::admin_quest_objectives::set_quest_progress_requirements,
        )?
    } else {
        b
    };
    let b = register_turn_in_responses(b)?;
    b.set_function("OpenQuestNpc", open_quest_npc)?
        .set_function("OpenMultiQuestNpc", open_multi_quest_npc)?
        .set_function("CloseQuestNpc", close_quest_npc)
}

fn register_quest_state_inputs(b: TableBuilder) -> LuaResult<TableBuilder> {
    let b = b.set_function("SetQuestLogEntry", set_quest_log_entry)?;
    #[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
    let b = b.set_function("SetQuestLogRewardItems", set_quest_log_reward_items)?;
    b.set_function("SetQuestGroupCreatable", set_quest_group_creatable)?
        .set_function("SetQuestLogSpecialItem", set_quest_log_special_item)
}

#[cfg(feature = "client-wrath")]
fn register_turn_in_responses(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function(
        "OfferQuestReward",
        super::real::wrath_quest_dialog::offer_response,
    )?
    .set_function(
        "TurnInSelectedQuest",
        super::real::wrath_quest_dialog::accept_response,
    )
}

#[cfg(not(feature = "client-wrath"))]
fn register_turn_in_responses(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("TurnInSelectedQuest", turn_in_selected_quest)
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-0-5"))]
fn request_quest_accept_confirmation(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let quest_title = String::from_stack(state, 2)?;
    let pending_offer = crate::lua_api::methods::borrow_state(state)?
        .pending_quest_offer
        .as_ref()
        .map(|offer| offer.quest_id);
    let Some(quest_id) = pending_offer else {
        return Ok(0);
    };

    // Root both strings across allocations and every synchronous listener.
    let saved_top = state.top;
    let name = crate::lua_api::methods::create_string(state, &name);
    state.push(name);
    let quest_title = crate::lua_api::methods::create_string(state, &quest_title);
    state.push(quest_title);
    let result = dispatch_event_now(
        state,
        "QUEST_ACCEPT_CONFIRM",
        &[name, quest_title, rilua::Val::Num(quest_id as f64)],
    );
    state.top = saved_top;
    result?;
    Ok(0)
}

fn set_quest_log_entry(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    if quest_id <= 0 {
        return Err(runtime_error(SET_QUEST_LOG_ENTRY_USAGE));
    }
    let title = String::from_stack(state, 2)?;
    let is_complete = bool::from_stack(state, 3)?;
    let mut sim = borrow_state_mut(state)?;
    set_supplied_quest_log_entry(&mut sim, quest_id, title, is_complete);
    Ok(0)
}

fn set_supplied_quest_log_entry(
    sim: &mut crate::lua_api::SimState,
    quest_id: i32,
    title: String,
    is_complete: bool,
) {
    if let Some(entry) = sim
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == quest_id)
    {
        entry.title = title;
        entry.is_complete = is_complete;
        return;
    }
    sim.quest_log_entries
        .entries
        .push(RichQuestLogEntry::new_supplied(
            quest_id,
            title,
            is_complete,
        ));
    sim.quest_log.push(quest_id as u32);
}

#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
fn set_quest_log_reward_items(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let items = decode_quest_reward_items(state)?;
    if quest_id <= 0 || !items.iter().all(valid_quest_reward_item) {
        return Err(runtime_error(SET_QUEST_LOG_REWARD_ITEMS_ERROR));
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(entry) = sim
        .quest_log_entries
        .entries
        .iter_mut()
        .find(|entry| entry.quest_id == quest_id)
    else {
        return Err(runtime_error(SET_QUEST_LOG_REWARD_ITEMS_ERROR));
    };
    entry.reward_items = items;
    Ok(0)
}

#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
fn decode_quest_reward_items(state: &mut LuaState) -> LuaResult<Vec<QuestRewardItem>> {
    let Val::Table(rows_ref) = stack_val(state, 2) else {
        return Err(runtime_error(SET_QUEST_LOG_REWARD_ITEMS_ERROR));
    };
    let row_values = {
        let rows = state
            .gc
            .tables
            .get(rows_ref)
            .ok_or_else(|| runtime_error(SET_QUEST_LOG_REWARD_ITEMS_ERROR))?;
        (1..=rows.len(&state.gc.string_arena))
            .map(|index| rows.get_int(index as i64))
            .collect::<Vec<_>>()
    };
    row_values
        .into_iter()
        .map(|row| decode_quest_reward_item(state, row))
        .collect()
}

#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
fn decode_quest_reward_item(state: &mut LuaState, row: Val) -> LuaResult<QuestRewardItem> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(SET_QUEST_LOG_REWARD_ITEMS_ERROR));
    }
    Ok(QuestRewardItem {
        name: read_reward_field(state, row, "name")?,
        texture: read_reward_field(state, row, "texture")?,
        count: read_reward_field(state, row, "count")?,
        quality: read_reward_field(state, row, "quality")?,
        is_usable: read_reward_field(state, row, "isUsable")?,
        item_id: read_reward_field(state, row, "itemID")?,
    })
}

#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
fn read_reward_field<T: FromStack>(state: &mut LuaState, row: Val, key: &str) -> LuaResult<T> {
    let top = state.top;
    let field = table_get(state, row, key);
    state.push(field);
    let value = T::from_stack(state, -1);
    state.top = top;
    value
}

#[cfg(any(feature = "profile-retail", feature = "client-wowforever"))]
fn valid_quest_reward_item(item: &QuestRewardItem) -> bool {
    !item.name.is_empty()
        && !item.texture.is_empty()
        && item.count > 0
        && (0..=7).contains(&item.quality)
        && item.item_id.is_none_or(|item_id| item_id > 0)
}

fn set_quest_group_creatable(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
    let can_create = bool::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .quest_log_entries
        .group_creation_by_quest
        .insert(quest_id, can_create);
    Ok(0)
}

fn set_quest_log_special_item(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let item_id = Option::<u32>::from_stack(state, 2)?;
    let items = &mut borrow_state_mut(state)?
        .quest_log_entries
        .quest_special_item_inputs
        .special_item_ids_by_log_index;
    if let Some(item_id) = item_id {
        items.insert(index, item_id);
    } else {
        items.remove(&index);
    }
    Ok(0)
}

pub(super) fn open_quest_npc(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = Option::<f64>::from_stack(state, 1)?
        .filter(|id| *id > 0.0)
        .map(|id| id as u32)
        .unwrap_or(DEFAULT_QUEST_ID);
    let title = Option::<String>::from_stack(state, 2)?
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| quest_title(quest_id));

    {
        let mut sim = borrow_state_mut(state)?;
        sim.gossip.active = true;
        sim.gossip.text = QUEST_GREETING_TEXT.to_string();
        sim.gossip.options.clear();
        sim.gossip.active_quests.clear();
        sim.quest_log_entries.readiness.dialog = None;
        sim.gossip.available_quests = vec![GossipQuestRow {
            quest_id,
            quest_info_id: quest_id,
            quest_level: 80,
            title,
            is_complete: Some(false),
            ..Default::default()
        }];
        refresh_gossip_counts(&mut sim.gossip);
    }

    dispatch_event_now(state, "GOSSIP_SHOW", &[])?;
    Ok(0)
}

pub(super) fn open_multi_quest_npc(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.gossip.active = true;
        sim.gossip.text = QUEST_GREETING_TEXT.to_string();
        sim.gossip.options.clear();
        sim.gossip.active_quests = vec![
            quest_row(COMPLETED_REWARD_QUEST_ID, true),
            quest_row(INCOMPLETE_ACTIVE_QUEST_ID, false),
        ];
        sim.gossip.available_quests = vec![quest_row(AVAILABLE_MULTI_QUEST_ID, false)];
        seed_completed_quest_reward(&mut sim.quest_log_entries.entries);
        if crate::c_api::quest_objectives::ENABLED {
            for id in [COMPLETED_REWARD_QUEST_ID, INCOMPLETE_ACTIVE_QUEST_ID] {
                crate::c_api::quest_readiness::sync_gossip_readiness(&mut sim, id as i32);
            }
        }
        sim.quest_log_entries.readiness.dialog = None;
        refresh_gossip_counts(&mut sim.gossip);
    }

    dispatch_event_now(state, "QUEST_GREETING", &[])?;
    Ok(0)
}

pub(super) fn close_quest_npc(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.gossip.active = false;
        sim.quest_log_entries.readiness.dialog = None;
        sim.gossip.text.clear();
        sim.gossip.options.clear();
        sim.gossip.active_quests.clear();
        sim.gossip.available_quests.clear();
        sim.gossip.num_options = 0;
        sim.gossip.num_active_quests = 0;
        sim.gossip.num_available_quests = 0;
    }

    dispatch_event_now(state, "GOSSIP_CLOSED", &[])?;
    Ok(0)
}

/// Simulator-only selected-completed turn-in. No inventory/currency/XP grants.
/// Cached QuestLogDocumentation declares questID, xpReward, moneyReward;
/// zero rewards are authored policy, not inferred native reward amounts.
#[cfg(not(feature = "client-wrath"))]
fn turn_in_selected_quest(state: &mut LuaState) -> LuaResult<u32> {
    let turned_in = {
        let mut sim = borrow_state_mut(state)?;
        commit_selected_quest_turn_in(&mut sim)
    };
    if let Some(quest_id) = turned_in {
        dispatch_event_now(
            state,
            "QUEST_TURNED_IN",
            &[Val::Num(f64::from(quest_id)), Val::Num(0.0), Val::Num(0.0)],
        )?;
    }
    state.push(Val::Bool(turned_in.is_some()));
    Ok(1)
}

#[cfg(not(feature = "client-wrath"))]
fn commit_selected_quest_turn_in(sim: &mut crate::lua_api::SimState) -> Option<i32> {
    let selected = sim.selected_quest_log_id?;
    let quest_id = i32::try_from(selected).ok()?;
    if !sim.quest_log_entries.turn_in(quest_id) {
        return None;
    }
    sim.quest_log.retain(|id| *id != selected);
    sim.quest_watch_types.remove(&quest_id);
    clear_turned_in_quest_dialog(sim, quest_id);
    Some(quest_id)
}

pub(super) fn clear_turned_in_quest_dialog(sim: &mut crate::lua_api::SimState, quest_id: i32) {
    let matches_pending = sim
        .pending_quest_offer
        .as_ref()
        .is_some_and(|offer| offer.quest_id == quest_id);
    if matches_pending {
        sim.pending_quest_offer = None;
    }
    sim.selected_quest_log_id = None;
    sim.gossip
        .active_quests
        .retain(|row| row.quest_id != quest_id as u32);
    sim.gossip
        .available_quests
        .retain(|row| row.quest_id != quest_id as u32);
    refresh_gossip_counts(&mut sim.gossip);
}

fn quest_row(quest_id: u32, is_complete: bool) -> GossipQuestRow {
    GossipQuestRow {
        quest_id,
        quest_info_id: quest_id,
        quest_level: 80,
        title: quest_title(quest_id),
        is_complete: Some(is_complete),
        ..Default::default()
    }
}

fn refresh_gossip_counts(gossip: &mut GossipState) {
    gossip.num_options = gossip.options.len() as i32;
    gossip.num_active_quests = gossip.active_quests.len() as i32;
    gossip.num_available_quests = gossip.available_quests.len() as i32;
}

fn seed_completed_quest_reward(entries: &mut [RichQuestLogEntry]) {
    let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.quest_id == COMPLETED_REWARD_QUEST_ID as i32)
    else {
        return;
    };
    if !entry.reward_items.is_empty() {
        return;
    }
    entry.reward_items.push(QuestRewardItem {
        name: COMPLETED_REWARD_NAME.to_string(),
        texture: COMPLETED_REWARD_TEXTURE.to_string(),
        count: 1,
        quality: 3,
        is_usable: true,
        item_id: None,
    });
}

fn quest_title(quest_id: u32) -> String {
    QUEST_LOG
        .iter()
        .find_map(|entry| match entry {
            QuestLogEntry::Quest {
                quest_id: id,
                title,
                ..
            } if *id == quest_id as i32 => Some((*title).to_string()),
            _ => None,
        })
        .unwrap_or_else(|| DEFAULT_QUEST_TITLE.to_string())
}
