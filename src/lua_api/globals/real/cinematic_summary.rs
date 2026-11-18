//! VERIFIED: pr46-49-decomp-answers.md R88ap part A §1 (Retail 12.1.5).
//! UNKNOWN: other start/finish effects and emitted events; none are invented.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::table_set_rust_fn_static;
mod inputs;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "CinematicStarted", started)?;
    table_set_rust_fn_static(state, state.global, "CinematicFinished", finished)?;
    table_set_rust_fn_static(state, state.global, "GetCurrentCinematicSummary", summary)
}

fn started(state: &mut LuaState) -> LuaResult<u32> {
    if let Some((kind, id)) = inputs::start_input(state) {
        if [0.0, 1.0, 2.0, 3.0].contains(&kind) {
            borrow_state_mut(state)?.current_cinematic_id = id;
        }
    }
    Ok(0)
}

fn finished(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.current_cinematic_id = 0;
    Ok(0)
}

fn summary(state: &mut LuaState) -> LuaResult<u32> {
    let text = {
        let sim = borrow_state(state)?;
        (sim.current_cinematic_id != 0)
            .then(|| sim.cinematic_summaries.get(&sim.current_cinematic_id))
            .flatten()
            .and_then(|text| text.as_ref())
            .filter(|text| !text.is_empty())
            .cloned()
    };
    let value = match text {
        Some(text) => Val::Str(state.gc.intern_string(text.as_bytes())),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}
