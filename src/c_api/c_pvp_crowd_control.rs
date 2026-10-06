//! Arena crowd-control durations from GUID-keyed host timer snapshots.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[derive(Debug, Clone, Copy)]
pub struct CrowdControlWindow {
    pub start_time: f64,
    pub duration: f64,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_PvP")?;
    table_set_rust_fn_static(state, ns, "GetArenaCrowdControlDuration", query_duration)
}

fn query_duration(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let window = {
        let sim = borrow_state(state)?;
        let now = sim.start_time.elapsed().as_secs_f64();
        crate::lua_api::globals::unit_misc::existing_guid_for_unit(&sim, &unit)
            .and_then(|guid| sim.arena_crowd_control.get(&guid).copied())
            .filter(|window| window.duration > 0.0 && now < window.start_time + window.duration)
    };
    // INFERRED: unknown or inactive units receive an empty duration object.
    // Native arena CC selection, production and secrecy remain unmodeled.
    let (start, duration) =
        window.map_or((0.0, 0.0), |window| (window.start_time, window.duration));
    crate::lua_api::globals::lua_duration_object::push_timed_duration_object(state, start, duration)
}
