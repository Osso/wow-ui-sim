//! rilua RustFn equivalents: C_Timer API, layout bridge, and string.format patch.
//!
//! # Timer API (C_Timer)
//!
//! Mirrors `globals/timer_api.rs` but uses rilua's `RustFn` (stateless function
//! pointers) instead of mlua closures. Because `RustFn = fn(&mut LuaState) ->
//! LuaResult<u32>` cannot capture `Rc<RefCell<SimState>>`, state is accessed via
//! `state.app_data::<WowLuaAppData>()`.  Timer callbacks are stored in a rilua
//! registry table (`__rilua_timer_callbacks`, keyed by timer ID) so GC keeps them
//! alive without mlua RegistryKeys.
//!
//! # OnUpdate dispatch
//!
//! Already implemented in `script_helpers::dispatch_on_update`. No
//! additional RustFns are needed here; callers use that function directly.
//!
//! # Layout bridge
//!
//! `GetFrameRect` and the individual edge/size queries are provided as RustFns
//! that accept a frame-backed table as `self` (argument 1) and return WoW
//! UI-coordinate values.  They mirror `frame/methods/methods_rect.rs` but use
//! rilua state access via `methods::borrow_state`.
//!
//! # String format
//!
//! `register_string_format` patches `string.format` in the rilua VM with a
//! Lua-level shim (loaded via `rilua::Lua::exec`) that handles `%F` and
//! positional arguments (`%1$s`).  Since the format logic is pure string
//! manipulation it can live entirely in Lua; the Rust implementation in
//! `string_format.rs` stays on the mlua side only.

use crate::lua_api::env::WowLuaAppData;
use crate::lua_api::methods::{borrow_state, registry_get, table_get};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{call_function_state, create_table, table_set};
use crate::lua_api::next_timer_id;
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::closure::{Closure, RustClosure, RustFn};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};
use std::time::{Duration, Instant};

// ── Constants ────────────────────────────────────────────────────────────────

/// Maximum allowed timer duration in seconds ((2^32 - 1) / 1000).
const MAX_TIMER_SECS: f64 = (u32::MAX as f64) / 1000.0;

/// Earlier Retail profiles retain their pre-R88y dual-publication route. Evidence: pr46-49-decomp-answers.md, sections R88y.
const LEGACY_RETAIL_TIMER_FLAGS: u8 = 3;

/// Registry key for the timer callback storage table.
const TIMER_CALLBACKS_KEY: &str = "__rilua_timer_callbacks";
pub(super) const TIMER_CONTAINER_FUNCTION: &str = "__rilua_timer_function";
pub(super) const TIMER_CONTAINER_OBJECT: &str = "__rilua_timer_container";

// ── Timer callback storage ───────────────────────────────────────────────────

/// Get or create the registry table that stores timer callbacks.
/// Returns a `GcRef<Table>` for the callback table.
fn timer_callback_table(state: &mut LuaState) -> rilua::vm::gc::arena::GcRef<Table> {
    let key_ref = state
        .gc
        .intern_string_static(TIMER_CALLBACKS_KEY.as_bytes());
    let registry = state.gc.tables.get(state.registry);
    if let Some(reg) = registry {
        if let Val::Table(t) = reg.get_str(key_ref, &state.gc.string_arena) {
            return t;
        }
    }
    let new_table = state.gc.alloc_table(Table::new());
    if let Some(reg) = state.gc.tables.get_mut(state.registry) {
        let _ = reg.raw_set(
            Val::Str(key_ref),
            Val::Table(new_table),
            &state.gc.string_arena,
        );
    }
    // Track the registry -> callback-table edge so the incremental collector
    // does not reclaim the freshly inserted table (the registry may already be
    // marked black). Without this, callbacks vanish under GC pressure.
    state.gc.barrier_back(state.registry);
    new_table
}

/// Store a callback `Val` for a timer ID in the registry table.
pub(crate) fn store_timer_callback(state: &mut LuaState, timer_id: u64, callback: Val) {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    let (stored_callback, source_callback) = timer_container_callback_record(state, callback);
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    let (stored_callback, source_callback) = (callback, callback);

    let table = timer_callback_table(state);
    if let Some(t) = state.gc.tables.get_mut(table) {
        let _ = t.raw_set(
            Val::Num(timer_id as f64),
            stored_callback,
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(table);
    if super::handler_timing::is_enabled() {
        let source = super::handler_timing::TimerCallbackSource::capture(state, source_callback);
        if let Some(app) = state.app_data::<WowLuaAppData>() {
            app.sim_state
                .borrow_mut()
                .timer_callback_sources
                .insert(timer_id, source);
        }
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn timer_container_callback_record(state: &mut LuaState, container: Val) -> (Val, Val) {
    let Some((_, _, callback)) = function_container_data(state, container) else {
        return (container, container);
    };
    let record = create_table(state);
    table_set(state, record, TIMER_CONTAINER_FUNCTION, callback);
    table_set(state, record, TIMER_CONTAINER_OBJECT, container);
    (record, callback)
}

/// Read callback state from a modeled FunctionContainer's weak backing map.
pub(crate) fn function_container_data(
    state: &mut LuaState,
    object: Val,
) -> Option<(Val, Val, Val)> {
    if !matches!(object, Val::Userdata(_)) {
        return None;
    }
    let bridge = registry_get(state, "__wow_function_containers");
    let objects = table_get(state, bridge, "objects");
    let Val::Table(map) = objects else {
        return None;
    };
    let backing = state
        .gc
        .tables
        .get(map)?
        .get(object, &state.gc.string_arena);
    let Val::Table(_) = backing else { return None };
    let callback = table_get(state, backing, "_callback");
    matches!(callback, Val::Function(_)).then_some((objects, backing, callback))
}

/// Queue a Rust callback for the next timer-processing boundary.
pub(crate) fn defer_rust_event(
    state: &mut LuaState,
    dispatch: RustFn,
    label: &'static str,
) -> LuaResult<u64> {
    let callback = Val::Function(
        state
            .gc
            .alloc_closure(Closure::Rust(RustClosure::new(dispatch, label))),
    );
    let id = next_timer_id(state)?;
    store_timer_callback(state, id, callback);
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let (owner_addon, fire_at) = {
        let sim = app.sim_state.borrow();
        (
            sim.loading_addon_index.or(sim.executing_addon_index),
            sim.clock_now(),
        )
    };
    enqueue_rust_event_timer(&app, id, owner_addon, fire_at);
    Ok(id)
}

fn enqueue_rust_event_timer(
    app: &WowLuaAppData,
    id: u64,
    owner_addon: Option<u16>,
    fire_at: Instant,
) {
    let timer = RiluaPendingTimer {
        id,
        fire_at,
        interval: None,
        remaining: None,
        cancelled: false,
        owner_addon,
        callback_receives_timer: false,
        callback_arg: None,
    };
    app.sim_state.borrow_mut().rilua_timers.push_back(timer);
}

/// Remove a timer callback from the registry table (called after firing/cancel).
pub fn remove_timer_callback(state: &mut LuaState, timer_id: u64) {
    if super::handler_timing::is_enabled() {
        if let Some(app) = state.app_data::<WowLuaAppData>() {
            app.sim_state
                .borrow_mut()
                .timer_callback_sources
                .remove(&timer_id);
        }
    }
    let key_ref = state
        .gc
        .intern_string_static(TIMER_CALLBACKS_KEY.as_bytes());
    let registry = state.gc.tables.get(state.registry);
    let Some(reg) = registry else { return };
    let callback_table = match reg.get_str(key_ref, &state.gc.string_arena) {
        Val::Table(t) => t,
        _ => return,
    };
    if let Some(t) = state.gc.tables.get_mut(callback_table) {
        let _ = t.raw_set(Val::Num(timer_id as f64), Val::Nil, &state.gc.string_arena);
    }
    state.gc.barrier_back(callback_table);
}

/// Retrieve a stored timer callback by ID.
pub fn get_timer_callback(state: &mut LuaState, timer_id: u64) -> Val {
    let key_ref = state
        .gc
        .intern_string_static(TIMER_CALLBACKS_KEY.as_bytes());
    // Resolve the callback table GcRef from the registry.
    let callback_table_ref = {
        let reg_key = state.registry;
        let Some(reg) = state.gc.tables.get(reg_key) else {
            return Val::Nil;
        };
        match reg.get_str(key_ref, &state.gc.string_arena) {
            Val::Table(t) => t,
            _ => return Val::Nil,
        }
    };
    // Look up the timer callback by ID.
    let Some(t) = state.gc.tables.get(callback_table_ref) else {
        return Val::Nil;
    };
    t.get(Val::Num(timer_id as f64), &state.gc.string_arena)
}

// ── Validation ───────────────────────────────────────────────────────────────

fn validate_seconds(seconds: f64) -> LuaResult<f64> {
    if seconds < 0.0 || seconds > MAX_TIMER_SECS || seconds.is_nan() || seconds.is_infinite() {
        return Err(runtime_error("bad argument #1 (invalid duration)"));
    }
    Ok(seconds)
}

/// Extract a function `Val` from the stack, accepting plain functions only.
/// Returns an error for non-function values.
fn extract_callback(state: &LuaState, index: i32) -> LuaResult<Val> {
    let val = stack_val(state, index);
    match val {
        Val::Function(_) => Ok(val),
        _ => Err(runtime_error(format!(
            "bad argument #{index} (function expected, got {})",
            val.type_name()
        ))),
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn extract_native_timer_callback(state: &mut LuaState, index: i32) -> LuaResult<Val> {
    let callback = stack_val(state, index);
    if matches!(callback, Val::Function(_)) {
        let namespace = crate::c_api::ensure_namespace(state, "C_FunctionContainers")?;
        let create = table_get(state, Val::Table(namespace), "CreateCallback");
        return call_function_state(state, create, &[callback]);
    }
    if function_container_data(state, callback).is_some() {
        return Ok(callback);
    }
    Err(runtime_error(format!(
        "bad argument #{index} (function or callback expected)"
    )))
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn create_timer_container_proxy(state: &mut LuaState, container: Val) -> LuaResult<Val> {
    let Some((objects, backing, _)) = function_container_data(state, container) else {
        return Err(runtime_error("timer callback container is unavailable"));
    };
    let constructor = table_get(state, Val::Table(state.global), "newproxy");
    let proxy = call_function_state(state, constructor, &[container])?;
    let Val::Table(map) = objects else {
        unreachable!()
    };
    state
        .gc
        .tables
        .get_mut(map)
        .ok_or_else(|| runtime_error("function container map was collected"))?
        .raw_set(proxy, backing, &state.gc.string_arena)?;
    state.gc.barrier_back(map);
    Ok(proxy)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn timer_container_handle_list(state: &mut LuaState, backing: Val) -> Val {
    let handles = table_get(state, backing, "_timerHandles");
    if matches!(handles, Val::Table(_)) {
        return handles;
    }
    let handles = create_table(state);
    table_set(state, backing, "_timerHandles", handles);
    handles
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn bind_timer_container_handle(state: &mut LuaState, container: Val, handle: Val) -> LuaResult<()> {
    let Some((_, backing, _)) = function_container_data(state, container) else {
        return Err(runtime_error("timer callback container is unavailable"));
    };
    let Val::Table(list) = timer_container_handle_list(state, backing) else {
        unreachable!()
    };
    let len = state
        .gc
        .tables
        .get(list)
        .map_or(0, |table| table.len(&state.gc.string_arena));
    state
        .gc
        .tables
        .get_mut(list)
        .ok_or_else(|| runtime_error("timer handles were collected"))?
        .raw_set(Val::Num((len + 1) as f64), handle, &state.gc.string_arena)?;
    state.gc.barrier_back(list);
    let id = table_get(state, handle, "__id");
    table_set(state, backing, "__id", id);
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn registration_result(state: &mut LuaState, id: u64, callback: Val) -> LuaResult<(Val, Val)> {
    let handle = create_timer_handle_table(state, id)?;
    let proxy = create_timer_container_proxy(state, callback)?;
    bind_timer_container_handle(state, callback, handle)?;
    Ok((callback, proxy))
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn registration_result(state: &mut LuaState, id: u64, _callback: Val) -> LuaResult<(Val, Val)> {
    let handle = create_timer_handle_table(state, id)?;
    Ok((handle, handle))
}

// ── RiluaPendingTimer ────────────────────────────────────────────────────────

/// A pending timer for the rilua VM. Unlike `PendingTimer` (which uses
/// `mlua::RegistryKey`), callbacks are stored in the rilua registry table
/// and referenced by ID. The `cancelled` flag is communicated by nilling
/// the callback entry in the registry table.
pub struct RiluaPendingTimer {
    /// Unique timer ID (shared key with `__rilua_timer_callbacks` table).
    pub id: u64,
    /// When this timer should fire.
    pub fire_at: Instant,
    /// Repeat interval (None = one-shot).
    pub interval: Option<Duration>,
    /// Remaining iterations for tickers with a limit.
    pub remaining: Option<i32>,
    /// Whether this timer has been cancelled.
    pub cancelled: bool,
    /// Addon that created this timer.
    pub owner_addon: Option<u16>,
    /// Whether the callback receives `callback_arg` as its first argument.
    pub callback_receives_timer: bool,
    /// NewTimer/NewTicker callback argument: a legacy handle or container proxy.
    pub callback_arg: Option<Val>,
}

// ── C_Timer.After ────────────────────────────────────────────────────────────

/// `C_Timer.After(seconds, callback)` — one-shot timer, no handle returned.
///
/// Accepts non-negative `seconds` and the active profile's supported callback.
/// The callback is stored in `__rilua_timer_callbacks[id]` and a pending timer is pushed onto
/// `SimState.rilua_timers`.
fn timer_after(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = parse_after_seconds(state)?;
    let callback = extract_profile_timer_callback(state, 2)?;
    enqueue_after_timer(state, seconds, callback)?;
    Ok(0)
}

fn parse_after_seconds(state: &LuaState) -> LuaResult<f64> {
    match stack_val(state, 1) {
        Val::Num(seconds) => Ok(seconds.max(0.0)),
        value => Err(runtime_error(format!(
            "bad argument #1 to 'After' (number expected, got {})",
            value.type_name()
        ))),
    }
}

fn extract_profile_timer_callback(state: &mut LuaState, index: i32) -> LuaResult<Val> {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    return extract_native_timer_callback(state, index);
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    extract_callback(state, index)
}

fn enqueue_after_timer(state: &mut LuaState, seconds: f64, callback: Val) -> LuaResult<()> {
    let id = next_timer_id(state)?;
    store_timer_callback(state, id, callback);
    let fire_at = borrow_state(state)?.clock_now() + Duration::from_secs_f64(seconds);
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let sim = app.sim_state.borrow();
    let owner_addon = sim.loading_addon_index.or(sim.executing_addon_index);
    drop(sim);
    app.sim_state
        .borrow_mut()
        .rilua_timers
        .push_back(RiluaPendingTimer {
            id,
            fire_at,
            interval: None,
            remaining: None,
            cancelled: false,
            owner_addon,
            callback_receives_timer: false,
            callback_arg: None,
        });
    Ok(())
}

// ── Timer handle table ───────────────────────────────────────────────────────

/// Create a timer handle table with a `:Cancel()` method.
///
/// The table has a `__id` field (the timer ID as a number) and a `Cancel`
/// function that nils out the callback entry, marking the timer as cancelled.
pub(crate) fn create_timer_handle_table(state: &mut LuaState, timer_id: u64) -> LuaResult<Val> {
    let handle = TableBuilder::new(state)
        .set("__id", timer_id)?
        .set_function("Cancel", timer_handle_cancel)?
        .build();
    Ok(handle)
}

/// `:Cancel()` method on timer handle tables.
///
/// Reads `self.__id`, then nils the corresponding entry in `__rilua_timer_callbacks`.
fn timer_handle_cancel(state: &mut LuaState) -> LuaResult<u32> {
    let self_val = stack_val(state, 1);
    let Val::Table(table_ref) = self_val else {
        return Err(runtime_error("Cancel: expected table self"));
    };
    let id_key = state.gc.intern_string(b"__id");
    let id_val = state
        .gc
        .tables
        .get(table_ref)
        .map(|t| t.get_str(id_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let Val::Num(id_f) = id_val else {
        return Ok(0);
    };
    let timer_id = id_f as u64;
    remove_timer_callback(state, timer_id);
    // Also mark cancelled in the rilua timer queue (best-effort: iterate and flip flag).
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let mut sim = app.sim_state.borrow_mut();
    for t in sim.rilua_timers.iter_mut() {
        if t.id == timer_id {
            t.cancelled = true;
            break;
        }
    }
    Ok(0)
}

// ── Shared timer helpers ─────────────────────────────────────────────────────

/// Parse and validate the seconds argument (stack index 1) for `NewTicker`/`NewTimer`.
fn parse_validated_seconds(state: &LuaState, fn_name: &str) -> LuaResult<f64> {
    match stack_val(state, 1) {
        Val::Num(n) => validate_seconds(n),
        got => Err(runtime_error(format!(
            "bad argument #1 to '{fn_name}' (number expected, got {})",
            got.type_name()
        ))),
    }
}

/// Read the current addon index and enqueue `timer` onto `rilua_timers`.
fn enqueue_timer(state: &mut LuaState, mut timer: RiluaPendingTimer) -> LuaResult<()> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    let mut sim = app.sim_state.borrow_mut();
    timer.owner_addon = sim.loading_addon_index.or(sim.executing_addon_index);
    sim.rilua_timers.push_back(timer);
    Ok(())
}

// ── C_Timer.NewTicker ────────────────────────────────────────────────────────

/// `C_Timer.NewTicker(seconds, callback, iterations?)` — repeating timer.
fn timer_new_ticker(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = parse_validated_seconds(state, "NewTicker")?;
    let callback = extract_profile_timer_callback(state, 2)?;
    let iterations = parse_ticker_iterations(state);
    schedule_ticker(state, seconds, callback, iterations)?;
    Ok(1)
}

fn parse_ticker_iterations(state: &LuaState) -> Option<i32> {
    match stack_val(state, 3) {
        Val::Num(iterations) => Some(iterations as i32),
        _ => None,
    }
}

fn schedule_ticker(
    state: &mut LuaState,
    seconds: f64,
    callback: Val,
    iterations: Option<i32>,
) -> LuaResult<()> {
    let id = next_timer_id(state)?;
    store_timer_callback(state, id, callback);
    let (return_value, callback_arg) = registration_result(state, id, callback)?;
    let interval = Duration::from_secs_f64(seconds);
    let timer = RiluaPendingTimer {
        id,
        fire_at: borrow_state(state)?.clock_now() + interval,
        interval: Some(interval),
        remaining: iterations,
        cancelled: false,
        owner_addon: None,
        callback_receives_timer: true,
        callback_arg: Some(callback_arg),
    };
    enqueue_timer(state, timer)?;
    state.push(return_value);
    Ok(())
}

// ── C_Timer.NewTimer ─────────────────────────────────────────────────────────

/// `C_Timer.NewTimer(seconds, callback)` — one-shot timer.
fn timer_new_timer(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = parse_validated_seconds(state, "NewTimer")?;
    let callback = extract_profile_timer_callback(state, 2)?;
    schedule_one_shot_timer(state, seconds, callback)?;
    Ok(1)
}

fn schedule_one_shot_timer(state: &mut LuaState, seconds: f64, callback: Val) -> LuaResult<()> {
    let id = next_timer_id(state)?;
    store_timer_callback(state, id, callback);
    let (return_value, callback_arg) = registration_result(state, id, callback)?;
    let timer = RiluaPendingTimer {
        id,
        fire_at: borrow_state(state)?.clock_now() + Duration::from_secs_f64(seconds),
        interval: None,
        remaining: None,
        cancelled: false,
        owner_addon: None,
        callback_receives_timer: true,
        callback_arg: Some(callback_arg),
    };
    enqueue_timer(state, timer)?;
    state.push(return_value);
    Ok(())
}

// ── Layout bridge ─────────────────────────────────────────────────────────────
//
// These RustFns mirror frame rect query methods from `methods_rect.rs`.
// They are registered on the rilua frame metatable (or as globals during
// development) and use `methods::frame_id_from_stack` + `borrow_state`.

/// Helper: resolve the rect for a frame and return (left, bottom, width, height)
/// in WoW UI coordinates, or nil×4 if no rect is available.
///
/// `self` is argument 1 (a frame-backed table). Returns 4 numbers or 0 values.
// ── Layout: public table builder ─────────────────────────────────────────────

/// Register layout query RustFns on an existing table (e.g. the frame
/// metatable). Delegates to `rect_geometry::register_rect_methods_on_table`
/// — kept as a name-compatible wrapper for historical callers.
pub fn register_layout_fns_on_table(
    state: &mut LuaState,
    table: rilua::vm::gc::arena::GcRef<Table>,
) -> LuaResult<()> {
    crate::lua_api::rect_geometry::register_rect_methods_on_table(state, table)
}

// ── register_all ─────────────────────────────────────────────────────────────

/// Register timer and layout RustFns on a rilua VM.
///
/// - `C_Timer` global table with `After`, `NewTicker`, `NewTimer`.
/// - `__rilua_layout_fns` global table with the layout query functions
///   (caller merges these into the frame metatable as appropriate).
///
/// # OnUpdate
///
/// OnUpdate dispatch is already handled by
/// `script_helpers::dispatch_on_update` — no additional registration
/// is needed here.
pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_c_timer(lua)?;
    register_layout_globals(lua)?;
    if !crate::client_profile::is_active_retail_12_1_5() {
        // temporary: master's formatter and install point outside Retail 12.1.5.
        crate::lua_api::string_format_legacy::patch_string_format(lua)?;
    }
    Ok(())
}

/// Register the `C_Timer` global table with the three timer functions.
fn register_c_timer(lua: &mut rilua::Lua) -> LuaResult<()> {
    let retail = crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail;
    let state = lua.state_mut();
    register_c_timer_functions(state, retail)?;
    #[cfg(feature = "timed-signal-maps")]
    crate::c_api::timed_signal_map::register_c_timer_timed_signal_map(state)?;
    #[cfg(not(feature = "timed-signal-maps"))]
    mark_timed_signal_map_factory_removed(state)?;
    Ok(())
}

fn register_c_timer_functions(state: &mut LuaState, retail: bool) -> LuaResult<()> {
    register_c_timer_member(state, retail, "After", timer_after)?;
    register_c_timer_member(state, retail, "NewTicker", timer_new_ticker)?;
    register_c_timer_member(state, retail, "NewTimer", timer_new_timer)
}

#[cfg(not(feature = "timed-signal-maps"))]
fn mark_timed_signal_map_factory_removed(state: &mut LuaState) -> LuaResult<()> {
    // Keep namespace lookup from fabricating an unsupported factory.
    use crate::lua_api::methods::{create_table, table_set_static};

    let c_timer_ref = crate::c_api::ensure_namespace(state, "C_Timer")?;
    let removed = create_table(state);
    table_set_static(state, removed, "NewTimedSignalMap", Val::Bool(true));
    table_set_static(
        state,
        Val::Table(c_timer_ref),
        "__wow_removed_keys",
        removed,
    );
    Ok(())
}

/// Register `__rilua_layout_fns` global table holding the layout query RustFns.
///
/// Callers that build the frame metatable should copy these entries onto it
/// (or set `__rilua_layout_fns` as a fallback `__index` delegate).
fn register_c_timer_member(
    state: &mut LuaState,
    retail: bool,
    member: &str,
    function: rilua::RustFn,
) -> LuaResult<()> {
    if retail && crate::client_profile::is_active_retail_12_1_5() {
        crate::c_api::register_namespace_function_from_r88y(state, "C_Timer", member, function)
    } else if retail {
        crate::c_api::register_namespace_function_with_flags(
            state,
            "C_Timer",
            member,
            function,
            LEGACY_RETAIL_TIMER_FLAGS,
        )
    } else {
        crate::c_api::register_unknown_namespace_function(state, "C_Timer", member, function)
    }
}

fn register_layout_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();

    let layout_ref = state.gc.alloc_table(Table::new());
    register_layout_fns_on_table(state, layout_ref)?;

    let key = state.gc.intern_string(b"__rilua_layout_fns");
    let global = state.global;
    state
        .gc
        .tables
        .get_mut(global)
        .ok_or_else(|| runtime_error("missing global table"))?
        .raw_set(
            Val::Str(key),
            Val::Table(layout_ref),
            &state.gc.string_arena,
        )?;

    Ok(())
}

#[cfg(all(test, any(feature = "client-retail", feature = "client-wowforever")))]
mod function_container_tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use crate::lua_api::methods::call_function_state_multi;

    #[test]
    fn native_timer_functions_accept_and_return_containers() {
        let env = WowLuaEnv::new().unwrap();
        for function in [timer_new_timer, timer_new_ticker] {
            let callback = env
                .eval::<Val>("return C_FunctionContainers.CreateCallback(function() end)")
                .unwrap();
            let result = {
                let mut lua = env.rilua_mut();
                let state = lua.state_mut();
                let callable = Val::Function(state.gc.alloc_closure(Closure::Rust(
                    RustClosure::new(function, "native_timer_container_test"),
                )));
                call_function_state_multi(
                    state,
                    callable,
                    &[Val::Num(3600.0), callback, Val::Num(1.0)],
                )
                .unwrap()
            };
            assert_eq!(result, vec![callback]);
        }
    }
}
