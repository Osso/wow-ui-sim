//! Legacy tutorial request recording, without tutorial event/flag/display policy.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::WowForever
            | ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
            | ClientProfile::Wrath
    ) {
        LuaApiMut::register_function(lua, "TriggerTutorial", record_tutorial_request)?;
    }
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        LuaApiMut::register_function(
            lua,
            "GetPrevCompleatedTutorial",
            get_prev_completed_tutorial,
        )?;
        LuaApiMut::register_function(
            lua,
            "GetNextCompleatedTutorial",
            get_next_completed_tutorial,
        )?;
    }
    Ok(())
}

// INFERRED, R88fh (native evidence pending): nearest flagged tutorial ID below id.
fn get_prev_completed_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    publish_neighbor(state, true)
}

// INFERRED, R88fh (native evidence pending): nearest flagged tutorial ID above id.
fn get_next_completed_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    publish_neighbor(state, false)
}

fn publish_neighbor(state: &mut LuaState, previous: bool) -> LuaResult<u32> {
    let tutorial_id = i32::from_stack(state, 1)?;
    let value = {
        let flags = &borrow_state(state)?.tutorial_flags;
        let neighbor = flags.iter().copied().map(|id| id as i32).filter(|id| {
            if previous {
                *id < tutorial_id
            } else {
                *id > tutorial_id
            }
        });
        if previous {
            neighbor.max()
        } else {
            neighbor.min()
        }
    };

    match value {
        Some(value) => state.push(rilua::Val::Num(value as f64)),
        None => state.push(rilua::Val::Nil),
    }
    Ok(1)
}

fn record_tutorial_request(state: &mut LuaState) -> LuaResult<u32> {
    let tutorial_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?.tutorial_requests.push(tutorial_id);
    Ok(0)
}
