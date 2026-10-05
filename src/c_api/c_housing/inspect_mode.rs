//! `C_HousingInspectMode`: decor inspection mode and the hovered decor instance.
//!
//! Hover targets come from host state (world picking is unmodeled). Mode changes
//! fire `HOUSING_INSPECT_MODE_STATE_UPDATED`; INFERRED: only real transitions fire.

use crate::c_api::helpers::ensure_namespace;
use crate::event::Event;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct InspectModeState {
    pub active: bool,
    /// Decor GUID under the cursor; only reported while inspect mode is active.
    pub hovered_decor_guid: Option<String>,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_HousingInspectMode")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("EnterInspectMode", enter_inspect_mode),
        ("ExitInspectMode", exit_inspect_mode),
        ("IsInInspectMode", is_in_inspect_mode),
        ("IsHoveringDecor", is_hovering_decor),
        ("GetHoveredDecorGUID", get_hovered_decor_guid),
    ];
    for &(name, function) in functions {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn set_active(state: &mut LuaState, active: bool) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    if sim.housing.inspect_mode.active != active {
        sim.housing.inspect_mode.active = active;
        sim.events.push(Event {
            name: "HOUSING_INSPECT_MODE_STATE_UPDATED".into(),
            args: Vec::new(),
        });
    }
    Ok(0)
}

fn enter_inspect_mode(state: &mut LuaState) -> LuaResult<u32> {
    set_active(state, true)
}

fn exit_inspect_mode(state: &mut LuaState) -> LuaResult<u32> {
    set_active(state, false)
}

fn is_in_inspect_mode(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.housing.inspect_mode.active;
    state.push(Val::Bool(active));
    Ok(1)
}

fn hovered_decor_guid(state: &LuaState) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let mode = &sim.housing.inspect_mode;
    Ok(mode
        .active
        .then(|| mode.hovered_decor_guid.clone())
        .flatten())
}

fn is_hovering_decor(state: &mut LuaState) -> LuaResult<u32> {
    let hovering = hovered_decor_guid(state)?.is_some();
    state.push(Val::Bool(hovering));
    Ok(1)
}

/// INFERRED: an empty GUID when nothing is hovered (the result is non-nilable).
fn get_hovered_decor_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = hovered_decor_guid(state)?.unwrap_or_default();
    let guid = create_string(state, &guid);
    state.push(guid);
    Ok(1)
}
