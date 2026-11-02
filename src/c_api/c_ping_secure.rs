//! `C_PingSecure` callback slots, radial-frame creation, error display and sends.
//!
//! Retail 12.1.5 (WM38, WM38F, WM47, WM51) refuses tainted callers of every
//! member before reading arguments, creates the radial frame once, dispatches
//! `PING_SYSTEM_ERROR`, selects UI receivers and sends pings. Earlier epochs
//! keep inert callback storage and a no-op `CreateFrame`.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::create_frame::helpers_shared::create_frame_instance;
use crate::lua_api::globals::real::protected_globals::refuse_tainted_caller;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_table, frame_ref, table_get,
    table_set, val_to_string,
};
use crate::lua_api::script_helpers::call_error_handler_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use crate::widget::{AnchorPoint, WidgetType};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

mod burst;
mod eligibility;
mod sends;
mod targeting;

pub use burst::{PingBurst, PingCooldownInterval};
pub use sends::PingPendingTarget;

const CALLBACK_TABLE_KEY: &str = "__wow_ping_secure_callbacks";
const RADIAL_FRAME_SIDE: f32 = 10.0;

/// WM38 and PingManagerSecureDocumentation.lua describe the Retail 12.1.5 surface.
fn is_retail_12_1_5() -> bool {
    crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail
        && cfg!(feature = "retail-12-1-5")
}

pub(crate) fn register_c_ping_secure_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_PingSecure")?;
    table_set_rust_fn_static(state, table_ref, "CreateFrame", create_frame)?;
    #[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
    register_global_callback_accessors(state)?;
    register_callback_setters(state, table_ref)?;
    if is_retail_12_1_5() {
        sends::register(state, table_ref)?;
        targeting::register(state, table_ref)?;
        // PingManagerSecureDocumentation.lua:21-30; the clear member is not declared.
        table_set_rust_fn_static(state, table_ref, "DisplayError", display_error)
    } else {
        table_set_rust_fn_static(
            state,
            table_ref,
            "ClearPendingPingOffScreenCallback",
            clear_pending_ping_off_screen_callback,
        )
    }
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
    Ok(())
}

const PING_SECURE_CALLBACK_SETTERS: &[(&str, fn(&mut LuaState) -> LuaResult<u32>)] = &[
    (
        "SetPendingPingOffScreenCallback",
        set_namespace_pending_ping_off_screen_callback,
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

/// WM38F: an unnamed native frame is created once, stored, then announced
/// synchronously to the installed radial callback; repeats are no-ops.
fn create_frame(state: &mut LuaState) -> LuaResult<u32> {
    if !is_retail_12_1_5() || refuse_tainted_caller(state)? {
        return Ok(0);
    }
    if borrow_state(state)?.ping_radial_frame.is_some() {
        return Ok(0);
    }
    let frame_id =
        create_frame_instance(state, WidgetType::Frame, "Frame", None, None, false, None)?;
    place_radial_frame(state, frame_id)?;
    borrow_state_mut(state)?.ping_radial_frame = Some(frame_id);
    notify_radial_wheel_created(state, frame_id)?;
    Ok(0)
}

/// WM47 Q10: the parentless frame's CENTER is anchored to WorldFrame's BOTTOMLEFT
/// at zero offsets; MEDIUM strata and level 0 are the frame defaults. WM51-F5/F6:
/// the native side (10/1024)/S x M is exactly 10 GetCursorPosition units, the
/// simulator's layout canvas (INFERRED), on every screen.
fn place_radial_frame(state: &LuaState, frame_id: u64) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let world_frame = sim
        .widgets
        .get_id_by_name("WorldFrame")
        .ok_or_else(|| runtime_error("C_PingSecure.CreateFrame requires WorldFrame"))?;
    if let Some(frame) = sim.widgets.get_mut_for_anchor_edit(frame_id) {
        frame.set_size(RADIAL_FRAME_SIDE, RADIAL_FRAME_SIDE);
        frame.set_point(
            AnchorPoint::Center,
            Some(world_frame as usize),
            AnchorPoint::BottomLeft,
            0.0,
            0.0,
        );
    }
    sim.widgets.add_anchor_dependent(world_frame, frame_id);
    sim.widgets.mark_anchor_rect_dirty(frame_id);
    Ok(())
}

fn notify_radial_wheel_created(state: &mut LuaState, frame_id: u64) -> LuaResult<()> {
    let callbacks = callback_table(state);
    let callback = table_get(state, callbacks, "pingRadialWheelCreated");
    if !matches!(callback, Val::Function(_)) {
        return Ok(());
    }
    let frame = frame_ref(state, frame_id)?;
    // WM38: callback wrappers isolate Lua errors from the native caller.
    if let Err(error) = call_function_state(state, callback, &[frame]) {
        call_error_handler_state(state, &error.to_string());
    }
    Ok(())
}

/// WM38: nonempty string-compatible text dispatches `PING_SYSTEM_ERROR`.
fn display_error(state: &mut LuaState) -> LuaResult<u32> {
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let text = read_error_text(state)?;
    if !text.is_empty() {
        let message = Val::Str(state.gc.intern_string(text.as_bytes()));
        dispatch_event_now(state, "PING_SYSTEM_ERROR", &[message])?;
    }
    Ok(0)
}

fn read_error_text(state: &LuaState) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let text = match value {
        Val::Str(_) => val_to_string(state, value),
        Val::Num(_) => Some(value.to_string()),
        _ => None,
    };
    text.ok_or_else(|| runtime_error("Usage: C_PingSecure.DisplayError(error)"))
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn set_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    store_stack_callback(state, "pendingPingOffScreen")
}

fn set_namespace_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    if is_retail_12_1_5() {
        return set_namespace_callback(state, "pendingPingOffScreen");
    }
    let callback = stack_val(state, 1);
    #[cfg(feature = "retail-12-0-7")]
    let callback = authenticate_pending_ping_callback(state, callback)?;
    let callbacks = callback_table(state);
    table_set(state, callbacks, "pendingPingOffScreen", callback);
    Ok(0)
}

#[cfg(feature = "retail-12-0-7")]
fn authenticate_pending_ping_callback(state: &LuaState, callback: Val) -> LuaResult<Val> {
    let callback = rilua::table_security::unwrap_secret(state, callback)?;
    // INFERRED: ignore extra arguments after authenticating every supplied value.
    for argument in state.stack.iter().take(state.top).skip(state.base + 1) {
        rilua::table_security::unwrap_secret(state, *argument)?;
    }
    // INFERRED: accept functions only, not nil or callable tables; clear uses the separate API.
    if !matches!(callback, Val::Function(_)) {
        return Err(rilua::runtime_error(
            "pending ping callback must be a function",
        ));
    }
    Ok(callback)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "pendingPingOffScreen")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn set_button_callback(state: &mut LuaState) -> LuaResult<u32> {
    store_stack_callback(state, "button")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_button_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "button")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn set_toggle_run_callback(state: &mut LuaState) -> LuaResult<u32> {
    store_stack_callback(state, "toggleRun")
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn get_toggle_run_callback(state: &mut LuaState) -> LuaResult<u32> {
    get_callback(state, "toggleRun")
}

fn clear_pending_ping_off_screen_callback(state: &mut LuaState) -> LuaResult<u32> {
    clear_callback(state, "pendingPingOffScreen")
}

fn set_ping_cooldown_started_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "pingCooldownStarted")
}

fn set_ping_pin_frame_added_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "pingPinFrameAdded")
}

fn set_ping_pin_frame_removed_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "pingPinFrameRemoved")
}

fn set_ping_pin_frame_screen_clamp_state_updated_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "pingPinFrameScreenClampStateUpdated")
}

fn set_ping_radial_wheel_created_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "pingRadialWheelCreated")
}

fn set_send_macro_ping_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "sendMacroPing")
}

fn set_toggle_ping_listener_callback(state: &mut LuaState) -> LuaResult<u32> {
    set_namespace_callback(state, "togglePingListener")
}

/// Retail 12.1.5 setters refuse tainted callers, then replace the slot with a
/// required callback; earlier epochs store the argument as given.
fn set_namespace_callback(state: &mut LuaState, key: &str) -> LuaResult<u32> {
    if !is_retail_12_1_5() {
        return store_stack_callback(state, key);
    }
    if refuse_tainted_caller(state)? {
        return Ok(0);
    }
    let callback = read_required_callback(state)?;
    let callbacks = callback_table(state);
    table_set(state, callbacks, key, callback);
    Ok(0)
}

/// WM38: a required callback read through the secret-aware converter.
/// INFERRED: only functions convert; missing, nil and other values are argument errors.
fn read_required_callback(state: &LuaState) -> LuaResult<Val> {
    let callback = unwrap_secret(state, stack_val(state, 1))?;
    match callback {
        Val::Function(_) => Ok(callback),
        _ => Err(runtime_error("ping callback must be a function")),
    }
}

fn store_stack_callback(state: &mut LuaState, key: &str) -> LuaResult<u32> {
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
    let callback = table_get(state, callbacks, key);
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
        // Retail 12.1.5 PingManagerSecureDocumentation.lua lists no
        // ClearPendingPingOffScreenCallback; its checks belong to the 12.0.7 epoch.
        let retail_12_1_5 = crate::client_profile::ACTIVE
            == crate::client_profile::ClientProfile::Retail
            && cfg!(feature = "retail-12-1-5");
        env.exec(&format!("PendingClearPublished = {}", !retail_12_1_5))
            .expect("record pending-clear publication");
        let result: String = env
            .eval(
                r#"
                local C_PingSecure = __secureenv.C_PingSecure
                if type(C_PingSecure) ~= "table" then return "namespace" end
                if type(C_PingSecure.CreateFrame) ~= "function" then return "create_frame" end
                if type(C_PingSecure.SetPendingPingOffScreenCallback) ~= "function" then return "pending_callback" end
                if PendingClearPublished and type(C_PingSecure.ClearPendingPingOffScreenCallback) ~= "function" then return "clear_pending_callback" end
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
                if PendingClearPublished then
                    C_PingSecure.ClearPendingPingOffScreenCallback()
                    if __wow_ping_secure_callbacks.pendingPingOffScreen ~= nil then return "pending_cleared" end
                end
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
