//! LuaDurationObject table-proxy type for WoW duration tracking.
//!
//! Implements `C_DurationUtil.CreateDuration()` which returns a plain Lua
//! table with a shared metatable.  The result is `type() == "table"` — not
//! userdata — matching the rilua proxy convention used by `AbbreviateConfig`,
//! `FunctionContainer`, etc.
//!
//! The shared metatable is stored in the rilua registry under the key
//! `__duration_obj_mt`.  The methods table (all method closures) is stored
//! under `__duration_obj_methods`.  The per-instance numeric ID is stored at
//! the integer key `0` in each object table (not writable via `__newindex`
//! since that only guards string keys).
//!
//! Wired from `register_tail_globals` in `register.rs`, after
//! `missing_surface::register_all`, so the duration surface is installed before
//! Blizzard/addon Lua can request duration objects.

mod core;

use crate::lua_api::methods::{
    call_function_state, create_table, registry_get, registry_set, table_get, table_set,
    table_set_static,
};
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

const MT_KEY: &'static str = "__duration_obj_mt";
const METHODS_KEY: &'static str = "__duration_obj_methods";

/// Method names that `__newindex` treats as read-only.
const METHOD_NAMES: &[&str] = &[
    "Assign",
    "Copy",
    "EvaluateElapsedDuration",
    "EvaluateElapsedPercent",
    "EvaluateRemainingDuration",
    "EvaluateRemainingPercent",
    "GetClock",
    "GetClockTime",
    "GetElapsedDuration",
    "GetElapsedPercent",
    "GetEndTime",
    "GetModRate",
    "GetRemainingDuration",
    "GetRemainingPercent",
    "GetStartTime",
    "GetTotalDuration",
    "HasExpired",
    "HasSecretValues",
    "HasStarted",
    "IsActive",
    "IsZero",
    "Reset",
    "SetClock",
    "SetTimeFromEnd",
    "SetTimeFromStart",
    "SetTimeSpan",
    "SetToDefaults",
];

/// Metamethod names that are also read-only.
const META_NAMES: &[&str] = &["__eq", "__index", "__metatable", "__newindex", "__tostring"];

// ── Public entry point ────────────────────────────────────────────────────────

/// Register `C_DurationUtil.CreateDuration` and `C_DurationUtil.GetCurrentTime`
/// in the Lua globals.
///
/// If `C_DurationUtil` already exists as a table it is reused; otherwise a new
/// one is created.  `GetCurrentTime` is only written if the key is currently
/// nil (matching the master behaviour).
pub fn register_lua_duration_object(lua: &mut rilua::Lua) -> crate::Result<()> {
    let state = lua.state_mut();

    // Build the shared metatable once and stash it in the registry.
    ensure_metatable(state);

    let global = Val::Table(state.global);

    // Reuse or create C_DurationUtil namespace.
    let ns = {
        let existing = crate::lua_api::methods::table_get(state, global, "C_DurationUtil");
        if let Val::Table(_) = existing {
            existing
        } else {
            let ns = create_table(state);
            crate::lua_api::methods::table_set(state, global, "C_DurationUtil", ns);
            ns
        }
    };

    // Install CreateDuration.
    let create_fn = make_closure(state, "C_DurationUtil.CreateDuration", create_duration);
    table_set_static(state, ns, "CreateDuration", create_fn);

    // Install CreateManualClock only if missing.
    let existing = crate::lua_api::methods::table_get(state, ns, "CreateManualClock");
    if existing == Val::Nil {
        let create_clock_fn = make_closure(
            state,
            "C_DurationUtil.CreateManualClock",
            create_manual_clock,
        );
        table_set_static(state, ns, "CreateManualClock", create_clock_fn);
    }

    // Install GetCurrentTime only if missing.
    let existing = crate::lua_api::methods::table_get(state, ns, "GetCurrentTime");
    if existing == Val::Nil {
        let get_time_fn = make_closure(state, "C_DurationUtil.GetCurrentTime", get_current_time);
        table_set_static(state, ns, "GetCurrentTime", get_time_fn);
    }

    Ok(())
}

/// Create a new `LuaDurationObject` table value for callers that expose
/// duration objects through other namespaces such as `C_ActionBar`.
pub(crate) fn new_duration_object_value(state: &mut LuaState) -> Val {
    ensure_metatable(state);
    new_duration_object(state)
}

/// Push a duration snapshot using the same validated setter exposed to Lua.
pub(crate) fn push_timed_duration_object(
    state: &mut LuaState,
    start: f64,
    seconds: f64,
) -> LuaResult<u32> {
    let duration = new_duration_object_value(state);
    state.push(duration);
    let key = state.gc.intern_string(b"SetTimeFromStart");
    let set_time = state.gettable(duration, Val::Str(key))?;
    call_function_state(
        state,
        set_time,
        &[duration, Val::Num(start), Val::Num(seconds), Val::Num(1.0)],
    )?;
    Ok(1)
}

// ── Metamethod / method implementations ──────────────────────────────────────

/// `C_DurationUtil.CreateDuration()` — returns a new duration proxy table.
fn create_duration(state: &mut LuaState) -> LuaResult<u32> {
    let obj = new_duration_object(state);
    state.push(obj);
    Ok(1)
}

/// `C_DurationUtil.CreateManualClock(initialTime)` — best-effort mutable clock table.
fn create_manual_clock(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let time = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0);
    let clock = create_table(state);
    table_set(state, clock, "time", Val::Num(time));
    install_clock_method(
        state,
        clock,
        "GetTime",
        "ManualClock.GetTime",
        clock_get_time,
    );
    install_clock_method(
        state,
        clock,
        "SetTime",
        "ManualClock.SetTime",
        clock_set_time,
    );
    install_clock_method(
        state,
        clock,
        "AdvanceTime",
        "ManualClock.AdvanceTime",
        clock_advance_time,
    );
    install_clock_method(
        state,
        clock,
        "RewindTime",
        "ManualClock.RewindTime",
        clock_rewind_time,
    );
    install_clock_method(
        state,
        clock,
        "ResetTime",
        "ManualClock.ResetTime",
        clock_reset_time,
    );
    state.push(clock);
    Ok(1)
}

/// `C_DurationUtil.GetCurrentTime()` — simulator elapsed time, matching GetTime.
fn get_current_time(state: &mut LuaState) -> LuaResult<u32> {
    let time = core::current_time(state)?;
    state.push(Val::Num(time));
    Ok(1)
}

fn clock_get_time(state: &mut LuaState) -> LuaResult<u32> {
    let clock = crate::lua_bridge::stack_val(state, 1);
    let time = clock_time(state, clock);
    state.push(time);
    Ok(1)
}

fn clock_set_time(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let clock = crate::lua_bridge::stack_val(state, 1);
    let time = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0);
    table_set(state, clock, "time", Val::Num(time));
    Ok(0)
}

fn clock_advance_time(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let clock = crate::lua_bridge::stack_val(state, 1);
    let delta = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0);
    let time = clock_time_number(state, clock) + delta;
    table_set(state, clock, "time", Val::Num(time));
    Ok(0)
}

fn clock_rewind_time(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let clock = crate::lua_bridge::stack_val(state, 1);
    let delta = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0);
    let time = clock_time_number(state, clock) - delta;
    table_set(state, clock, "time", Val::Num(time));
    Ok(0)
}

fn clock_reset_time(state: &mut LuaState) -> LuaResult<u32> {
    let clock = crate::lua_bridge::stack_val(state, 1);
    table_set(state, clock, "time", Val::Num(0.0));
    Ok(0)
}

fn clock_time(state: &mut LuaState, clock: Val) -> Val {
    table_get(state, clock, "time")
}

fn clock_time_number(state: &mut LuaState, clock: Val) -> f64 {
    match clock_time(state, clock) {
        Val::Num(time) => time,
        _ => 0.0,
    }
}

fn install_clock_method(
    state: &mut LuaState,
    clock: Val,
    key: &'static str,
    closure_name: &'static str,
    func: rilua::RustFn,
) {
    let closure = make_closure(state, closure_name, func);
    table_set_static(state, clock, key, closure);
}

/// `__index(table, key)` — look up `key` in the instance table first, then
/// fall back to the shared methods table.
fn duration_index(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let self_val = crate::lua_bridge::stack_val(state, 1);
    let key = Option::<String>::from_stack(state, 2)?.unwrap_or_default();

    // Raw read from the instance table.
    let raw = match self_val {
        Val::Table(t_ref) => {
            let key_ref = state.gc.intern_string(key.as_bytes());
            state
                .gc
                .tables
                .get(t_ref)
                .map(|t| t.get_str(key_ref, &state.gc.string_arena))
                .unwrap_or(Val::Nil)
        }
        _ => Val::Nil,
    };
    if raw != Val::Nil {
        state.push(raw);
        return Ok(1);
    }

    // Fall back to methods table.
    let methods = registry_get(state, METHODS_KEY);
    let val = match methods {
        Val::Table(_) => crate::lua_api::methods::table_get(state, methods, &key),
        _ => Val::Nil,
    };
    state.push(val);
    Ok(1)
}

/// `__newindex(table, key, value)` — block writes to read-only keys; raw-set
/// everything else directly into the instance table.
fn duration_newindex(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::FromStack;
    let self_val = crate::lua_bridge::stack_val(state, 1);
    let key = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let value = crate::lua_bridge::stack_val(state, 3);

    if is_readonly_key(&key) {
        return Err(rilua::runtime_error(format!(
            "Attempted to assign to read-only key {}",
            key
        )));
    }

    if let Val::Table(t_ref) = self_val {
        let key_ref = state.gc.intern_string(key.as_bytes());
        if let Some(t) = state.gc.tables.get_mut(t_ref) {
            let _ = t.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
        }
        state.gc.barrier_back(t_ref);
    }
    Ok(0)
}

/// `__tostring(table)` — returns `"LuaDurationObject: 0x<hex-id>"`.
fn duration_tostring(state: &mut LuaState) -> LuaResult<u32> {
    let self_val = crate::lua_bridge::stack_val(state, 1);
    let id = match self_val {
        Val::Table(t_ref) => state
            .gc
            .tables
            .get(t_ref)
            .map(|t| t.get_int(0))
            .and_then(|v| {
                if let Val::Num(n) = v {
                    Some(n as u64)
                } else {
                    None
                }
            })
            .unwrap_or(0),
        _ => 0,
    };
    let s = format!("LuaDurationObject: 0x{:016x}", id);
    let val = create_string_static_owned(state, &s);
    state.push(val);
    Ok(1)
}

// ── Method bodies ─────────────────────────────────────────────────────────────

fn require_duration(state: &mut LuaState, index: i32) -> LuaResult<Val> {
    let object = crate::lua_bridge::stack_val(state, index);
    let expected = registry_get(state, MT_KEY);
    if let (Val::Table(reference), Val::Table(metatable)) = (object, expected) {
        let actual = state
            .gc
            .tables
            .get(reference)
            .and_then(|table| table.metatable());
        if actual == Some(metatable) {
            return Ok(object);
        }
    }
    Err(rilua::runtime_error(format!(
        "expected LuaDurationObject at argument {index}"
    )))
}

fn m_assign(state: &mut LuaState) -> LuaResult<u32> {
    let target = require_duration(state, 1)?;
    let source = require_duration(state, 2)?;
    core::assign_state(state, source, target)?;
    Ok(0)
}

fn m_copy(state: &mut LuaState) -> LuaResult<u32> {
    let source = require_duration(state, 1)?;
    let obj = new_duration_object(state);
    state.push(obj);
    core::copy_state(state, source, obj)?;
    Ok(1)
}

fn m_get_clock(state: &mut LuaState) -> LuaResult<u32> {
    let object = crate::lua_bridge::stack_val(state, 1);
    let clock = table_get(state, object, "clock");
    state.push(clock);
    Ok(1)
}

fn m_set_clock(state: &mut LuaState) -> LuaResult<u32> {
    let object = crate::lua_bridge::stack_val(state, 1);
    core::require_secret_access(state, object)?;
    let clock = crate::lua_bridge::stack_val(state, 2);
    table_set(state, object, "clock", clock);
    Ok(0)
}

/// Inspect authenticated timing wrappers without exposing their payloads.
pub(crate) fn duration_has_secret_values(state: &LuaState, value: Val) -> bool {
    core::has_secret_values(state, value)
}

fn m_has_secret_values(state: &mut LuaState) -> LuaResult<u32> {
    let object = crate::lua_bridge::stack_val(state, 1);
    state.push(Val::Bool(duration_has_secret_values(state, object)));
    Ok(1)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Returns true if `key` is a method or metamethod name (write-protected).
fn is_readonly_key(key: &str) -> bool {
    METHOD_NAMES.contains(&key) || META_NAMES.contains(&key)
}

/// Allocate a new duration proxy table, assign the shared metatable, and
/// store the instance ID at integer key 0.
fn new_duration_object(state: &mut LuaState) -> Val {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let obj_ref = state.gc.alloc_table(Table::new());

    // Store instance ID at integer key 0 (invisible to string-keyed __newindex).
    if let Some(t) = state.gc.tables.get_mut(obj_ref) {
        let _ = t.raw_set(Val::Num(0.0), Val::Num(id as f64), &state.gc.string_arena);
    }

    // Attach shared metatable.
    let mt = registry_get(state, MT_KEY);
    if let (Val::Table(_), Val::Table(mt_ref)) = (Val::Table(obj_ref), mt) {
        if let Some(t) = state.gc.tables.get_mut(obj_ref) {
            t.set_metatable(Some(mt_ref));
        }
    }

    state.gc.barrier_back(obj_ref);
    let object = Val::Table(obj_ref);
    core::initialize(state, object);
    object
}

/// Build (or retrieve from the registry) the shared metatable.
///
/// The metatable has:
/// - `__index`    → Rust fn (instance raw → methods fallback)
/// - `__newindex` → Rust fn (blocks method-name writes)
/// - `__tostring` → Rust fn (formats with instance ID)
/// - `__metatable` = false  (hides the metatable from Lua `getmetatable`)
fn ensure_metatable(state: &mut LuaState) {
    // Already built in a previous call.
    if let Val::Table(_) = registry_get(state, MT_KEY) {
        return;
    }

    // Build the shared methods table.
    let methods = build_methods_table(state);
    registry_set(state, METHODS_KEY, methods);

    // Build the metatable.
    let mt = create_table(state);

    let index_fn = make_closure(state, "__duration_index", duration_index);
    table_set_static(state, mt, "__index", index_fn);

    let newindex_fn = make_closure(state, "__duration_newindex", duration_newindex);
    table_set_static(state, mt, "__newindex", newindex_fn);

    let tostring_fn = make_closure(state, "__duration_tostring", duration_tostring);
    table_set_static(state, mt, "__tostring", tostring_fn);

    // __metatable = false hides the metatable (getmetatable returns false).
    table_set_static(state, mt, "__metatable", Val::Bool(false));

    registry_set(state, MT_KEY, mt);
}

/// Build the shared methods table installed as the fallback for `__index`.
fn build_methods_table(state: &mut LuaState) -> Val {
    let methods = create_table(state);
    install_lifecycle_methods(state, methods);
    install_query_methods(state, methods);
    core::register(state, methods);
    methods
}

/// Install Assign and Copy — methods that write or clone the duration object.
fn install_lifecycle_methods(state: &mut LuaState, methods: Val) {
    install_method(
        state,
        methods,
        "Assign",
        "LuaDurationObject.Assign",
        m_assign,
    );
    install_method(state, methods, "Copy", "LuaDurationObject.Copy", m_copy);
    install_method(
        state,
        methods,
        "SetClock",
        "LuaDurationObject.SetClock",
        m_set_clock,
    );
}

/// Install duration query methods with local best-effort state.
fn install_query_methods(state: &mut LuaState, methods: Val) {
    install_method(
        state,
        methods,
        "GetClock",
        "LuaDurationObject.GetClock",
        m_get_clock,
    );
    install_method(
        state,
        methods,
        "HasSecretValues",
        "LuaDurationObject.HasSecretValues",
        m_has_secret_values,
    );
}

fn install_method(
    state: &mut LuaState,
    methods: Val,
    key: &'static str,
    closure_name: &'static str,
    func: rilua::RustFn,
) {
    let closure = make_closure(state, closure_name, func);
    table_set_static(state, methods, key, closure);
}

/// Wrap a `RustFn` in a `Closure::Rust` and return it as a `Val::Function`.
fn make_closure(state: &mut LuaState, name: &'static str, func: rilua::RustFn) -> Val {
    use rilua::vm::closure::{Closure, RustClosure};
    let closure = Closure::Rust(RustClosure::new(func, name));
    Val::Function(state.gc.alloc_closure(closure))
}

/// Intern an owned string and return it as a `Val::Str`.
///
/// Unlike `create_string_static`, this handles runtime-built strings such as
/// the `__tostring` output.
fn create_string_static_owned(state: &mut LuaState, s: &str) -> Val {
    let r = state.gc.intern_string(s.as_bytes());
    Val::Str(r)
}
