//! Minimap tracking defaults and Forever's quest tracking state.

use super::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string_static, create_table_with_fields, val_to_string,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::workarounds::RETAIL_TRACKING_DEFAULT_MASK;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

// Forever 70707 MinimapConstantsDocumentation.lua and the Camelot quest settings.
const TRIVIAL_QUESTS: u32 = 1024;
const TRIVIAL_QUESTS_INDEX: f64 = 1.0;
const FOREVER_FILTER_MASK: i32 = 0x1ff_ffff;

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_Minimap")?;
    // MinimapDocumentation.lua:28 in both profiles.
    table_set_rust_fn_static(state, namespace, "GetDefaultTrackingValue", query_default)?;
    if ACTIVE == ClientProfile::WowForever {
        table_set_rust_fn_static(state, namespace, "IsFilteredOut", is_filtered_out)?;
        table_set_rust_fn_static(state, namespace, "GetNumTrackingTypes", |state| {
            state.push(Val::Num(TRIVIAL_QUESTS_INDEX));
            Ok(1)
        })?;
        table_set_rust_fn_static(state, namespace, "GetTrackingFilter", get_tracking_filter)?;
        table_set_rust_fn_static(state, namespace, "GetTrackingInfo", get_tracking_info)?;
        table_set_rust_fn_static(state, namespace, "SetTracking", set_tracking)?;
        table_set_rust_fn_static(state, namespace, "ClearAllTracking", |state| {
            set_trivial_quests(state, false)
        })?;
    }
    Ok(())
}

fn query_default(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Nil => return Err(runtime_error("GetDefaultTrackingValue requires filterType")),
        Val::Num(number) => number,
        Val::Str(_) => val_to_string(state, value)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(0.0),
        _ => 0.0,
    };
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType integer overflow",
        ));
    }
    let filter = number as i32;
    let (max_filter, default_mask) = if ACTIVE == ClientProfile::WowForever {
        // Camelot InterfaceOverrides.lua defaults Trivial Quests to off.
        // Other filters retain the unmodeled, unfiltered behavior.
        (FOREVER_FILTER_MASK, !TRIVIAL_QUESTS)
    } else {
        (0x7fffff, RETAIL_TRACKING_DEFAULT_MASK)
    };
    if !(0..=max_filter).contains(&filter) {
        return Err(runtime_error(
            "GetDefaultTrackingValue filterType out of range",
        ));
    }
    let filter = filter as u32;
    state.push(Val::Bool(default_mask & filter == filter));
    Ok(1)
}

fn is_filtered_out(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(filter) = value else {
        return Err(runtime_error("IsFilteredOut requires a numeric filterType"));
    };
    if !(0.0..=f64::from(FOREVER_FILTER_MASK)).contains(&filter) || filter.fract() != 0.0 {
        return Err(runtime_error("IsFilteredOut filterType out of range"));
    }
    let filter = filter as u32;
    let filtered = filter & TRIVIAL_QUESTS != 0 && !borrow_state(state)?.track_trivial_quests;
    state.push(Val::Bool(filtered));
    Ok(1)
}

fn is_trivial_quests_index(state: &mut LuaState) -> LuaResult<bool> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    Ok(matches!(value, Val::Num(index) if index == TRIVIAL_QUESTS_INDEX))
}

fn get_tracking_filter(state: &mut LuaState) -> LuaResult<u32> {
    let filter = if is_trivial_quests_index(state)? {
        create_table_with_fields(state, &[("filterID", Val::Num(f64::from(TRIVIAL_QUESTS)))])
    } else {
        create_table_with_fields(
            state,
            &[("filterID", Val::Num(0.0)), ("spellID", Val::Num(0.0))],
        )
    };
    state.push(filter);
    Ok(1)
}

fn get_tracking_info(state: &mut LuaState) -> LuaResult<u32> {
    if !is_trivial_quests_index(state)? {
        state.push(Val::Nil);
        return Ok(1);
    }
    let active = borrow_state(state)?.track_trivial_quests;
    let name = match global_val(state, "MINIMAP_TRACKING_TRIVIAL_QUESTS") {
        name @ Val::Str(_) => name,
        _ => create_string_static(state, "Trivial Quests"),
    };
    let tracking_type = create_string_static(state, "filter");
    let info = create_table_with_fields(
        state,
        &[
            ("name", name),
            ("texture", Val::Num(0.0)),
            ("active", Val::Bool(active)),
            ("type", tracking_type),
            ("subType", Val::Num(0.0)),
        ],
    );
    state.push(info);
    Ok(1)
}

fn set_tracking(state: &mut LuaState) -> LuaResult<u32> {
    if !is_trivial_quests_index(state)? {
        return Ok(0);
    }
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let Val::Bool(enabled) = value else {
        return Err(runtime_error("SetTracking requires a boolean"));
    };
    set_trivial_quests(state, enabled)
}

fn set_trivial_quests(state: &mut LuaState, enabled: bool) -> LuaResult<u32> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        std::mem::replace(&mut sim.track_trivial_quests, enabled) != enabled
    };
    if changed {
        fire_named_event_state(state, "MINIMAP_UPDATE_TRACKING", &[]);
    }
    Ok(0)
}
