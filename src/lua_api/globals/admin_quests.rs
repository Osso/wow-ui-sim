//! Rilua A_Admin handlers — quest NPC / gossip dialog seeding.

use crate::lua_api::globals::quest_surface::data::{QUEST_LOG, QuestLogEntry};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::sim_substates::{GossipQuestRow, GossipState, QuestRewardItem};
use crate::lua_api::state::QuestLogEntry as RichQuestLogEntry;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::LuaResult;
#[cfg(not(feature = "client-wrath"))]
use rilua::Val;
use rilua::vm::state::LuaState;

const DEFAULT_QUEST_ID: u32 = 80000;
const DEFAULT_QUEST_TITLE: &str = "The Lost Expedition";
const COMPLETED_REWARD_QUEST_ID: u32 = 80001;
const INCOMPLETE_ACTIVE_QUEST_ID: u32 = 80000;
const AVAILABLE_MULTI_QUEST_ID: u32 = 80002;
const COMPLETED_REWARD_NAME: &str = "Earthen Lockbox";
const COMPLETED_REWARD_TEXTURE: &str = "Interface\\Icons\\INV_Box_01";
const QUEST_GREETING_TEXT: &str = "How can I help you, adventurer?";

pub(super) fn register_quests(b: TableBuilder) -> LuaResult<TableBuilder> {
    #[cfg(all(feature = "profile-retail", feature = "retail-12-0-5"))]
    let b = b.set_function(
        "RequestQuestAcceptConfirmation",
        request_quest_accept_confirmation,
    )?;
    let b = if crate::c_api::quest_objectives::ENABLED {
        b.set_function(
            "SetQuestObjectives",
            super::admin_quest_objectives::set_quest_objectives,
        )?
        .set_function(
            "SetQuestReadiness",
            super::real::quest_dialog::set_quest_readiness,
        )?
    } else {
        b
    };
    let b = register_turn_in_responses(b)?;
    b.set_function("OpenQuestNpc", open_quest_npc)?
        .set_function("OpenMultiQuestNpc", open_multi_quest_npc)?
        .set_function("CloseQuestNpc", close_quest_npc)
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
