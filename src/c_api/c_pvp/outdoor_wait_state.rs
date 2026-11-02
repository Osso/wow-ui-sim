//! Supplied world-state values for outdoor PvP timers.
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Default)]
pub struct OutdoorPvpWorldState {
    pub values: HashMap<i32, i64>,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetOutdoorPvPWaitTime", query)
}

fn query(state: &mut LuaState) -> LuaResult<u32> {
    const USAGE: &str = "Usage: local pvpWaitTime = C_PvP.GetOutdoorPvPWaitTime(uiMapID)";
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(text) => state
            .gc
            .string_arena
            .get(text)
            .and_then(|s| std::str::from_utf8(s.data()).ok())
            .and_then(|s| s.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| runtime_error(USAGE))?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(USAGE));
    }
    let keys = match number.trunc() as i32 {
        123 => Some((3801, 4354)),
        244 => Some((5387, 5332)),
        _ => None,
    };
    let deadline = {
        let sim = borrow_state(state)?;
        keys.and_then(|(gate, deadline)| {
            let values = &sim.outdoor_pvp_world_state.values;
            values
                .get(&gate)
                .filter(|value| **value != 0)
                .map(|_| *values.get(&deadline).unwrap_or(&0))
        })
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| runtime_error(format!("system clock precedes Unix epoch: {error}")))?
        .as_secs();
    let remaining = deadline.map(|deadline| i128::from(deadline) - i128::from(now));
    let result = match remaining {
        Some(seconds) if seconds > 0 => Val::Num(seconds as f64),
        _ => Val::Nil,
    };
    state.push(result);
    Ok(1)
}
