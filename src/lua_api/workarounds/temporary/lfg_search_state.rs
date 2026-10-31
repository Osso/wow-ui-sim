//! Undocumented search clearing; retire when the native LFG search protocol is modeled.
use crate::lua_api::{methods::borrow_state_mut, timer_layout};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn clear_search_results(state: &mut LuaState) -> LuaResult<u32> {
    let ids = {
        let mut sim = borrow_state_mut(state)?;
        sim.lfg_search_results.clear();
        let ids = std::mem::take(&mut sim.lfg_search_timer_ids);
        sim.rilua_timers.retain(|timer| !ids.contains(&timer.id));
        ids
    };
    for id in ids {
        timer_layout::remove_timer_callback(state, id);
    }
    Ok(0)
}
