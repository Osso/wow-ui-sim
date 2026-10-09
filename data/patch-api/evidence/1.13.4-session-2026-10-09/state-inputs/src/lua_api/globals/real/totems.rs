//! Host-supplied totem slots share one snapshot across info and duration queries.

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

#[derive(Debug, Clone)]
pub struct Totem {
    pub name: String,
    pub start_time: f64,
    pub duration: f64,
    pub icon: u32,
}

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetTotemInfo", get_totem_info)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        LuaApiMut::register_function(lua, "GetNumTotemSlots", get_num_totem_slots)?;
        LuaApiMut::register_function(lua, "GetTotemDuration", get_totem_duration)?;
    }
    Ok(())
}

fn read_totem(state: &LuaState) -> LuaResult<Option<Totem>> {
    let slot = u32::from_stack(state, 1)? as usize;
    let sim = borrow_state(state)?;
    let totem = slot
        .checked_sub(1)
        .and_then(|slot| sim.totem_slots.get(slot))
        .and_then(|slot| slot.as_ref());
    let now = sim.start_time.elapsed().as_secs_f64();
    Ok(totem
        .filter(|totem| totem.duration > 0.0 && now < totem.start_time + totem.duration)
        .cloned())
}

fn get_totem_info(state: &mut LuaState) -> LuaResult<u32> {
    match read_totem(state)? {
        Some(totem) => {
            state.push(Val::Bool(true));
            let name = create_string(state, &totem.name);
            state.push(name);
            state.push(Val::Num(totem.start_time));
            state.push(Val::Num(totem.duration));
            state.push(Val::Num(f64::from(totem.icon)));
        }
        None => {
            state.push(Val::Bool(false));
            state.push(Val::Nil);
            state.push(Val::Num(0.0));
            state.push(Val::Num(0.0));
            state.push(Val::Nil);
        }
    }
    Ok(5)
}

#[cfg(feature = "retail-12-0-5")]
fn get_num_totem_slots(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.totem_slots.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn get_totem_duration(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: inactive/out-of-range slots have an empty duration object.
    let (start, seconds) =
        read_totem(state)?.map_or((0.0, 0.0), |totem| (totem.start_time, totem.duration));
    crate::lua_api::globals::lua_duration_object::push_timed_duration_object(state, start, seconds)
}
