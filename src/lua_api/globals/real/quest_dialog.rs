//! Modeled non-C quest dialog/popup globals; reward claim intentionally absent.
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) use crate::c_api::quest_readiness::{complete_quest, is_quest_completable};

pub(crate) fn get_progress_money(state: &mut LuaState) -> LuaResult<u32> {
    let money = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .map(|requirements| requirements.money)
            .unwrap_or(0)
    };
    state.push(Val::Num(f64::from(money)));
    Ok(1)
}

pub(crate) fn get_progress_item_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .map(|requirements| requirements.items.len())
            .unwrap_or(0)
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(crate) fn get_progress_currency_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .map(|requirements| requirements.currencies.len())
            .unwrap_or(0)
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(crate) fn get_progress_text(state: &mut LuaState) -> LuaResult<u32> {
    let text = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .map(|requirements| requirements.text.clone())
            .unwrap_or_default()
    };
    let text = create_string(state, &text);
    state.push(text);
    Ok(1)
}

pub(crate) fn get_progress_portrait(state: &mut LuaState) -> LuaResult<u32> {
    let portrait = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::requirements_for_active_dialog(&sim)
            .map(|value| {
                (
                    value.portrait_display_id,
                    value.portrait_name.clone(),
                    value.portrait_text.clone(),
                )
            })
            .unwrap_or_default()
    };
    let text = create_string(state, &portrait.2);
    let name = create_string(state, &portrait.1);
    state.push(Val::Num(f64::from(portrait.0)));
    state.push(text);
    state.push(name);
    Ok(3)
}

pub(crate) fn add_auto_quest_popup(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let kind = String::from_stack(state, 2)?;
    let added = {
        let mut sim = borrow_state_mut(state)?;
        let active = sim
            .quest_log_entries
            .entries
            .iter()
            .any(|entry| entry.quest_id == id);
        let popups = &mut sim.quest_log_entries.readiness.popups;
        let valid_kind = kind == "OFFER" || kind == "COMPLETE";
        let add = active && valid_kind && !popups.iter().any(|(quest_id, _)| *quest_id == id);
        if add {
            popups.push((id, kind));
        }
        add
    };
    state.push(Val::Bool(added));
    Ok(1)
}

pub(crate) fn get_num_auto_quest_popups(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .quest_log_entries
        .readiness
        .popups
        .len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub(crate) fn get_auto_quest_popup(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let popup = {
        let sim = borrow_state(state)?;
        index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| sim.quest_log_entries.readiness.popups.get(index))
            .cloned()
    };
    let Some((id, kind)) = popup else {
        return Ok(0);
    };
    let kind = create_string(state, &kind);
    state.push(Val::Num(f64::from(id)));
    state.push(kind);
    Ok(2)
}

pub(crate) fn remove_auto_quest_popup(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .quest_log_entries
        .readiness
        .popups
        .retain(|(quest_id, _)| *quest_id != id);
    Ok(0)
}

pub(crate) fn set_quest_readiness(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let ready = bool::from_stack(state, 2)?;
    let auto_complete = bool::from_stack(state, 3)?;
    let notification = {
        let mut sim = borrow_state_mut(state)?;
        crate::c_api::quest_readiness::commit_supplied_readiness(&mut sim, id, ready, auto_complete)
    };
    if let Some(autocomplete) = notification {
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "QUEST_LOG_UPDATE",
            &[],
        )?;
        crate::c_api::quest_readiness::notify_readiness(state, id, autocomplete)?;
    }
    state.push(Val::Bool(notification.is_some()));
    Ok(1)
}
