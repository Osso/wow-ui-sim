//! Quest verbs that publish accepted IDs and rich quest snapshots, mutate
//! per-session quest / achievement tracking fields, and queue quest events.
//!
//! Migrates 8 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `ConfirmAcceptQuest()`               — consumes `pending_quest_offer`,
//!                                            publishes rich/lightweight membership,
//!                                            fires `QUEST_ACCEPTED`.
//! - `CloseQuestFrame()`                  — fires `QUEST_FINISHED`.
//! - `AcknowledgeAutoQuestPopUp(id)`      — silent dismiss. Accepts any arg.
//! - `QuestChoiceFrame_SetActiveChoice(id)` — write `quest_choice_id`.
//! - `QuestMapLogTitleButton_OnClick(id)` — write `selected_quest_log_id`.
//! - `SetAbandonQuest()`                  — snapshot the currently selected
//!                                            quest into `abandon_quest_id`.
//! - `SetTrackedAchievement(id, tracked)` — add/remove achievement id from
//!                                            `tracked_achievements`.
//! - `UntrackAchievement(id)`             — remove id (no-op when absent).
//!
//! Wrath marks with `SetAbandonQuest`, then commits through `AbandonQuest`
//! and queues `QUEST_LOG_UPDATE`. Other profiles retain the immediate
//! `SetAbandonQuest` transition and its `QUEST_REMOVED` notification.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::event::Event;
use crate::lua_api::frame::methods::methods_hierarchy::reparent_widget;
use crate::lua_api::methods::{borrow_state_mut, extract_frame_id};
use crate::lua_api::state::QuestPortraitState;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn push_event(state: &mut LuaState, name: &str) -> LuaResult<()> {
    borrow_state_mut(state)?.events.push(Event {
        name: name.to_string(),
        args: Vec::new(),
    });
    Ok(())
}

fn stack_u32(state: &mut LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

fn stack_i32(state: &mut LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_bool(state: &mut LuaState, index: i32) -> Option<bool> {
    match stack_val(state, index) {
        Val::Bool(b) => Some(b),
        Val::Nil => None,
        Val::Num(n) => Some(n != 0.0),
        _ => None,
    }
}

/// Consume the selected snapshot and publish rich/lightweight membership before
/// the existing queued QUEST_ACCEPTED. No pending offer remains a silent no-op.
fn confirm_accept_quest(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut st = borrow_state_mut(state)?;
        let Some(offer) = st.pending_quest_offer.as_ref() else {
            return Ok(0);
        };
        let id = u32::try_from(offer.quest_id)
            .map_err(|_| rilua::runtime_error("pending quest ID must be nonnegative"))?;
        let offer = st
            .pending_quest_offer
            .take()
            .expect("validated pending offer");
        append_quest_log_entry(&mut st.quest_log, id);
        if !st
            .quest_log_entries
            .entries
            .iter()
            .any(|entry| entry.quest_id == offer.quest_id)
        {
            st.quest_log_entries.entries.push(offer);
        }
    }
    push_event(state, "QUEST_ACCEPTED")?;
    Ok(0)
}

fn append_quest_log_entry(quest_log: &mut Vec<u32>, quest_id: u32) {
    if !quest_log.contains(&quest_id) {
        quest_log.push(quest_id);
    }
}

#[cfg(feature = "client-wrath")]
fn accept_quest(state: &mut LuaState) -> LuaResult<u32> {
    let index = {
        let mut sim = borrow_state_mut(state)?;
        if !super::group_queries::unit_exists_in_state(&sim, "player") {
            return Ok(0);
        }
        let Some(offer) = sim.pending_quest_offer.take() else {
            return Ok(0);
        };
        let id = offer.quest_id as u32;
        if sim.quest_log_entries.entries.len() >= 25 {
            None
        } else {
            append_quest_log_entry(&mut sim.quest_log, id);
            sim.quest_log_entries.entries.push(offer);
            Some(sim.quest_log_entries.entries.len())
        }
    };
    publish_accept_response(state, index)?;
    finish_wrath_quest_dialog(state)?;
    Ok(0)
}

#[cfg(feature = "client-wrath")]
pub(super) fn finish_wrath_quest_dialog(state: &mut LuaState) -> LuaResult<()> {
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.gossip.giver_guid == 0 {
            return Ok(());
        }
        // Authenticated 12340 order: NPC/inform, interaction, synchronous event.
        sim.gossip.giver_guid = 0;
        sim.gossip.inform_guid = 0;
        sim.gossip.interact_guid = 0;
    }
    super::state_backed_queries::dispatch_event_now(state, "QUEST_FINISHED", &[])
}

#[cfg(feature = "client-wrath")]
fn close_wrath_quest(state: &mut LuaState) -> LuaResult<u32> {
    // CloseQuest retains current quest/phase, sent latch and pending requests.
    finish_wrath_quest_dialog(state)?;
    Ok(0)
}

#[cfg(feature = "client-wrath")]
fn publish_accept_response(state: &mut LuaState, index: Option<usize>) -> LuaResult<()> {
    let Some(index) = index else {
        let text = crate::lua_api::methods::create_string_static(state, "Your quest log is full.");
        return super::state_backed_queries::dispatch_event_now(state, "UI_ERROR_MESSAGE", &[text]);
    };
    // Simulator server-response policy; not a claim that AcceptQuest inserts locally.
    super::state_backed_queries::dispatch_event_now(
        state,
        "QUEST_ACCEPTED",
        &[Val::Num(index as f64)],
    )?;
    notify_wrath_log_changed(state)
}

#[cfg(feature = "client-wrath")]
pub(super) fn notify_wrath_log_changed(state: &mut LuaState) -> LuaResult<()> {
    let player = crate::lua_api::methods::create_string_static(state, "player");
    super::state_backed_queries::dispatch_event_now(state, "UNIT_QUEST_LOG_CHANGED", &[player])?;
    let mut sim = borrow_state_mut(state)?;
    if !sim
        .events
        .pending()
        .iter()
        .any(|event| event.name == "QUEST_LOG_UPDATE")
    {
        sim.events.push_simple("QUEST_LOG_UPDATE");
    }
    Ok(())
}

/// `CloseQuestFrame()` — fires `QUEST_FINISHED`. WoW also nils the pending
/// offer; mirror that here.
fn close_quest_frame(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.pending_quest_offer = None;
        sim.quest_log_entries.readiness.dialog = None;
    }
    push_event(state, "QUEST_FINISHED")?;
    Ok(0)
}

/// `AcknowledgeAutoQuestPopUp(id)` — silent dismiss. The id is accepted
/// but not recorded; retail just clears the popup.
fn acknowledge_auto_quest_popup(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_u32(state, 1);
    Ok(0)
}

/// `QuestChoiceFrame_SetActiveChoice(id)` — write `quest_choice_id`.
fn quest_choice_set_active(state: &mut LuaState) -> LuaResult<u32> {
    let id = stack_u32(state, 1);
    borrow_state_mut(state)?.quest_choice_id = id;
    Ok(0)
}

/// `QuestMapLogTitleButton_OnClick(questId)` — write `selected_quest_log_id`.
fn quest_map_log_title_button_on_click(state: &mut LuaState) -> LuaResult<u32> {
    let id = stack_u32(state, 1);
    let mut sim = borrow_state_mut(state)?;
    #[cfg(not(feature = "client-wrath"))]
    if sim.selected_quest_log_id != id {
        sim.quest_log_entries.readiness.dialog = None;
    }
    sim.selected_quest_log_id = id;
    Ok(0)
}

/// `SetAbandonQuest()` — snapshot the currently selected quest as the
/// abandon target and fire `QUEST_REMOVED` to signal intent. The sim does
/// not run a two-step confirm (`AbandonQuest` in retail); the mark is the
/// full commitment.
#[cfg(not(feature = "client-wrath"))]
fn set_abandon_quest(state: &mut LuaState) -> LuaResult<u32> {
    let (removed, marked_id) = {
        let mut st = borrow_state_mut(state)?;
        let Some(id) = st.selected_quest_log_id else {
            return Ok(0);
        };
        let before = st.quest_log.len();
        st.quest_log.retain(|q| *q != id);
        let removed = st.quest_log.len() != before;
        st.quest_log_entries
            .entries
            .retain(|entry| entry.quest_id != id as i32);
        st.quest_log_entries.objective_progress.remove(&(id as i32));
        st.quest_log_entries
            .background_materials
            .remove(&(id as i32));
        st.quest_watch_types.remove(&(id as i32));
        st.quest_log_entries.readiness.remove_quest(id as i32);
        st.abandon_quest_id = Some(id);
        if st
            .pending_quest_offer
            .as_ref()
            .map(|offer| offer.quest_id as u32)
            == Some(id)
        {
            st.pending_quest_offer = None;
        }
        (removed, id)
    };
    if removed {
        push_event(state, "QUEST_REMOVED")?;
    }
    let _ = marked_id; // kept for future admin hooks
    Ok(0)
}

/// Wrath selection marks an active identity; it does not commit abandonment.
#[cfg(feature = "client-wrath")]
fn set_abandon_quest(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.abandon_quest_id = sim.selected_quest_log_id.filter(|id| {
        sim.quest_log_entries
            .entries
            .iter()
            .any(|row| row.quest_id as u32 == *id)
    });
    Ok(0)
}

/// Model the server removal response, retaining the client's marked identity.
#[cfg(feature = "client-wrath")]
fn abandon_quest(state: &mut LuaState) -> LuaResult<u32> {
    let removed = abandon_marked_quest(&mut *borrow_state_mut(state)?);
    if !removed {
        return Ok(0);
    }
    notify_wrath_log_changed(state)?;
    Ok(0)
}

#[cfg(feature = "client-wrath")]
fn abandon_marked_quest(sim: &mut crate::lua_api::SimState) -> bool {
    let Some(id) = sim.abandon_quest_id else {
        return false;
    };
    if !sim.quest_log_entries.abandon(id as i32) {
        return false;
    }
    sim.quest_log.retain(|quest| *quest != id);
    if sim.selected_quest_log_id == Some(id) {
        sim.selected_quest_log_id = None;
    }
    if sim
        .pending_quest_offer
        .as_ref()
        .map(|offer| offer.quest_id as u32)
        == Some(id)
    {
        sim.pending_quest_offer = None;
    }
    true
}

/// `SetTrackedAchievement(id, tracked)` — add or remove by id.
fn set_tracked_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let tracked = stack_bool(state, 2).unwrap_or(true);
    let mut st = borrow_state_mut(state)?;
    if tracked {
        if !st.tracked_achievements.contains(&id) {
            st.tracked_achievements.push(id);
        }
    } else {
        st.tracked_achievements
            .retain(|tracked_id| *tracked_id != id);
    }
    Ok(0)
}

/// `UntrackAchievement(id)` — remove from `tracked_achievements`.
fn untrack_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_i32(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?
        .tracked_achievements
        .retain(|tracked_id| *tracked_id != id);
    Ok(0)
}

fn stack_string(state: &mut LuaState, index: i32) -> String {
    let Val::Str(s) = stack_val(state, index) else {
        return String::new();
    };
    state
        .gc
        .string_arena
        .get(s)
        .and_then(|lua_str| std::str::from_utf8(lua_str.data()).ok())
        .map(str::to_owned)
        .unwrap_or_default()
}

fn stack_f64(state: &mut LuaState, index: i32) -> f64 {
    match stack_val(state, index) {
        Val::Num(n) => n,
        _ => 0.0,
    }
}

/// `QuestFrame_ShowQuestPortrait(parent, portraitDisplayID,
/// mountPortraitDisplayID, modelSceneID, text, name, x, y, hideModel)`
/// — records the request on `SimState.quest_portrait_state` and
/// reparents the global `QuestModelScene` frame to `parent` so the
/// dialog visually owns the portrait. Tests probe state directly; the
/// 3D model itself is intentionally not rendered.
fn quest_frame_show_quest_portrait(state: &mut LuaState) -> LuaResult<u32> {
    let parent_val = stack_val(state, 1);
    let parent_id = extract_frame_id(state, parent_val);
    let portrait_display_id = stack_i32(state, 2).unwrap_or(0);
    let mount_portrait_display_id = stack_i32(state, 3).unwrap_or(0);
    let model_scene_id = stack_i32(state, 4).unwrap_or(0);
    let text = stack_string(state, 5);
    let name = stack_string(state, 6);
    let x = stack_f64(state, 7);
    let y = stack_f64(state, 8);
    let hide_model = stack_bool(state, 9).unwrap_or(false);

    let mut sim = borrow_state_mut(state)?;
    let model_scene_id_u64 = sim.widgets.get_id_by_name("QuestModelScene");
    sim.quest_portrait_state = Some(QuestPortraitState {
        parent_frame_id: parent_id,
        portrait_display_id,
        mount_portrait_display_id,
        model_scene_id,
        text,
        name,
        x,
        y,
        hide_model,
    });
    if let Some(scene_id) = model_scene_id_u64 {
        reparent_widget(&mut sim.widgets, scene_id, parent_id);
    }
    Ok(0)
}

/// `QuestFrame_HideQuestPortrait()` — clears
/// `SimState.quest_portrait_state` and detaches the global
/// `QuestModelScene` from its parent (matching Blizzard's
/// `SetParent(nil)` step).
fn quest_frame_hide_quest_portrait(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.quest_portrait_state = None;
    if let Some(scene_id) = sim.widgets.get_id_by_name("QuestModelScene") {
        reparent_widget(&mut sim.widgets, scene_id, None);
    }
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_quest_state_verbs(lua)?;
    register_quest_portrait_verbs(lua)?;
    Ok(())
}

fn register_quest_state_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "client-wrath")]
    LuaApiMut::register_function(lua, "AcceptQuest", accept_quest)?;
    LuaApiMut::register_function(lua, "ConfirmAcceptQuest", confirm_accept_quest)?;
    LuaApiMut::register_function(lua, "CloseQuestFrame", close_quest_frame)?;
    #[cfg(feature = "client-wrath")]
    LuaApiMut::register_function(lua, "CloseQuest", close_wrath_quest)?;
    if crate::c_api::quest_objectives::ENABLED {
        LuaApiMut::register_function(lua, "CloseQuest", close_quest_frame)?;
    }
    LuaApiMut::register_function(
        lua,
        "AcknowledgeAutoQuestPopUp",
        acknowledge_auto_quest_popup,
    )?;
    LuaApiMut::register_function(
        lua,
        "QuestChoiceFrame_SetActiveChoice",
        quest_choice_set_active,
    )?;
    LuaApiMut::register_function(
        lua,
        "QuestMapLogTitleButton_OnClick",
        quest_map_log_title_button_on_click,
    )?;
    LuaApiMut::register_function(lua, "SetAbandonQuest", set_abandon_quest)?;
    #[cfg(feature = "client-wrath")]
    LuaApiMut::register_function(lua, "AbandonQuest", abandon_quest)?;
    LuaApiMut::register_function(lua, "SetTrackedAchievement", set_tracked_achievement)?;
    LuaApiMut::register_function(lua, "UntrackAchievement", untrack_achievement)?;
    Ok(())
}

fn register_quest_portrait_verbs(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(
        lua,
        "QuestFrame_ShowQuestPortrait",
        quest_frame_show_quest_portrait,
    )?;
    LuaApiMut::register_function(
        lua,
        "QuestFrame_HideQuestPortrait",
        quest_frame_hide_quest_portrait,
    )?;
    Ok(())
}
