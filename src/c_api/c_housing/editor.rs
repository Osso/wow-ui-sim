//! Host availability and deferred mode transitions; no simulated server approval.

use crate::c_api::helpers::ensure_namespace;
use crate::event::{Event, EventArg};
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const MODE_NONE: i32 = 0;
const MODE_BASIC: i32 = 1;
const LAST_MODE: i32 = 6;
const SUCCESS: i32 = 0;
const GENERIC_FAILURE: i32 = 37;

#[derive(Clone, Debug)]
pub struct EditorState {
    pub status_available: bool,
    pub availability: i32,
    pub mode_availability: [i32; 7],
    pub default_mode: i32,
    pub pending_mode: Option<i32>,
}

impl Default for EditorState {
    fn default() -> Self {
        // INFERRED: unconfigured editor denies requests; no native default claim.
        Self {
            status_available: false,
            availability: GENERIC_FAILURE,
            mode_availability: [GENERIC_FAILURE; 7],
            default_mode: MODE_BASIC,
            pending_mode: None,
        }
    }
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_HouseEditor")?;
    for (name, function) in [
        ("ActivateHouseEditorMode", activate as rilua::RustFn),
        ("EnterHouseEditor", enter),
        ("GetHouseEditorAvailability", availability),
        ("GetHouseEditorModeAvailability", mode_availability),
        ("IsHouseEditorActive", is_active),
        ("IsHouseEditorModeActive", is_mode_active),
        ("IsHouseEditorStatusAvailable", status_available),
        ("LeaveHouseEditor", leave),
        ("GetActiveHouseEditorMode", active_mode),
        ("GetHouseEditorPlayerType", player_type),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    Ok(())
}

fn read_mode(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Num(value)
            if (f64::from(MODE_NONE)..=f64::from(LAST_MODE)).contains(&value)
                && value.fract() == 0.0 =>
        {
            Ok(value as i32)
        }
        _ => Err(rilua::runtime_error(
            "house editor mode must be an integer in 0..6",
        )),
    }
}

fn active_mode(state: &mut LuaState) -> LuaResult<u32> {
    let mode = borrow_state(state)?.housing.active_house_editor_mode;
    state.push(Val::Num(f64::from(mode)));
    Ok(1)
}

fn player_type(state: &mut LuaState) -> LuaResult<u32> {
    let player_type = borrow_state(state)?.housing.house_editor_player_type;
    state.push(player_type.map_or(Val::Nil, |value| Val::Num(f64::from(value))));
    Ok(1)
}

fn availability(state: &mut LuaState) -> LuaResult<u32> {
    let result = borrow_state(state)?.housing.editor.availability;
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn mode_availability(state: &mut LuaState) -> LuaResult<u32> {
    let mode = read_mode(state)?;
    let result = borrow_state(state)?.housing.editor.mode_availability[mode as usize];
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn status_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.housing.editor.status_available;
    state.push(Val::Bool(available));
    Ok(1)
}

fn is_active(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.housing.active_house_editor_mode != MODE_NONE;
    state.push(Val::Bool(active));
    Ok(1)
}

fn is_mode_active(state: &mut LuaState) -> LuaResult<u32> {
    let mode = read_mode(state)?;
    let active = borrow_state(state)?.housing.active_house_editor_mode == mode;
    state.push(Val::Bool(active));
    Ok(1)
}

fn activate(state: &mut LuaState) -> LuaResult<u32> {
    let mode = read_mode(state)?;
    push_request_result(state, mode)
}

fn enter(state: &mut LuaState) -> LuaResult<u32> {
    let mode = borrow_state(state)?.housing.editor.default_mode;
    if !(MODE_BASIC..=LAST_MODE).contains(&mode) {
        return Err(rilua::runtime_error(
            "host default house editor mode must be in 1..6",
        ));
    }
    push_request_result(state, mode)
}

fn push_request_result(state: &mut LuaState, mode: i32) -> LuaResult<u32> {
    let result = request_mode(&mut *borrow_state_mut(state)?, mode);
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn request_mode(sim: &mut SimState, mode: i32) -> i32 {
    let housing = &mut sim.housing;
    let editor = &mut housing.editor;
    if !editor.status_available {
        return GENERIC_FAILURE;
    }
    if editor.availability != SUCCESS {
        return editor.availability;
    }
    let result = editor.mode_availability[mode as usize];
    if result == SUCCESS && mode != housing.active_house_editor_mode {
        // Documentation: success starts a request, not the final server transition.
        editor.pending_mode = Some(mode);
    }
    result
}

/// Host delivery boundary. Success commits the pending mode; failure preserves active mode.
pub fn complete_mode_change(sim: &mut SimState, result: i32) {
    if let Some((name, payload)) = resolve_mode_change(sim, result) {
        sim.events.push(Event {
            name: name.into(),
            args: vec![EventArg::Number(f64::from(payload))],
        });
    }
}

fn resolve_mode_change(sim: &mut SimState, result: i32) -> Option<(&'static str, i32)> {
    let mode = sim.housing.editor.pending_mode.take()?;
    Some(if result == SUCCESS {
        sim.housing.active_house_editor_mode = mode;
        ("HOUSE_EDITOR_MODE_CHANGED", mode)
    } else {
        ("HOUSE_EDITOR_MODE_CHANGE_FAILURE", result)
    })
}

fn leave(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.housing.editor.pending_mode = None;
    if sim.housing.active_house_editor_mode != MODE_NONE {
        // INFERRED: leaving is a local transition; never requires a server acknowledgment.
        sim.housing.active_house_editor_mode = MODE_NONE;
        sim.events.push(Event {
            name: "HOUSE_EDITOR_MODE_CHANGED".into(),
            args: vec![EventArg::Number(f64::from(MODE_NONE))],
        });
    }
    Ok(0)
}

/// `A_Admin.SetHouseEditorStatus(statusAvailable, availability, defaultMode[, playerType])`
/// supplies the editor gate, `A_Admin.SetHouseEditorModeAvailability(mode, result)`
/// one mode's `Enum.HousingResult`, and `A_Admin.CompleteHouseEditorModeChange(result)`
/// the server reply to a pending request (the mode events dispatch at once).
/// Simulator inputs, not native housing permissions.
pub(crate) fn register_admin(
    builder: crate::lua_bridge::TableBuilder,
) -> LuaResult<crate::lua_bridge::TableBuilder> {
    builder
        .set_function("SetHouseEditorStatus", admin_set_status)?
        .set_function(
            "SetHouseEditorModeAvailability",
            admin_set_mode_availability,
        )?
        .set_function("CompleteHouseEditorModeChange", admin_complete_mode_change)
}

fn admin_set_status(state: &mut LuaState) -> LuaResult<u32> {
    let status_available = bool::from_stack(state, 1)?;
    let availability = i32::from_stack(state, 2)?;
    let default_mode = i32::from_stack(state, 3)?;
    let player_type = Option::<i32>::from_stack(state, 4)?;
    if !(MODE_BASIC..=LAST_MODE).contains(&default_mode) {
        return Err(rilua::runtime_error(
            "A_Admin.SetHouseEditorStatus: defaultMode must be in 1..6",
        ));
    }
    let mut sim = borrow_state_mut(state)?;
    let editor = &mut sim.housing.editor;
    editor.status_available = status_available;
    editor.availability = availability;
    editor.default_mode = default_mode;
    sim.housing.house_editor_player_type = player_type;
    Ok(0)
}

fn admin_set_mode_availability(state: &mut LuaState) -> LuaResult<u32> {
    let mode = read_mode(state)?;
    let result = i32::from_stack(state, 2)?;
    borrow_state_mut(state)?.housing.editor.mode_availability[mode as usize] = result;
    Ok(0)
}

fn admin_complete_mode_change(state: &mut LuaState) -> LuaResult<u32> {
    let result = i32::from_stack(state, 1)?;
    let event = resolve_mode_change(&mut *borrow_state_mut(state)?, result);
    if let Some((name, payload)) = event {
        let args = [Val::Num(f64::from(payload))];
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(state, name, &args)?;
    }
    Ok(0)
}
