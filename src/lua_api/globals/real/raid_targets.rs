//! Remove every assigned GUID marker using the same state as SetRaidTarget.
//! Cached RaidMarkersDocumentation declares no arguments/results and restricted access.
//! Notification timing follows the simulator's existing SetRaidTarget policy;
//! native protected-action authorization remains unmodeled.

use crate::event::Event;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

fn remove_raid_targets(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.unit_raid_target_icons.clear();
        sim.events.push(Event {
            name: "RAID_TARGET_UPDATE".to_string(),
            args: Vec::new(),
        });
    }
    dispatch_event_now(state, "RAID_TARGET_UPDATE", &[])?;
    Ok(0)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "RemoveRaidTargets", remove_raid_targets)
}
