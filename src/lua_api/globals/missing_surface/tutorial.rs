//! Minimal `C_Tutorial` surface with per-id flag storage.

use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register_tutorial_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_c_tutorial_table(state);
    table_set_rust_fn_static(
        state,
        table_ref,
        "AcknowledgeTutorial",
        acknowledge_tutorial,
    )?;
    table_set_rust_fn_static(state, table_ref, "HasSeenTutorial", has_seen_tutorial)?;
    table_set_rust_fn_static(state, table_ref, "GetTutorialStatus", get_tutorial_status)?;
    table_set_rust_fn_static(state, table_ref, "SetTutorialFlag", set_tutorial_flag)?;
    table_set_rust_fn_static(state, table_ref, "AbandonTutorialArea", noop)?;
    table_set_rust_fn_static(state, table_ref, "ReturnToTutorialArea", noop)?;
    table_set_rust_fn_static(state, table_ref, "GetCombatEventInfo", noop)?;
    Ok(())
}

fn ensure_c_tutorial_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_Tutorial");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|table| table.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }

    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), table, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table_ref
}

fn noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn acknowledge_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    let tutorial_id = u32::from_stack(state, 1)?;
    borrow_state_mut(state)?.tutorial_flags.insert(tutorial_id);
    Ok(0)
}

// INFERRED, R88fh (native evidence pending): use showTutorials for its stored setting.
pub(super) fn get_tutorials_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .cvars
        .get("showTutorials")
        .is_none_or(|value| value == "1");
    state.push(Val::Bool(enabled));
    Ok(1)
}

// R88fh VERIFIED: IDs 1–65 read one supplied bit; other IDs return no values.
pub(super) fn is_tutorial_flagged(state: &mut LuaState) -> LuaResult<u32> {
    let Some(tutorial_id) = tutorial_id(state)? else {
        return Ok(0);
    };
    let flagged = borrow_state(state)?.tutorial_flags.contains(&tutorial_id);
    state.push(Val::Bool(flagged));
    Ok(1)
}

// R88fh VERIFIED: an in-range ID sets its tutorial bit and returns no values.
pub(super) fn flag_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    if let Some(tutorial_id) = tutorial_id(state)? {
        borrow_state_mut(state)?.tutorial_flags.insert(tutorial_id);
    }
    Ok(0)
}

// R88fh VERIFIED (partial model): clear the tutorial-bit state; native also resets settings.
pub(super) fn reset_tutorials(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.tutorial_flags.clear();
    Ok(0)
}

fn tutorial_id(state: &mut LuaState) -> LuaResult<Option<u32>> {
    let Val::Num(value) = stack_val(state, 1) else {
        return Err(runtime_error("Usage: Trigger(\"tutorial\")"));
    };
    let value = value.trunc();
    if !(1.0..=65.0).contains(&value) {
        return Ok(None);
    }
    Ok(Some(value as u32))
}

pub(crate) fn set_tutorial_flag(state: &mut LuaState) -> LuaResult<u32> {
    let Some(tutorial_id) = tutorial_id(state)? else {
        return Ok(0);
    };
    let flagged = Option::<bool>::from_stack(state, 2)?.unwrap_or(true);
    let mut sim = borrow_state_mut(state)?;
    if flagged {
        sim.tutorial_flags.insert(tutorial_id);
    } else {
        sim.tutorial_flags.remove(&tutorial_id);
    }
    Ok(0)
}

fn has_seen_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    push_seen_status(state)
}

fn get_tutorial_status(state: &mut LuaState) -> LuaResult<u32> {
    push_seen_status(state)
}

fn push_seen_status(state: &mut LuaState) -> LuaResult<u32> {
    let tutorial_id = u32::from_stack(state, 1)?;
    let seen = {
        let sim = borrow_state(state)?;
        has_tutorial_flag(&sim, tutorial_id)
    };
    state.push(Val::Bool(seen));
    Ok(1)
}

fn has_tutorial_flag(sim: &SimState, tutorial_id: u32) -> bool {
    sim.tutorial_flags.contains(&tutorial_id)
}
