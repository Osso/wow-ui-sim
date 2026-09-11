//! Independent duration wrappers share an event clock; retained clocks freeze at termination.
use super::model::EventClock;
use crate::lua_api::globals::lua_duration_object::new_duration_object_value;
use crate::lua_api::methods::call_function_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{state::LuaState, value::Userdata};
use rilua::{LuaResult, Val, runtime_error};
use std::cell::RefCell;
use std::rc::Rc;

struct TimelineClock(Rc<RefCell<EventClock>>);

pub(super) fn create(state: &mut LuaState, clock: Rc<RefCell<EventClock>>) -> LuaResult<Val> {
    let duration = clock.borrow().duration;
    let metatable = rilua::stdlib::new_metatable(state, "EncounterTimelineClock")?;
    table_set_rust_fn_static(state, metatable, "__index", read_clock)?;
    let value = Userdata::with_metatable(Box::new(TimelineClock(clock)), metatable);
    let clock = Val::Userdata(state.gc.alloc_userdata(value));
    // Keep allocations rooted across normal Lua method dispatch and reentrant GC.
    state.push(clock);
    let timer = new_duration_object_value(state);
    state.push(timer);
    let result = initialize(state, timer, clock, duration);
    state.pop();
    state.pop();
    result.map(|()| timer)
}

fn initialize(state: &mut LuaState, timer: Val, clock: Val, duration: f64) -> LuaResult<()> {
    for (method, arguments) in [
        (
            "SetTimeFromStart",
            vec![timer, Val::Num(0.0), Val::Num(duration)],
        ),
        ("SetClock", vec![timer, clock]),
    ] {
        let key = state.gc.intern_string(method.as_bytes());
        let function = state.gettable(timer, Val::Str(key))?;
        call_function_state(state, function, &arguments)?;
    }
    Ok(())
}

fn read_clock(state: &mut LuaState) -> LuaResult<u32> {
    let key = state.gc.intern_string(b"time");
    if stack_val(state, 2) != Val::Str(key) {
        state.push(Val::Nil);
        return Ok(1);
    }
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error("timeline clock requires userdata"));
    };
    let elapsed = state
        .gc
        .userdata
        .get(reference)
        .and_then(|object| object.downcast_ref::<TimelineClock>())
        .ok_or_else(|| runtime_error("invalid timeline clock"))?
        .0
        .borrow()
        .elapsed();
    state.push(Val::Num(elapsed));
    Ok(1)
}
