//! Solo GROUP_FORMED publication from explicit host inputs on the shared tick.
//! INFERRED: identity changes form a new group; an observed exit clears the latch.
//! No synthesized party GUID, implicit join, or read-only query side effects.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn tick(state: &mut LuaState) -> LuaResult<()> {
    let formation = {
        let mut sim = borrow_state_mut(state)?;
        let entry = current_formation(&sim);
        if sim.solo_group_last == entry {
            return Ok(());
        }
        sim.solo_group_last = entry.clone();
        entry
    };
    if let Some((category, guid)) = formation {
        // INFERRED: host event payload is public; no native restriction producer.
        let guid = create_string(state, &guid);
        state.push(guid); // Root payload through allocating/reentrant listeners.
        let result = dispatch_event_now(
            state,
            "GROUP_FORMED",
            &[Val::Num(f64::from(category)), guid],
        );
        state.pop();
        result?;
    }
    Ok(())
}

fn current_formation(sim: &crate::lua_api::state::SimState) -> Option<(i32, String)> {
    if sim.party_group_active {
        return None;
    }
    let follower_entry = sim.solo_follower_dungeon && sim.world.in_instance;
    if !sim.has_active_delve && !follower_entry {
        return None;
    }
    let category = sim.solo_group_category?;
    if !matches!(category, 1 | 2) {
        return None;
    }
    let guid = sim
        .solo_group_guid
        .as_ref()
        .filter(|guid| !guid.is_empty())?;
    Some((category, guid.clone()))
}
