//! Host-supplied content-tracking lists and Retail queries.
//!
//! Empty lists mean that the simulator host supplied no tracked IDs; they do not
//! establish the native client's initial tracking state.

use std::collections::BTreeMap;

use crate::c_api::helpers::{
    ensure_namespace, parse_numeric_string, read_untainted_signed32_argument,
};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_table, table_set_num};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct ContentTrackingState {
    /// Parsed tracking type to ordered ID list. The vector preserves duplicates.
    pub tracked_ids_by_type: BTreeMap<u32, Vec<i32>>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_ContentTracking")?;
    table_set_rust_fn_static(state, namespace, "GetTrackedIDs", get_tracked_ids)?;
    table_set_rust_fn_static(state, namespace, "IsTracking", is_tracking)
}

/// The decoded Retail reader uses zero when Lua numeric conversion fails.
fn tracking_type(state: &LuaState, index: i32) -> u32 {
    let value = stack_val(state, index);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|text| parse_numeric_string(text.data())),
        _ => None,
    }
    .unwrap_or(0.0);
    number as u32
}

fn get_tracked_ids(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED declaration shape: one ID table; an empty host list remains empty.
    let trackable_type = tracking_type(state, 1);
    let ids = borrow_state(state)?
        .content_tracking
        .tracked_ids_by_type
        .get(&trackable_type)
        .cloned()
        .unwrap_or_default();

    // VERIFIED local copy behavior: preserve ID order and duplicates.
    let result = create_table(state);
    let Val::Table(table) = result else {
        unreachable!();
    };
    for (index, id) in ids.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(f64::from(id)));
    }
    state.push(result);
    Ok(1)
}

fn is_tracking(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED declaration shape: one boolean result.
    let trackable_type = tracking_type(state, 1);
    let trackable_id = read_untainted_signed32_argument(
        state,
        2,
        "IsTracking",
        "Usage: local isTracking = C_ContentTracking.IsTracking(trackableType, trackableID)",
    )?;
    let matches = borrow_state(state)?
        .content_tracking
        .tracked_ids_by_type
        .get(&trackable_type)
        .is_some_and(|ids| ids.contains(&trackable_id));
    state.push(Val::Bool(matches));
    Ok(1)
}
