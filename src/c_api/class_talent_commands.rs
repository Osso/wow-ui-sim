//! Retail ClassTalentHelper commands delegate to the real Blizzard UI callbacks.
//!
//! The simulator supplies a secure native-event boundary, not a replacement for
//! the vendor's specialization/loadout selection or its UI mutation policy.

use crate::loader::stack_taint::with_secure_stack;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::create_string;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, function) in [
        (
            "SwitchToLoadoutByName",
            switch_loadout_by_name as rilua::vm::closure::RustFn,
        ),
        ("SwitchToLoadoutByIndex", switch_loadout_by_index),
        (
            "SwitchToSpecializationByName",
            switch_specialization_by_name,
        ),
        (
            "SwitchToSpecializationByIndex",
            switch_specialization_by_index,
        ),
    ] {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn switch_loadout_by_name(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_name_command(state, "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_NAME")
}

fn switch_loadout_by_index(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_index_command(state, "CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX")
}

fn switch_specialization_by_name(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_name_command(state, "CLASS_TALENTS_SWITCH_TO_SPECIALIZATION_BY_NAME")
}

fn switch_specialization_by_index(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_index_command(state, "CLASS_TALENTS_SWITCH_TO_SPECIALIZATION_BY_INDEX")
}

fn dispatch_name_command(state: &mut LuaState, event: &str) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    // Recreate the payload at the native boundary. Do not pass an addon table,
    // callback or secret wrapper into a privileged delegate.
    with_secure_stack(state, |state| {
        let payload = create_string(state, &name);
        dispatch_event_now(state, event, &[payload])
    })?;
    Ok(0)
}

fn dispatch_index_command(state: &mut LuaState, event: &str) -> LuaResult<u32> {
    // INFERRED: retain the existing signed-i32 input representation. Selection
    // validation belongs to the vendor callback; never clamp zero to spec 1.
    let index = i32::from_stack(state, 1)?;
    let payload = Val::Num(f64::from(index));
    // INFERRED: synchronous command-event delivery using the existing dispatcher.
    // The secure stack is restored on both success and a Lua callback error;
    // registered addon closures still enter with their own closure taint.
    with_secure_stack(state, |state| dispatch_event_now(state, event, &[payload]))?;
    Ok(0)
}
