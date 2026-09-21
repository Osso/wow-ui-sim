//! rilua-side script dispatch helpers for Phase 3 migration.
//!
//! Mirrors `script_helpers.rs` but operates on `rilua::Lua` / `LuaState`
//! instead of `mlua::Lua`. Script handlers are stored in registry tables
//! using the same key format (`{widget_id}_{handler_name}`).

use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::string::LuaString;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, Val};

use crate::lua_api::methods::{call_function_state, val_to_string};

mod event_dispatch;
mod on_update_cache;
mod source_metadata;
#[cfg(test)]
mod tests;

pub(crate) use event_dispatch::{dispatch_named_event_handlers, event_matches_unit_filter};
pub use event_dispatch::{dispatch_on_update, fire_named_event_state, get_event_listeners};
pub use on_update_cache::reconcile_on_update_runtime_cache_if_dirty;
pub use source_metadata::get_script_source_binding;
use source_metadata::{remove_script_source_binding, set_script_source_binding};

// ── Registry helpers ────────────────────────────────────────────────

const SCRIPTS_PRECALL_KEY: &str = "__scripts_pre";
const SCRIPTS_KEY: &str = "__scripts";
const SCRIPTS_POSTCALL_KEY: &str = "__scripts_post";
pub(super) const ON_UPDATE_SCRIPTS_KEY: &str = "__on_update_scripts";
pub(super) const ON_POST_UPDATE_SCRIPTS_KEY: &str = "__on_post_update_scripts";
const ERROR_HANDLER_KEY: &str = "__error_handler";
const PROTECTED_LUA_PCALL_WRAPPER_FACTORY_KEY: &str = "__protected_lua_pcall_wrapper_factory";
const LUA_MULTRET: i32 = -1;
const SCRIPT_HANDLER_TABLE_HASH_SLOTS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptBinding {
    Precall,
    Normal,
    Postcall,
}

impl ScriptBinding {
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Precall),
            1 => Some(Self::Normal),
            2 => Some(Self::Postcall),
            _ => None,
        }
    }

    fn registry_key(self) -> &'static str {
        match self {
            Self::Precall => SCRIPTS_PRECALL_KEY,
            Self::Normal => SCRIPTS_KEY,
            Self::Postcall => SCRIPTS_POSTCALL_KEY,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptDispatchHandler {
    pub binding: ScriptBinding,
    pub handler: Val,
}

/// Get a named table from rilua's registry, returning None if absent.
///
pub fn registry_table(state: &mut LuaState, key: &'static str) -> Option<GcRef<Table>> {
    let key_ref = registry_key_ref(state, key);
    let registry = state.gc.tables.get(state.registry)?;
    match registry.get_str(key_ref, &state.gc.string_arena) {
        Val::Table(t) => Some(t),
        _ => None,
    }
}

/// Get or create a named table in rilua's registry.
pub fn registry_table_or_create(state: &mut LuaState, key: &'static str) -> GcRef<Table> {
    if let Some(existing) = registry_table(state, key) {
        return existing;
    }
    let new_table = state.gc.alloc_table(Table::new());
    let key_ref = registry_key_ref(state, key);
    let registry = state.registry;
    if let Some(reg) = state.gc.tables.get_mut(registry) {
        let _ = reg.raw_set(
            Val::Str(key_ref),
            Val::Table(new_table),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(registry);
    new_table
}

fn registry_key_ref(state: &mut LuaState, key: &'static str) -> GcRef<LuaString> {
    state.gc.intern_string_static(key.as_bytes())
}

/// Set a string-keyed value in a table.
pub fn table_set_str(state: &mut LuaState, table: GcRef<Table>, key: &str, value: Val) {
    let key_ref = state.gc.intern_string(key.as_bytes());
    table_set_str_ref(state, table, key_ref, value);
}

fn table_set_str_ref(
    state: &mut LuaState,
    table: GcRef<Table>,
    key_ref: GcRef<LuaString>,
    value: Val,
) {
    if let Some(t) = state.gc.tables.get_mut(table) {
        let _ = t.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table);
}

/// Get a string-keyed value from a table.
pub fn table_get_str(state: &mut LuaState, table: GcRef<Table>, key: &str) -> Val {
    let key_ref = state.gc.intern_string(key.as_bytes());
    table_get_str_ref(state, table, key_ref)
}

fn table_get_str_ref(state: &LuaState, table: GcRef<Table>, key_ref: GcRef<LuaString>) -> Val {
    state
        .gc
        .tables
        .get(table)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

// ── Script storage ──────────────────────────────────────────────────

/// Get a script handler for a given frame + handler name.
pub fn get_script(state: &mut LuaState, widget_id: u64, handler_name: &str) -> Option<Val> {
    get_script_binding(state, widget_id, handler_name, ScriptBinding::Normal)
}

pub fn get_script_binding(
    state: &mut LuaState,
    widget_id: u64,
    handler_name: &str,
    binding: ScriptBinding,
) -> Option<Val> {
    let handler_key = script_handler_key_ref(state, handler_name);
    get_script_binding_ref(state, widget_id, handler_key, binding)
}

fn get_script_binding_ref(
    state: &mut LuaState,
    widget_id: u64,
    handler_key: GcRef<LuaString>,
    binding: ScriptBinding,
) -> Option<Val> {
    let scripts = registry_table(state, binding.registry_key())?;
    let handlers = script_frame_table(state, scripts, widget_id, false)?;
    match table_get_str_ref(state, handlers, handler_key) {
        Val::Nil => None,
        val => Some(val),
    }
}

/// Set a script handler for a given frame + handler name.
pub fn set_script(state: &mut LuaState, widget_id: u64, handler_name: &str, func: Val) {
    set_script_binding(state, widget_id, handler_name, ScriptBinding::Normal, func);
}

pub fn set_script_binding(
    state: &mut LuaState,
    widget_id: u64,
    handler_name: &str,
    binding: ScriptBinding,
    func: Val,
) {
    // Fast-installed script handlers are often freshly-allocated closures.
    // Root them on the Lua stack before any key interning/allocation below,
    // otherwise a GC step in that gap can invalidate the closure ref before
    // it ever reaches the __scripts registry table.
    let stack_slot = state.top;
    state.ensure_stack(stack_slot + 1);
    state.stack_set(stack_slot, func);
    state.top = stack_slot + 1;

    let scripts = registry_table_or_create(state, binding.registry_key());
    let handlers =
        script_frame_table(state, scripts, widget_id, true).expect("created handler table");
    let handler_key = script_handler_key_ref(state, handler_name);
    table_set_str_ref(state, handlers, handler_key, func);
    set_script_source_binding(state, widget_id, handler_name, binding, func);
    sync_on_update_cache(state, widget_id, handler_name);

    state.top = stack_slot;
}

/// Remove a script handler.
pub fn remove_script(state: &mut LuaState, widget_id: u64, handler_name: &str) {
    remove_script_binding(state, widget_id, handler_name, ScriptBinding::Normal);
}

pub fn remove_all_scripts(state: &mut LuaState, widget_id: u64) {
    for binding in [
        ScriptBinding::Precall,
        ScriptBinding::Normal,
        ScriptBinding::Postcall,
    ] {
        if let Some(scripts) = registry_table(state, binding.registry_key())
            && let Some(table) = state.gc.tables.get_mut(scripts)
        {
            let _ = table.raw_set(Val::Num(widget_id as f64), Val::Nil, &state.gc.string_arena);
            state.gc.barrier_back(scripts);
        }
    }
    sync_on_update_cache(state, widget_id, "OnUpdate");
    sync_on_update_cache(state, widget_id, "OnPostUpdate");
}

pub fn remove_script_binding(
    state: &mut LuaState,
    widget_id: u64,
    handler_name: &str,
    binding: ScriptBinding,
) {
    if let Some(scripts) = registry_table(state, binding.registry_key()) {
        if let Some(handlers) = script_frame_table(state, scripts, widget_id, false) {
            let handler_key = script_handler_key_ref(state, handler_name);
            table_set_str_ref(state, handlers, handler_key, Val::Nil);
        }
    }
    remove_script_source_binding(state, widget_id, handler_name, binding);
    sync_on_update_cache(state, widget_id, handler_name);
}

fn script_frame_table(
    state: &mut LuaState,
    scripts: GcRef<Table>,
    widget_id: u64,
    create: bool,
) -> Option<GcRef<Table>> {
    let key = widget_id as i64;
    if let Some(table) = state.gc.tables.get(scripts)
        && let Val::Table(existing) = table.get_int(key)
    {
        return Some(existing);
    }

    if !create {
        return None;
    }

    let handlers = state
        .gc
        .alloc_table(Table::with_sizes(0, SCRIPT_HANDLER_TABLE_HASH_SLOTS));
    if let Some(table) = state.gc.tables.get_mut(scripts) {
        let _ = table.raw_set(
            Val::Num(widget_id as f64),
            Val::Table(handlers),
            &state.gc.string_arena,
        );
    }
    state.gc.barrier_back(scripts);
    Some(handlers)
}

pub fn get_scripts_for_dispatch(
    state: &mut LuaState,
    widget_id: u64,
    handler_name: &str,
) -> Vec<Val> {
    get_script_handlers_for_dispatch(state, widget_id, handler_name)
        .into_iter()
        .map(|handler| handler.handler)
        .collect()
}

pub fn get_script_handlers_for_dispatch(
    state: &mut LuaState,
    widget_id: u64,
    handler_name: &str,
) -> Vec<ScriptDispatchHandler> {
    let handler_key = script_handler_key_ref(state, handler_name);
    let mut scripts = Vec::with_capacity(3);
    for binding in [
        ScriptBinding::Precall,
        ScriptBinding::Normal,
        ScriptBinding::Postcall,
    ] {
        if let Some(script) = get_script_binding_ref(state, widget_id, handler_key, binding) {
            scripts.push(ScriptDispatchHandler {
                binding,
                handler: script,
            });
        }
    }
    scripts
}

fn sync_on_update_cache(state: &mut LuaState, widget_id: u64, handler_name: &str) {
    let cache_key = match handler_name {
        "OnUpdate" => ON_UPDATE_SCRIPTS_KEY,
        "OnPostUpdate" => ON_POST_UPDATE_SCRIPTS_KEY,
        _ => return,
    };
    let handler_key = script_handler_key_ref(state, handler_name);
    let value = if any_script_binding_present_ref(state, widget_id, handler_key) {
        Val::Bool(true)
    } else {
        Val::Nil
    };
    let table_ref = registry_table_or_create(state, cache_key);
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(widget_id as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
    on_update_cache::sync_on_update_runtime_cache(state, widget_id);
}

fn any_script_binding_present_ref(
    state: &mut LuaState,
    widget_id: u64,
    handler_key: GcRef<LuaString>,
) -> bool {
    [
        ScriptBinding::Precall,
        ScriptBinding::Normal,
        ScriptBinding::Postcall,
    ]
    .into_iter()
    .any(|binding| get_script_binding_ref(state, widget_id, handler_key, binding).is_some())
}

fn script_handler_key_ref(state: &mut LuaState, handler_name: &str) -> GcRef<LuaString> {
    match static_script_handler_name(handler_name) {
        Some(static_name) => state.gc.intern_string_static(static_name),
        None => state.gc.intern_string(handler_name.as_bytes()),
    }
}

fn static_script_handler_name(handler_name: &str) -> Option<&'static [u8]> {
    static_mouse_script_handler_name(handler_name)
        .or_else(|| static_input_script_handler_name(handler_name))
        .or_else(|| static_animation_script_handler_name(handler_name))
        .or_else(|| static_lifecycle_script_handler_name(handler_name))
}

fn static_mouse_script_handler_name(handler_name: &str) -> Option<&'static [u8]> {
    match handler_name {
        "OnClick" => Some(b"OnClick"),
        "OnDoubleClick" => Some(b"OnDoubleClick"),
        "PreClick" => Some(b"PreClick"),
        "PostClick" => Some(b"PostClick"),
        "OnEnter" => Some(b"OnEnter"),
        "OnLeave" => Some(b"OnLeave"),
        "OnMouseDown" => Some(b"OnMouseDown"),
        "OnMouseUp" => Some(b"OnMouseUp"),
        "OnMouseWheel" => Some(b"OnMouseWheel"),
        "OnDragStart" => Some(b"OnDragStart"),
        "OnDragStop" => Some(b"OnDragStop"),
        "OnReceiveDrag" => Some(b"OnReceiveDrag"),
        _ => None,
    }
}

fn static_input_script_handler_name(handler_name: &str) -> Option<&'static [u8]> {
    match handler_name {
        "OnEnterPressed" => Some(b"OnEnterPressed"),
        "OnEscapePressed" => Some(b"OnEscapePressed"),
        "OnTabPressed" => Some(b"OnTabPressed"),
        "OnSpacePressed" => Some(b"OnSpacePressed"),
        "OnArrowPressed" => Some(b"OnArrowPressed"),
        "OnTextChanged" => Some(b"OnTextChanged"),
        "OnTextSet" => Some(b"OnTextSet"),
        "OnChar" => Some(b"OnChar"),
        "OnEditFocusGained" => Some(b"OnEditFocusGained"),
        "OnEditFocusLost" => Some(b"OnEditFocusLost"),
        "OnInputLanguageChanged" => Some(b"OnInputLanguageChanged"),
        "OnKeyDown" => Some(b"OnKeyDown"),
        "OnKeyUp" => Some(b"OnKeyUp"),
        "OnGamePadStick" | "OnGamepadStick" => Some(b"OnGamePadStick"),
        "OnValueChanged" => Some(b"OnValueChanged"),
        "OnHyperlinkClick" => Some(b"OnHyperlinkClick"),
        "OnHyperlinkEnter" => Some(b"OnHyperlinkEnter"),
        "OnHyperlinkLeave" => Some(b"OnHyperlinkLeave"),
        _ => None,
    }
}

fn static_animation_script_handler_name(handler_name: &str) -> Option<&'static [u8]> {
    match handler_name {
        "OnPlay" => Some(b"OnPlay"),
        "OnFinished" => Some(b"OnFinished"),
        "OnStop" => Some(b"OnStop"),
        "OnLoop" => Some(b"OnLoop"),
        "OnPause" => Some(b"OnPause"),
        _ => None,
    }
}

fn static_lifecycle_script_handler_name(handler_name: &str) -> Option<&'static [u8]> {
    match handler_name {
        "OnLoad" => Some(b"OnLoad"),
        "OnEvent" => Some(b"OnEvent"),
        "OnUpdate" => Some(b"OnUpdate"),
        "OnPostUpdate" => Some(b"OnPostUpdate"),
        "OnShow" => Some(b"OnShow"),
        "OnHide" => Some(b"OnHide"),
        "OnEnable" => Some(b"OnEnable"),
        "OnDisable" => Some(b"OnDisable"),
        "OnSizeChanged" => Some(b"OnSizeChanged"),
        "OnAttributeChanged" => Some(b"OnAttributeChanged"),
        _ => None,
    }
}

// ── Error handler ───────────────────────────────────────────────────

/// Call the WoW error handler and log to stderr.
pub fn call_error_handler(lua: &mut rilua::Lua, error_msg: &str) {
    call_error_handler_state(lua.state_mut(), error_msg);
}

/// State-only variant for RustFn call sites that only hold `&mut LuaState`.
pub fn call_error_handler_state(state: &mut LuaState, error_msg: &str) {
    let collected_error = augment_error_with_traceback(state, error_msg);
    let _ = sink_lua_error(
        state,
        &collected_error,
        LuaErrorEmitPolicy::FirstOccurrenceOnly,
    );
    let Ok(handler) = ensure_error_handler(state) else {
        return;
    };
    let Val::Function(_) = handler else {
        return;
    };
    let msg_ref = state.gc.intern_string(collected_error.as_bytes());
    let _ = protected_call_state_with_policy(
        state,
        handler,
        &[Val::Str(msg_ref)],
        LuaErrorEmitPolicy::FirstOccurrenceOnly,
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LuaErrorEmitPolicy {
    /// Do not record or mirror the error.
    Never,
    /// Mirror to stderr + GUI console only for the first normalized occurrence.
    FirstOccurrenceOnly,
    /// Mirror every occurrence to stderr + GUI console.
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LuaErrorSinkResult {
    is_first: bool,
}

/// Canonical Lua error sink.
///
/// This is the single entry point that records a Lua error into simulator state
/// (raw message + addon attribution + normalized counts) and optionally mirrors
/// it to stderr/GUI console using a single normalized-occurrence policy.
fn sink_lua_error(
    state: &LuaState,
    error_msg: &str,
    emit_policy: LuaErrorEmitPolicy,
) -> LuaErrorSinkResult {
    use super::env::WowLuaAppData;

    let Some(app) = state.app_data::<WowLuaAppData>() else {
        return LuaErrorSinkResult { is_first: false };
    };
    let Ok(mut sim) = app.sim_state.try_borrow_mut() else {
        return LuaErrorSinkResult { is_first: false };
    };

    sim.lua_errors.push(error_msg.to_string());
    let addon_name = sim
        .executing_addon_index
        .or(sim.loading_addon_index)
        .and_then(|idx| sim.addons.get(idx as usize))
        .map(|addon| addon.folder_name.clone());
    sim.lua_error_records
        .push(crate::lua_api::state::LuaErrorRecord {
            message: error_msg.to_string(),
            addon_name,
        });

    let normalized = crate::lua_errors::extract_error_message(error_msg);
    let entry = sim.lua_error_counts.entry(normalized).or_insert(0);
    let is_first = *entry == 0;
    *entry += 1;

    let should_emit = match emit_policy {
        LuaErrorEmitPolicy::Never => false,
        LuaErrorEmitPolicy::FirstOccurrenceOnly => is_first,
        LuaErrorEmitPolicy::Always => true,
    };
    if should_emit {
        eprintln!("Lua error: {error_msg}");
        sim.console_output.push(format!("Lua error: {error_msg}"));
    }

    LuaErrorSinkResult { is_first }
}

/// Route `addframetext(...)` output through the canonical Lua error sink.
///
/// This mirrors every emitted message to stderr and GUI console while also
/// recording raw messages, normalized counts, and addon-attributed records.
pub fn report_addframetext_error(state: &LuaState, message: &str) {
    let _ = sink_lua_error(state, message, LuaErrorEmitPolicy::Always);
}

fn augment_error_with_traceback(state: &mut LuaState, error_msg: &str) -> String {
    if error_msg.contains("stack traceback:") {
        return error_msg.to_string();
    }

    let Ok(func) = state.load("return debug and debug.traceback and debug.traceback() or ''")
    else {
        return error_msg.to_string();
    };
    let Ok(results) = protected_call_state(state, Val::Function(func.gc_ref()), &[]) else {
        return error_msg.to_string();
    };
    let Some(traceback) = results
        .into_iter()
        .next()
        .and_then(|value| val_to_string(state, value))
    else {
        return error_msg.to_string();
    };
    let traceback = traceback.trim();
    if traceback.is_empty() {
        return error_msg.to_string();
    }

    format!("{error_msg}\n{traceback}")
}

pub fn protected_call_state(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
) -> Result<Vec<Val>, Val> {
    protected_call_state_with_policy(state, func, args, LuaErrorEmitPolicy::Never)
}

fn protected_call_state_with_policy(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
    emit_policy: LuaErrorEmitPolicy,
) -> Result<Vec<Val>, Val> {
    let saved_top = state.top;
    let call_base = saved_top;
    let saved_ci = state.ci;
    let saved_n_ccalls = state.n_ccalls;
    let saved_call_depth = state.call_depth;

    prepare_protected_call_stack(state, call_base, func, args);

    let call_result = state.call_function(call_base, LUA_MULTRET);
    match call_result {
        Ok(()) => {
            let results = (call_base..state.top)
                .map(|idx| state.stack_get(idx))
                .collect();
            state.top = saved_top;
            Ok(results)
        }
        Err(err) => {
            let error_val = recover_call_state(
                state,
                saved_ci,
                saved_n_ccalls,
                saved_call_depth,
                call_base,
                err,
            );
            state.top = saved_top;
            record_protected_call_error(state, error_val, emit_policy);
            Err(error_val)
        }
    }
}

fn prepare_protected_call_stack(state: &mut LuaState, call_base: usize, func: Val, args: &[Val]) {
    state.error_object = None;
    state.ensure_stack(call_base + 1 + args.len());
    state.stack_set(call_base, func);
    for (index, arg) in args.iter().copied().enumerate() {
        state.stack_set(call_base + 1 + index, arg);
    }
    state.top = call_base + 1 + args.len();
}

fn record_protected_call_error(state: &LuaState, error_val: Val, emit_policy: LuaErrorEmitPolicy) {
    if emit_policy == LuaErrorEmitPolicy::Never {
        return;
    }
    if let Some(error_msg) = val_to_string(state, error_val) {
        let _ = sink_lua_error(state, &error_msg, emit_policy);
    }
}

fn recover_call_state(
    state: &mut LuaState,
    saved_ci: usize,
    saved_n_ccalls: u16,
    saved_call_depth: u16,
    call_base: usize,
    err: rilua::LuaError,
) -> Val {
    state.ci = saved_ci;
    state.base = state.call_stack[state.ci].base;
    state.n_ccalls = saved_n_ccalls;
    state.call_depth = saved_call_depth;
    if state.ci < rilua::vm::state::MAXCALLS {
        state.ci_overflow = false;
    }
    state.close_upvalues(call_base);
    state.error_object.take().unwrap_or_else(|| {
        let r = state.gc.intern_string(err.to_string().as_bytes());
        Val::Str(r)
    })
}

/// Call a function through Lua's protected `xpcall` path.
///
/// A cached bridge captures `debug.traceback` before the failing Lua stack
/// unwinds.
pub fn protected_lua_pcall_state(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
) -> Result<Vec<Val>, String> {
    let bridge =
        ensure_protected_lua_pcall_wrapper_factory(state).map_err(|error| error.to_string())?;
    let mut protected_args = Vec::with_capacity(args.len() + 1);
    protected_args.push(func);
    protected_args.extend_from_slice(args);
    let results = protected_call_state(state, Val::Function(bridge.gc_ref()), &protected_args)
        .map_err(|error| {
            val_to_string(state, error)
                .unwrap_or_else(|| format!("script error ({})", error.type_name()))
        })?;
    match results.first().copied() {
        Some(Val::Bool(true)) => Ok(results.into_iter().skip(1).collect()),
        Some(Val::Bool(false)) => {
            let error = results
                .get(1)
                .copied()
                .and_then(|value| val_to_string(state, value))
                .unwrap_or_else(|| "script error".to_string());
            Err(error)
        }
        _ => Ok(results),
    }
}

pub fn call_void_function_state(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
) -> Result<(), String> {
    call_function_state(state, func, args)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn ensure_protected_lua_pcall_wrapper_factory(
    state: &mut LuaState,
) -> rilua::LuaResult<rilua::Function> {
    let existing = registry_value(state, PROTECTED_LUA_PCALL_WRAPPER_FACTORY_KEY);
    if let Val::Function(func_ref) = existing {
        return Ok(rilua::Function::from_gc_ref(func_ref));
    }

    let factory = state.load(
        r##"
        local func = ...
        local argc = select("#", ...) - 1
        local args = { select(2, ...) }
        local handler = debug and debug.traceback or function(err) return tostring(err) end
        return xpcall(function()
            return func(unpack(args, 1, argc))
        end, handler)
    "##,
    )?;
    set_registry_value(
        state,
        PROTECTED_LUA_PCALL_WRAPPER_FACTORY_KEY,
        Val::Function(factory.gc_ref()),
    );
    Ok(factory)
}

fn ensure_error_handler(state: &mut LuaState) -> rilua::LuaResult<Val> {
    let existing = registry_value(state, ERROR_HANDLER_KEY);
    if existing != Val::Nil {
        return Ok(existing);
    }

    let func = state.load("return function(_msg) end")?;
    let call_base = state.top;
    state.ensure_stack(call_base + 2);
    state.stack_set(call_base, Val::Function(func.gc_ref()));
    state.top = call_base + 1;
    state.call_function(call_base, 1)?;
    let handler = state.stack_get(call_base);
    state.top = call_base;
    set_registry_value(state, ERROR_HANDLER_KEY, handler);
    Ok(handler)
}

pub(crate) fn registry_value(state: &mut LuaState, key: &'static str) -> Val {
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    state
        .gc
        .tables
        .get(state.registry)
        .map(|table| table.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

pub(crate) fn set_registry_value(state: &mut LuaState, key: &'static str, value: Val) {
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    let registry = state.registry;
    if let Some(table) = state.gc.tables.get_mut(registry) {
        let _ = table.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(registry);
}

/// Collect a Lua error into SimState for later retrieval.
pub fn collect_lua_error(state: &LuaState, msg: &str) -> bool {
    sink_lua_error(state, msg, LuaErrorEmitPolicy::Never).is_first
}
