//! Alpha gradient methods.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "ClearAlphaGradient", clear_alpha_gradient)?;
    table_set_rust_fn_static(state, mt, "HasAlphaGradient", has_alpha_gradient)?;
    table_set_rust_fn_static(state, mt, "SetAlphaGradient", set_alpha_gradient)?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(state, mt, "GetAlphaGradient", get_alpha_gradient)?;
    Ok(())
}

pub fn clear_alpha_gradient(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.alpha_gradients.clear();
    }
    Ok(0)
}

pub fn has_alpha_gradient(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = borrow_state(state)?
        .widgets
        .get(id)
        .map(|f| !f.alpha_gradients.is_empty())
        .unwrap_or(false);
    state.push(Val::Bool(val));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn get_alpha_gradient(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let gradient = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.alpha_gradients.get(&0).copied());
    let (start, length) = gradient
        .map(|gradient| (gradient.start, gradient.length))
        .unwrap_or((0.0, 0.0));
    state.push(Val::Num(start as f64));
    state.push(Val::Num(length as f64));
    Ok(2)
}

pub fn set_alpha_gradient(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let start = read_gradient_start(state);
    let length = read_gradient_length(state);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.alpha_gradients.insert(
            0,
            crate::widget::AlphaGradient {
                start,
                length: length.max(0.0),
            },
        );
    }
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn read_gradient_start(state: &mut LuaState) -> f32 {
    match stack_val(state, 2) {
        Val::Num(value) => value as f32,
        _ => 0.0,
    }
}

fn read_gradient_length(state: &mut LuaState) -> f32 {
    match stack_val(state, 3) {
        Val::Num(value) => value as f32,
        Val::Table(point) => match table_get(state, Val::Table(point), "y") {
            Val::Num(value) => value as f32,
            _ => 1.0,
        },
        _ => 1.0,
    }
}
