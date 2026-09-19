//! Minimal `C_PingSecure` surface.
//!
//! Retail PingUI registers secure callbacks through this namespace during
//! startup. The simulator does not model protected ping-wheel creation yet, so
//! callbacks are accepted and retained as inert compatibility state while
//! `CreateFrame` is a no-op.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CALLBACK_TABLE_KEY: &str = "__wow_ping_secure_callbacks";

pub(crate) fn register_c_ping_secure_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_PingSecure")?;
    table_set_rust_fn_static(state, table_ref, "CreateFrame", create_frame)?;
    #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
    register_global_callback_accessors(state)?;
    register_callback_setters(state, table_ref)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_global_callback_accessors(state: &mut LuaState) -> LuaResult<()> {
    let globals = state.global;
    table_set_rust_fn_static(
        state,
        globals,
        "GetSecurePendingButtonCallback",
        get_button_callback,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetSecurePendingPingOffScreenCallback",
        get_pending_ping_off_screen_callback,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetSecurePendingToggleRunCallback",
        get_toggle_run_callback,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "SetSecurePendingButtonCallback",
        set_button_callback,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "SetSecurePendingPingOffScreenCallback",
        set_pending_ping_off_screen_callback,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "SetSecurePendingToggleRunCallback",
        set_toggle_run_callback,
    )?;
    Ok(())
}

fn register_callback_setters(
    state: &mut LuaState,
    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    for (name, function) in PING_SECURE_CALLBACK_SETTERS {
        table_set_rust_fn_static(state, table_ref, name, *function)?;
    }
    table_set_rust_fn_static(
        state,
        table_ref,
        "ClearPendingPingOffScreenCallback",
        clear_pending_ping_off_screen_callback,
    )
}

const PING_SECURE_CALLBACK_SETTERS: &[(&str, fn(&mut LuaState) -> LuaResult<u32>)] = &[
    (
        "SetPendingPingOffScreenCallback",
        set_pending_ping_off_screen_callback,
    ),
    (
        "SetPingCooldownStartedCallback",
        set_ping_cooldown_started_callback,
    ),
    (
        "SetPingPinFrameAddedCallback",
        set_ping_pin_frame_added_callback,
    ),
    (
        "SetPingPinFrameRemovedCallback",
        set_ping_pin_frame_removed_callback,
    ),
    (
        "SetPingPinFrameScreenClampStateUpdatedCallback",
        set_ping_pin_frame_screen_clamp_state_updated_callback,
    ),
    (
        "SetPingRadialWheelCreatedCallback",
        set_ping_radial_wheel_created_callback,
    ),
    ("SetSendMacroPingCallback", set_send_macro_ping_callback),
    (
        "SetTogglePingListenerCallback",
        set_toggle_ping_listener_callback,
    ),
];

fn create_frame(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn set_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pendingPingOffScreen")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "pendingPingOffScreen")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn set_button_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "button")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_button_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "button")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn set_toggle_run_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "toggleRun")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_toggle_run_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "toggleRun")
}

fn clear_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    clear_callback(state, "pendingPingOffScreen")
}

fn set_ping_cooldown_started_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pingCooldownStarted")
}

fn set_ping_pin_frame_added_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pingPinFrameAdded")
}

fn set_ping_pin_frame_removed_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pingPinFrameRemoved")
}

fn set_ping_pin_frame_screen_clamp_state_updated_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pingPinFrameScreenClampStateUpdated")
}

fn set_ping_radial_wheel_created_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "pingRadialWheelCreated")
}

fn set_send_macro_ping_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "sendMacroPing")
}

fn set_toggle_ping_listener_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_callback(state, "togglePingListener")
}

fn set_callback(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    let callbacks = callback_table(state);
    table_set(state, callbacks, key, stack_val(state, 1));
    Ok(0)
}

fn clear_callback(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    let callbacks = callback_table(state);
    table_set(state, callbacks, key, Val::Nil);
    Ok(0)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_callback(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    let callbacks = callback_table(state);
    let callback = crate::lua_api::methods::table_get(state, callbacks, key);
    state.push(callback);
    Ok(1)
}

fn callback_table(state: &mut LuaState) -> Val {
    let key = state.gc.intern_string(CALLBACK_TABLE_KEY.as_bytes());
    let current = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if matches!(current, Val::Table(_)) {
        return current;
    }

    let callbacks = create_table(state);
    if let Some(globals) = state.gc.tables.get_mut(state.global) {
        let _ = globals.raw_set(Val::Str(key), callbacks, &state.gc.string_arena);
    }
    state.gc.barrier_back(state.global);
    callbacks
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn registers_ping_secure_api() {
        let env = WowLuaEnv::new().expect("lua env");
        let result: String = env
            .eval(
                r#"
                if type(C_PingSecure) ~= "table" then return "namespace" end
                if type(C_PingSecure.CreateFrame) ~= "function" then return "create_frame" end
                if type(C_PingSecure.SetPendingPingOffScreenCallback) ~= "function" then return "pending_callback" end
                if type(C_PingSecure.ClearPendingPingOffScreenCallback) ~= "function" then return "clear_pending_callback" end
                if type(C_PingSecure.SetPingCooldownStartedCallback) ~= "function" then return "cooldown_callback" end
                if type(C_PingSecure.SetPingPinFrameAddedCallback) ~= "function" then return "pin_added_callback" end
                if type(C_PingSecure.SetPingPinFrameRemovedCallback) ~= "function" then return "pin_removed_callback" end
                if type(C_PingSecure.SetPingPinFrameScreenClampStateUpdatedCallback) ~= "function" then return "pin_clamp_callback" end
                if type(C_PingSecure.SetPingRadialWheelCreatedCallback) ~= "function" then return "radial_callback" end
                if type(C_PingSecure.SetSendMacroPingCallback) ~= "function" then return "send_macro_callback" end
                if type(C_PingSecure.SetTogglePingListenerCallback) ~= "function" then return "toggle_callback" end
                C_PingSecure.CreateFrame()
                local pending = function() return "pending" end
                C_PingSecure.SetPendingPingOffScreenCallback(pending)
                if __wow_ping_secure_callbacks.pendingPingOffScreen ~= pending then return "pending_stored" end
                C_PingSecure.ClearPendingPingOffScreenCallback()
                if __wow_ping_secure_callbacks.pendingPingOffScreen ~= nil then return "pending_cleared" end
                C_PingSecure.SetPendingPingOffScreenCallback(function() end)
                C_PingSecure.SetPingCooldownStartedCallback(function() end)
                C_PingSecure.SetPingPinFrameAddedCallback(function() end)
                C_PingSecure.SetPingPinFrameRemovedCallback(function() end)
                C_PingSecure.SetPingPinFrameScreenClampStateUpdatedCallback(function() end)
                C_PingSecure.SetPingRadialWheelCreatedCallback(function() end)
                C_PingSecure.SetSendMacroPingCallback(function() end)
                C_PingSecure.SetTogglePingListenerCallback(function() end)
                if type(rawget(__secureenv, "C_PingSecure")) ~= "table" then return "secure_namespace" end
                return "ok"
                "#,
            )
            .expect("C_PingSecure probe");

        assert_eq!(result, "ok");
    }
}
