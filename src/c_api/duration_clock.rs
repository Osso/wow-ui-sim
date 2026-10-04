//! Host-owned manual duration clocks; numeric policies are documented as inferred.
use crate::lua_api::methods::{table_get, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table, value::Userdata};
use rilua::{LuaResult, Val, runtime_error};

const METATABLE: &str = "LuaDurationManualClock";

struct ManualClock {
    time: f64,
}

/// Existing optional initialTime remains available to Classic and earlier callers.
/// The later cached factory declaration does not document this extension.
pub(crate) fn create(state: &mut LuaState) -> LuaResult<u32> {
    let input = unwrap_secret(state, stack_val(state, 1))?;
    let time = match input {
        Val::Nil => 0.0,
        value => finite_number(value)?,
    };
    let metatable = ensure_metatable(state)?;
    let clock = Userdata::with_metatable(Box::new(ManualClock { time }), metatable);
    let reference = state.gc.alloc_userdata(clock);
    state.push(Val::Userdata(reference));
    Ok(1)
}

fn ensure_metatable(state: &mut LuaState) -> LuaResult<GcRef<Table>> {
    let metatable = rilua::stdlib::new_metatable(state, METATABLE)?;
    if table_get(state, Val::Table(metatable), "__index") == Val::Nil {
        table_set_rust_fn_static(state, metatable, "__index", read_property)?;
        table_set_static(
            state,
            Val::Table(metatable),
            "__metatable",
            Val::Bool(false),
        );
        install_methods(state, metatable)?;
    }
    Ok(metatable)
}

fn install_methods(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, method) in [
        ("GetTime", get_time as rilua::RustFn),
        ("SetTime", set_time),
        ("AdvanceTime", advance_time),
        ("RewindTime", rewind_time),
        ("ResetTime", reset_time),
    ] {
        table_set_rust_fn_static(state, metatable, name, method)?;
    }
    Ok(())
}

fn read_clock(state: &LuaState, value: Val) -> LuaResult<&ManualClock> {
    let Val::Userdata(reference) = value else {
        return Err(runtime_error("expected LuaDurationManualClock receiver"));
    };
    state
        .gc
        .userdata
        .get(reference)
        .and_then(|object| object.downcast_ref::<ManualClock>())
        .ok_or_else(|| runtime_error("incompatible LuaDurationManualClock receiver"))
}

fn write_time(state: &mut LuaState, value: Val, time: f64) -> LuaResult<()> {
    let Val::Userdata(reference) = value else {
        return Err(runtime_error("expected LuaDurationManualClock receiver"));
    };
    let clock = state
        .gc
        .userdata
        .get_mut(reference)
        .and_then(|object| object.downcast_mut::<ManualClock>())
        .ok_or_else(|| runtime_error("incompatible LuaDurationManualClock receiver"))?;
    clock.time = time;
    Ok(())
}

fn finite_number(value: Val) -> LuaResult<f64> {
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error("manual clock requires a finite number")),
    }
}

/// Read-only time is the existing duration-core/timeline clock protocol;
/// there is no mutable table storage or alternate manual-clock provider.
fn read_property(state: &mut LuaState) -> LuaResult<u32> {
    let receiver = stack_val(state, 1);
    let key = stack_val(state, 2);
    let time_key = Val::Str(state.gc.intern_string(b"time"));
    let value = if key == time_key {
        Val::Num(read_clock(state, receiver)?.time)
    } else {
        let metatable = rilua::stdlib::new_metatable(state, METATABLE)?;
        state.gettable(Val::Table(metatable), key)?
    };
    state.push(value);
    Ok(1)
}

fn get_time(state: &mut LuaState) -> LuaResult<u32> {
    let time = read_clock(state, stack_val(state, 1))?.time;
    state.push(Val::Num(time));
    Ok(1)
}

#[derive(Clone, Copy)]
enum Mutation {
    Set,
    Advance,
    Rewind,
}

pub(crate) fn authenticate_arguments(state: &LuaState) -> LuaResult<()> {
    for argument in &state.stack[state.base..state.top] {
        unwrap_secret(state, *argument)?;
    }
    Ok(())
}

fn mutate_time(state: &mut LuaState, mutation: Mutation) -> LuaResult<u32> {
    // Authenticate ignored extras too, before numeric or receiver validation.
    authenticate_arguments(state)?;
    let input = unwrap_secret(state, stack_val(state, 2))?;
    let receiver = unwrap_secret(state, stack_val(state, 1))?;
    let number = finite_number(input)?;
    let previous = read_clock(state, receiver)?.time;
    let time = match mutation {
        Mutation::Set => number,
        Mutation::Advance => previous + number,
        Mutation::Rewind => previous - number,
    };
    let time = finite_number(Val::Num(time))?;
    write_time(state, receiver, time)?;
    Ok(0)
}

fn set_time(state: &mut LuaState) -> LuaResult<u32> {
    mutate_time(state, Mutation::Set)
}

fn advance_time(state: &mut LuaState) -> LuaResult<u32> {
    mutate_time(state, Mutation::Advance)
}

fn rewind_time(state: &mut LuaState) -> LuaResult<u32> {
    mutate_time(state, Mutation::Rewind)
}

fn reset_time(state: &mut LuaState) -> LuaResult<u32> {
    write_time(state, stack_val(state, 1), 0.0)?;
    Ok(0)
}
