//! Host-supplied content-tracking lists and Retail/Forever queries.
//!
//! Empty lists mean that the simulator host supplied no tracked IDs; they do not
//! establish the native client's initial tracking state.

use std::collections::BTreeMap;

use crate::c_api::helpers::{
    ensure_namespace, parse_numeric_string, read_untainted_signed32_argument,
};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set_num};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub struct ContentTrackingState {
    /// INFERRED (R88cw): collectable-source tracking is disabled until supplied.
    pub collectable_source_tracking_enabled: bool,
    /// Parsed tracking type to ordered ID list. The vector preserves duplicates.
    pub tracked_ids_by_type: BTreeMap<u32, Vec<i32>>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }

    let namespace = ensure_namespace(state, "C_ContentTracking")?;
    // Retail and Forever ContentTrackingDocumentation.lua:151–163 declare
    // GetTrackedIDs. Sharing the host list model is INFERRED.
    table_set_rust_fn_static(state, namespace, "GetTrackedIDs", get_tracked_ids)?;
    // Retail and Forever ContentTrackingDocumentation.lua declares the
    // capability query at lines 29–35 and IsTracking at lines 233–245.
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetCollectableSourceTrackingEnabled",
            get_collectable_source_tracking_enabled,
        )?;
        table_set_rust_fn_static(state, namespace, "IsTracking", is_tracking)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        builder.set_function(
            "SetContentTrackingEnabled",
            set_collectable_source_tracking_enabled,
        )
    } else {
        Ok(builder)
    }
}

fn get_collectable_source_tracking_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .content_tracking
        .collectable_source_tracking_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_collectable_source_tracking_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = bool::from_stack(state, 1)?;
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let changed = sim.content_tracking.collectable_source_tracking_enabled != enabled;
        sim.content_tracking.collectable_source_tracking_enabled = enabled;
        changed
    };
    if changed {
        fire_named_event_state(
            state,
            "CONTENT_TRACKING_IS_ENABLED_UPDATE",
            &[Val::Bool(enabled)],
        );
    }
    Ok(0)
}

/// The decoded Retail reader uses zero when Lua numeric conversion fails.
fn tracking_type(state: &LuaState, value: Val) -> u32 {
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
    let argument = stack_val(state, 1);
    let argument = if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        // The PTR and Forever declarations are AllowedWhenUntainted.
        rilua::table_security::unwrap_secret(state, argument)?
    } else {
        argument
    };
    let trackable_type = tracking_type(state, argument);
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
    let trackable_type = tracking_type(state, stack_val(state, 1));
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
