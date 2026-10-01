//! Ordinary housing basic-mode state. Placement and collision effects are unmodeled.

use super::catalog::{read_selector, read_variant_id};
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
#[cfg(feature = "retail-12-0-0")]
use crate::lua_bridge::stack_val;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Replaces three unconditional Lua publishers without gating other basic-mode APIs.
pub(super) fn register_pending(state: &mut LuaState) -> LuaResult<()> {
    let basic_mode = ensure_namespace(state, "C_HousingBasicMode")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("StartPlacingNewDecor", start_placing_new_decor),
        ("IsPlacingNewDecor", is_placing_new_decor),
        ("CancelActiveEditing", cancel_active_editing),
    ];
    for &(name, function) in functions {
        table_set_rust_fn_static(state, basic_mode, name, function)?;
    }
    Ok(())
}

fn start_placing_new_decor(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let id = read_variant_id(state, selector)?;
    let mut sim = borrow_state_mut(state)?;
    // Inferred eligibility only: no reservation, instance, selection or event.
    if sim
        .housing
        .catalog
        .variants
        .get(&id)
        .is_some_and(|record| record.num_stored > 0)
    {
        sim.housing.pending_new_decor = Some(id);
    }
    Ok(0)
}

fn is_placing_new_decor(state: &mut LuaState) -> LuaResult<u32> {
    let pending = borrow_state(state)?.housing.pending_new_decor.is_some();
    state.push(Val::Bool(pending));
    Ok(1)
}

fn cancel_active_editing(state: &mut LuaState) -> LuaResult<u32> {
    // Bounded cancellation inference; placed/customize/preview state stays untouched.
    borrow_state_mut(state)?.housing.pending_new_decor = None;
    Ok(0)
}

#[cfg(feature = "retail-12-0-0")]
pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let basic_mode = ensure_namespace(state, "C_HousingBasicMode")?;
    table_set_rust_fn_static(
        state,
        basic_mode,
        "IsFreePlaceEnabled",
        is_free_place_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        basic_mode,
        "SetFreePlaceEnabled",
        set_free_place_enabled,
    )
}

#[cfg(feature = "retail-12-0-0")]
fn is_free_place_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = !borrow_state(state)?.housing.free_place_disabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
fn set_free_place_enabled(state: &mut LuaState) -> LuaResult<u32> {
    // Simulator validation policy; native coercion and exact errors are unverified.
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "SetFreePlaceEnabled requires a boolean",
        ));
    };
    borrow_state_mut(state)?.housing.free_place_disabled = !enabled;
    Ok(0)
}
