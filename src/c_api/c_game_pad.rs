//! Retail device-query defaults and bounded PTR/Forever gamepad policies.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set_static};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

type GamePadNamespace = GcRef<Table>;

/// Simulator input for one named stick. Coordinates are model inputs only;
/// publishing their Euclidean length is inferred policy, not native verification.
#[derive(Clone, Debug, PartialEq)]
pub struct MappedStick {
    pub config_name: String,
    pub x: f64,
    pub y: f64,
}

/// Ordered sticks for an explicitly configured environment-local snapshot.
/// This is deliberately not a physical device or the full native mapped state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MappedStickSnapshot {
    pub sticks: Vec<MappedStick>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_GamePad")?;
    if ACTIVE == ClientProfile::Retail {
        register_retail_device_queries(state, namespace)?;
    }
    if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        register_mapped_state_queries(state, namespace)?;
        register_hover_policy_queries(state, namespace)?;
    }
    if ACTIVE == ClientProfile::WowForever {
        table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)?;
    }
    Ok(())
}

fn register_retail_device_queries(
    state: &mut LuaState,
    namespace: GamePadNamespace,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetAllDeviceIDs", get_all_device_ids)?;
    table_set_rust_fn_static(state, namespace, "GetDeviceRawState", get_device_raw_state)?;
    table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)
}

fn register_mapped_state_queries(
    state: &mut LuaState,
    namespace: GamePadNamespace,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDeviceMappedState",
        get_device_mapped_state,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "StickIndexToConfigName",
        stick_index_to_config_name,
    )
}

fn register_hover_policy_queries(
    state: &mut LuaState,
    namespace: GamePadNamespace,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAllowHoverEventsWithFreeLook",
        get_allow_hover_events_with_free_look,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetAllowHoverEventsWithFreeLook",
        set_allow_hover_events_with_free_look,
    )
}

fn get_all_device_ids(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): no physical-device input means an empty list.
    let device_ids = create_table(state);
    state.push(device_ids);
    Ok(1)
}

fn get_device_raw_state(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): no supplied device means no raw state.
    state.push(Val::Nil);
    Ok(1)
}

fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (simulator mapping): project the existing CVar, not hardware state.
    let enabled = borrow_state(state)?.cvars.get_bool("GamePadEnable");
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_allow_hover_events_with_free_look(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.gamepad_allow_hover_events_with_free_look;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_allow_hover_events_with_free_look(state: &mut LuaState) -> LuaResult<u32> {
    // AllowedWhenUntainted: use VM access checks without clearing caller taint.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Bool(enabled) = value else {
        return Err(runtime_error(
            "C_GamePad.SetAllowHoverEventsWithFreeLook requires a boolean",
        ));
    };
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.gamepad_allow_hover_events_with_free_look == enabled {
            return Ok(0);
        }
        sim.gamepad_allow_hover_events_with_free_look = enabled;
    }
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(0);
    }
    // Change-only publication and state-before-callback are simulator guesses.
    fire_named_event_state(
        state,
        "GAME_PAD_ALLOW_HOVER_EVENTS_WITH_FREE_LOOK_CHANGED",
        &[Val::Bool(enabled)],
    );
    Ok(0)
}

fn get_device_mapped_state(state: &mut LuaState) -> LuaResult<u32> {
    // Drop the SimState borrow before any Lua allocation.
    let snapshot = borrow_state(state)?.gamepad_mapped_sticks.clone();
    let Some(snapshot) = snapshot else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let mapped = create_table(state);
    state.push(mapped);
    table_set_static(
        state,
        mapped,
        "stickCount",
        Val::Num(snapshot.sticks.len() as f64),
    );
    let sticks = create_table(state);
    table_set_static(state, mapped, "sticks", sticks);
    for (index, stick) in snapshot.sticks.iter().enumerate() {
        let value = create_table(state);
        // Attach each child to the rooted result before interning field keys.
        super::helpers::set_table_array(state, sticks, (index + 1) as i64, value);
        table_set_static(state, value, "len", Val::Num(stick.x.hypot(stick.y)));
    }
    Ok(1)
}

fn stick_index_to_config_name(state: &mut LuaState) -> LuaResult<u32> {
    // Only the ordinary numeric loop is source-observed. No coercion or
    // implicit secret unwrapping; rejection is explicit simulator policy.
    let Val::Num(index) = stack_val(state, 1) else {
        return Err(runtime_error(
            "C_GamePad.StickIndexToConfigName requires an ordinary numeric index",
        ));
    };
    let name = if index.is_finite() && index >= 0.0 && index.fract() == 0.0 {
        borrow_state(state)?
            .gamepad_mapped_sticks
            .as_ref()
            .and_then(|snapshot| snapshot.sticks.get(index as usize))
            .map(|stick| stick.config_name.clone())
    } else {
        None
    };
    let value = match name {
        Some(name) => Val::Str(state.gc.intern_string(name.as_bytes())),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}
