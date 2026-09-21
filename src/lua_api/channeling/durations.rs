//! Duration snapshots from the modeled player cast/channel timeline.
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

#[derive(Clone, Copy)]
enum DurationKind {
    Cast,
    Channel,
    Empower { include_hold: bool },
}

pub(super) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "UnitCastingDuration", casting)?;
    LuaApiMut::register_function(lua, "UnitChannelDuration", channel)?;
    LuaApiMut::register_function(lua, "UnitEmpoweredChannelDuration", empowered)
}

fn casting(state: &mut LuaState) -> LuaResult<u32> {
    push_duration(state, DurationKind::Cast)
}

fn channel(state: &mut LuaState) -> LuaResult<u32> {
    push_duration(state, DurationKind::Channel)
}

fn empowered(state: &mut LuaState) -> LuaResult<u32> {
    let include_hold = Option::<bool>::from_stack(state, 2)?.unwrap_or(true);
    push_duration(state, DurationKind::Empower { include_hold })
}

fn push_duration(state: &mut LuaState, kind: DurationKind) -> LuaResult<u32> {
    if String::from_stack(state, 1)? != "player" {
        return Ok(0);
    }
    let timing = {
        let sim = borrow_state(state)?;
        let cast = match kind {
            DurationKind::Cast => sim.casting.as_ref(),
            DurationKind::Channel => sim.channeling.as_ref(),
            DurationKind::Empower { .. } => sim.channeling.as_ref().filter(|c| c.empower.is_some()),
        };
        cast.map(|cast| {
            let end = match kind {
                DurationKind::Empower { include_hold: true } => cast.completion_time(),
                _ => cast.end_time,
            };
            (cast.start_time, end - cast.start_time)
        })
    };
    let Some((start, seconds)) = timing else {
        return Ok(0);
    };
    push_timed_duration_object(state, start, seconds)
}
