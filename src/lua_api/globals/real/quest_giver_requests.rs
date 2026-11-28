//! Retail and Forever quest-giver requests: `AcceptQuest`, `DeclineQuest` and
//! `GetQuestReward(choice)`.
//!
//! Declarations: Retail `_G` secure-publication rows (no documentation block).
//! Consumers: `Blizzard_StaticPopup_Game/GameDialogDefs.lua`
//! (`CONFIRM_ACCEPT_PVP_QUEST` accepts or declines the offered quest,
//! `CONFIRM_COMPLETE_EXPENSIVE_QUEST` claims `QuestInfoFrame.itemChoice`) and
//! `Mainline/QuestFrame.lua` (accept and complete buttons).
//! The offer is the existing `pending_quest_offer` set by
//! `C_GossipInfo.SelectAvailableQuest`; the claim is the reward dialog opened by
//! `C_GossipInfo.SelectActiveQuest` / `CompleteQuest` for a ready quest.
//! INFERRED (R-ag10-8): the server answers an accept with `QUEST_ACCEPTED(questID)`
//! and closes the dialog (`QUEST_FINISHED`); a decline only closes the dialog; a
//! claim turns the quest in (`QUEST_TURNED_IN(questID, 0, 0)`, no reward grants)
//! and closes the dialog. A claim without a valid choice, when the quest offers
//! choices, changes nothing.

use crate::c_api::quest_readiness::{QuestDialogPhase, has_active_npc_quest, is_ready};
use crate::lua_api::SimState;
use crate::lua_api::globals::admin_quests::commit_selected_quest_turn_in;
use crate::lua_api::globals::quest_verbs::append_quest_log_entry;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn take_offer_into_log(sim: &mut SimState) -> Option<i32> {
    let offer = sim.pending_quest_offer.take()?;
    let quest_id = offer.quest_id;
    append_quest_log_entry(&mut sim.quest_log, u32::try_from(quest_id).ok()?);
    let known = sim
        .quest_log_entries
        .entries
        .iter()
        .any(|e| e.quest_id == quest_id);
    if !known {
        sim.quest_log_entries.entries.push(offer);
    }
    Some(quest_id)
}

fn accept_quest(state: &mut LuaState) -> LuaResult<u32> {
    let Some(quest_id) = take_offer_into_log(&mut *borrow_state_mut(state)?) else {
        return Ok(0);
    };
    dispatch_event_now(state, "QUEST_ACCEPTED", &[Val::Num(f64::from(quest_id))])?;
    dispatch_event_now(state, "QUEST_FINISHED", &[])?;
    Ok(0)
}

fn decline_quest(state: &mut LuaState) -> LuaResult<u32> {
    let had_dialog = {
        let mut sim = borrow_state_mut(state)?;
        let offer = sim.pending_quest_offer.take();
        let dialog = sim.quest_log_entries.readiness.dialog.take();
        offer.is_some() || dialog.is_some()
    };
    if had_dialog {
        dispatch_event_now(state, "QUEST_FINISHED", &[])?;
    }
    Ok(0)
}

/// The quest whose reward dialog is open, ready and selected.
fn claimable_quest(sim: &SimState) -> Option<i32> {
    let (quest_id, phase) = sim.quest_log_entries.readiness.dialog?;
    let selected = sim.selected_quest_log_id == u32::try_from(quest_id).ok();
    let ready = has_active_npc_quest(sim, quest_id) && is_ready(sim, quest_id);
    (phase == QuestDialogPhase::Rewards && selected && ready).then_some(quest_id)
}

fn choice_is_valid(sim: &SimState, quest_id: i32, choice: Option<i64>) -> bool {
    let choices = sim
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .map_or(0, |entry| entry.reward_choices.len() as i64);
    choices == 0 || choice.is_some_and(|choice| (1..=choices).contains(&choice))
}

fn get_quest_reward(state: &mut LuaState) -> LuaResult<u32> {
    let choice = Option::<i64>::from_stack(state, 1)?;
    let turned_in = {
        let mut sim = borrow_state_mut(state)?;
        let Some(quest_id) = claimable_quest(&sim) else {
            return Ok(0);
        };
        if !choice_is_valid(&sim, quest_id, choice) {
            return Ok(0);
        }
        sim.quest_log_entries.readiness.dialog = None;
        commit_selected_quest_turn_in(&mut sim)
    };
    let Some(quest_id) = turned_in else {
        return Ok(0);
    };
    let id = Val::Num(f64::from(quest_id));
    dispatch_event_now(
        state,
        "QUEST_TURNED_IN",
        &[id, Val::Num(0.0), Val::Num(0.0)],
    )?;
    dispatch_event_now(state, "QUEST_FINISHED", &[])?;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "AcceptQuest", accept_quest)?;
    LuaApiMut::register_function(lua, "DeclineQuest", decline_quest)?;
    LuaApiMut::register_function(lua, "GetQuestReward", get_quest_reward)
}
