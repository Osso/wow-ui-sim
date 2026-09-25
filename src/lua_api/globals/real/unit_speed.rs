//! Player speed query backed by movement flags and configured capabilities.

use crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot;
use crate::lua_api::globals::unit_misc::guid_for_unit;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_unit_speed(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let speeds = resolve_unit_snapshot(&sim, &unit)
        .filter(|resolved| resolved.guid == guid_for_unit(&sim, "player"))
        .map(|_| {
            let movement = &sim.player.movement;
            let speeds = &sim.player.movement_speeds;
            let current = if !movement.moving {
                0.0
            } else if movement.swimming {
                speeds.swim
            } else if movement.flying {
                speeds.flight
            } else {
                speeds.run
            };
            (current, speeds.run, speeds.flight, speeds.swim)
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0));
    drop(sim);
    for speed in [speeds.0, speeds.1, speeds.2, speeds.3] {
        state.push(Val::Num(speed));
    }
    Ok(4)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetUnitSpeed", get_unit_speed)
}
