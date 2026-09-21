//! Secret, protected, and anchoring restriction methods.
//!
//! Explicit masks track declarations; they do not implement per-aspect secret
//! return tagging or context-dependent enforcement.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
#[cfg(feature = "forbidden-aspects")]
use rilua::runtime_error;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const OBJECT_SECRET_ASPECT: u32 = 1;

pub fn register(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    #[cfg(feature = "forbidden-aspects")]
    table_set_rust_fn_static(state, mt, "AddSecretAspect", add_secret_aspect)?;
    table_set_rust_fn_static(state, mt, "HasAnySecretAspect", has_any_secret_aspect)?;
    table_set_rust_fn_static(state, mt, "HasSecretAspect", has_secret_aspect)?;
    table_set_rust_fn_static(state, mt, "HasSecretValues", has_secret_values)?;
    table_set_rust_fn_static(state, mt, "IsAnchoringRestricted", is_anchoring_restricted)?;
    table_set_rust_fn_static(state, mt, "IsAnchoringSecret", is_anchoring_secret)?;
    table_set_rust_fn_static(
        state,
        mt,
        "IsPreventingSecretValues",
        is_preventing_secret_values,
    )?;
    table_set_rust_fn_static(state, mt, "IsProtected", is_protected)?;
    table_set_rust_fn_static(
        state,
        mt,
        "SetPreventSecretValues",
        set_prevent_secret_values,
    )?;
    Ok(())
}

#[cfg(feature = "forbidden-aspects")]
pub fn add_secret_aspect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let aspect = u32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(id)
        .ok_or_else(|| runtime_error("invalid frame"))?;
    frame.explicit_secret_aspects |= aspect;
    Ok(0)
}

pub fn has_any_secret_aspect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = frame_secret_aspects(&borrow_state(state)?.widgets, id) != 0;
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn has_secret_aspect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let aspect = match stack_val(state, 2) {
        Val::Num(n) => u32::try_from(n as i64).ok(),
        _ => None,
    };
    let mask = frame_secret_aspects(&borrow_state(state)?.widgets, id);
    let result = aspect.is_some_and(|aspect| mask & aspect != 0);
    state.push(Val::Bool(result));
    Ok(1)
}

pub fn has_secret_values(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = borrow_state(state)?
        .widgets
        .get(id)
        .is_some_and(|frame| frame.prevent_secret_values || frame.explicit_secret_aspects != 0);
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn is_anchoring_restricted(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = { frame_is_anchoring_restricted(&borrow_state(state)?.widgets, id) };
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn is_anchoring_secret(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = { frame_has_secret_values(&borrow_state(state)?.widgets, id) };
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn is_preventing_secret_values(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let val = {
        borrow_state(state)?
            .widgets
            .get(id)
            .map(|f| f.prevent_secret_values)
            .unwrap_or(false)
    };
    state.push(Val::Bool(val));
    Ok(1)
}

pub fn is_protected(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let protected = {
        borrow_state(state)?
            .widgets
            .get(id)
            .map(|f| f.is_protected)
            .unwrap_or(false)
    };
    state.push(Val::Bool(protected));
    state.push(Val::Bool(protected));
    Ok(2)
}

pub fn set_prevent_secret_values(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let prevent = bool::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut(id) {
        frame.prevent_secret_values = prevent;
    }
    Ok(0)
}

fn frame_secret_aspects(widgets: &crate::widget::WidgetRegistry, id: u64) -> u32 {
    let Some(frame) = widgets.get(id) else {
        return 0;
    };
    let derived = if frame.prevent_secret_values || frame.forbidden || frame.is_protected {
        OBJECT_SECRET_ASPECT
    } else {
        0
    };
    frame.explicit_secret_aspects | derived
}

fn frame_has_secret_values(widgets: &crate::widget::WidgetRegistry, id: u64) -> bool {
    widgets
        .get(id)
        .map(|f| f.prevent_secret_values)
        .unwrap_or(false)
}

fn frame_is_anchoring_restricted(widgets: &crate::widget::WidgetRegistry, id: u64) -> bool {
    widgets
        .get(id)
        .map(|f| f.forbidden || f.is_protected)
        .unwrap_or(false)
}
