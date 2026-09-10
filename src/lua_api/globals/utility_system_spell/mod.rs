//! rilua RustFn equivalents of globals from utility_api, system_api, and spell_api.
//!
//! Each `pub fn` matches the `RustFn` signature:
//!   `fn(state: &mut LuaState) -> LuaResult<u32>`
//!
//! Arguments are extracted with `stack_val(state, n)` (1-based).
//! Return values are pushed with `state.push(val)` and counted in the return.
//!
//! Complex operations (pcall, xpcall, securecall) are stubbed with TODO.

mod c_addon_profiler;
mod c_addons;
mod spell_api;
mod table_util;

use crate::lua_api::methods::call_function_state_multi;
use crate::lua_api::script_helpers::{
    call_error_handler_state, protected_call_state, registry_value, set_registry_value,
};
use crate::lua_bridge::stack_val;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const BYTE_LOOKUP_SIZE: usize = 256;
const XPCALL_WRAPPER_FACTORY_LUA: &str = r##"
    local native_xpcall = ...
    local unpack_args = unpack
    local select_args = select
    return function(func, handler, ...)
        local argc = select_args("#", ...)
        local args = { ... }
        return native_xpcall(function()
            return func(unpack_args(args, 1, argc))
        end, function(error)
            local ok, handled = native_xpcall(
                function() return handler(error) end,
                function() return nil end
            )
            if ok then
                return handled
            end
            return error
        end)
    end
"##;

// ── Utility API ─────────────────────────────────────────────────────────────

/// wipe(t) — clear all entries from a table and return it.
pub fn wipe(state: &mut LuaState) -> LuaResult<u32> {
    let t = stack_val(state, 1);
    let Val::Table(table_ref) = t else {
        state.push(t);
        return Ok(1);
    };

    super::real::table_freeze::ensure_mutable(state, table_ref)?;
    let mut keys = Vec::new();
    if let Some(table) = state.gc.tables.get(table_ref) {
        let mut key = Val::Nil;
        while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
            keys.push(next_key);
            key = next_key;
        }
    }

    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        for key in keys {
            let _ = table.raw_set(key, Val::Nil, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(table_ref);

    state.push(t);
    Ok(1)
}

/// tinsert(t [, pos], value) — append or insert a value into an array table.
pub fn tinsert(state: &mut LuaState) -> LuaResult<u32> {
    super::real::table_freeze::insert(state)
}

/// tremove(t [, pos]) — remove and return a value from an array table.
pub fn tremove(state: &mut LuaState) -> LuaResult<u32> {
    super::real::table_freeze::remove(state)
}

/// tContains(t, value) — return true if value is present in the table.
///
/// Blizzard's `TableUtil.lua` implementation scans with `pairs(tbl)`, not
/// `ipairs(tbl)`, so hash-only / mixed tables must participate too.
pub fn t_contains(state: &mut LuaState) -> LuaResult<u32> {
    let table = stack_val(state, 1);
    let needle = stack_val(state, 2);
    let found = match table {
        Val::Table(table_ref) => state
            .gc
            .tables
            .get(table_ref)
            .map(|table| table_contains_value(table, needle, state))
            .transpose()?
            .unwrap_or(false),
        _ => false,
    };
    state.push(Val::Bool(found));
    Ok(1)
}

fn table_contains_value(
    table: &rilua::vm::table::Table,
    needle: Val,
    state: &LuaState,
) -> LuaResult<bool> {
    let mut key = Val::Nil;
    while let Some((next_key, value)) = table.next(key, &state.gc.string_arena)? {
        if value == needle {
            return Ok(true);
        }
        key = next_key;
    }
    Ok(false)
}

/// tIndexOf(t, value) — return the integer index of value in t, or nil.
pub fn t_index_of(state: &mut LuaState) -> LuaResult<u32> {
    let table = stack_val(state, 1);
    let needle = stack_val(state, 2);
    let index = match table {
        Val::Table(table_ref) => state.gc.tables.get(table_ref).and_then(|table| {
            table
                .array_slice()
                .iter()
                .position(|value| *value == needle)
                .map(|index| index + 1)
        }),
        _ => None,
    };
    match index {
        Some(index) => state.push(Val::Num(index as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

/// tFilter(t, predicate) — keep only values for which predicate returns true.
pub fn t_filter(state: &mut LuaState) -> LuaResult<u32> {
    let table = stack_val(state, 1);
    let predicate = stack_val(state, 2);
    let Val::Table(table_ref) = table else {
        return Ok(0);
    };
    let Some(table_values) = state
        .gc
        .tables
        .get(table_ref)
        .map(|table| table.array_slice().to_vec())
    else {
        return Ok(0);
    };

    let mut kept = Vec::new();
    for value in table_values {
        if matches!(value, Val::Nil) {
            continue;
        }
        let keep = match crate::lua_api::methods::call_function_state(state, predicate, &[value]) {
            Ok(result) => matches!(result, Val::Bool(true)),
            Err(_) => false,
        };
        if keep {
            kept.push(value);
        }
    }

    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let len = table.array_slice().len();
        for index in 1..=len {
            let _ = table.raw_set(Val::Num(index as f64), Val::Nil, &state.gc.string_arena);
        }
        for (index, value) in kept.into_iter().enumerate() {
            let _ = table.raw_set(Val::Num((index + 1) as f64), value, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(table_ref);
    state.push(table);
    Ok(1)
}

pub use table_util::t_invert;
pub use table_util::table_util_find_indexed_mismatch;

/// getglobal(name) — return the global named `name`.
pub fn getglobal(state: &mut LuaState) -> LuaResult<u32> {
    let name_val = stack_val(state, 1);
    let Val::Str(name_ref) = name_val else {
        return Err(runtime_error("getglobal: expected string argument"));
    };
    let name = {
        let lua_str = state
            .gc
            .string_arena
            .get(name_ref)
            .ok_or_else(|| runtime_error("getglobal: invalid string ref"))?;
        String::from_utf8(lua_str.data().to_vec())
            .map_err(|_| runtime_error("getglobal: non-UTF8 name"))?
    };
    let global = state.global;
    let key_ref = state.gc.intern_string(name.as_bytes());
    let val = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    state.push(val);
    Ok(1)
}

/// setglobal(name, value) — set the global named `name` to `value`.
pub fn setglobal(state: &mut LuaState) -> LuaResult<u32> {
    let name_val = stack_val(state, 1);
    let value = stack_val(state, 2);
    let Val::Str(name_ref) = name_val else {
        return Err(runtime_error(
            "setglobal: expected string as first argument",
        ));
    };
    let name = {
        let lua_str = state
            .gc
            .string_arena
            .get(name_ref)
            .ok_or_else(|| runtime_error("setglobal: invalid string ref"))?;
        String::from_utf8(lua_str.data().to_vec())
            .map_err(|_| runtime_error("setglobal: non-UTF8 name"))?
    };
    let global = state.global;
    let key_ref = state.gc.intern_string(name.as_bytes());
    if let Some(t) = state.gc.tables.get_mut(global) {
        let _ = t.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    Ok(0)
}

/// nop(...) — no-operation, discards all arguments.
pub fn nop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// `strsplit(delimiter, str [, limit])` — split `str` on any character in
/// `delimiter` and push each piece as a separate return value.
pub fn strsplit(state: &mut LuaState) -> LuaResult<u32> {
    let delim = val_to_string_bytes(state, stack_val(state, 1));
    let input = val_to_string_bytes(state, stack_val(state, 2));
    let limit = match stack_val(state, 3) {
        Val::Num(n) if n > 0.0 => Some(n as usize),
        _ => None,
    };
    let (Some(delim), Some(input)) = (delim, input) else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let pieces = split_on_delimiter_set(&input, &delim, limit);
    let count = pieces.len() as u32;
    for piece in pieces {
        let s = state.gc.intern_string(&piece);
        state.push(Val::Str(s));
    }
    Ok(count.max(1))
}

fn split_on_delimiter_set(input: &[u8], delim: &[u8], limit: Option<usize>) -> Vec<Vec<u8>> {
    let mut delim_lookup = [false; BYTE_LOOKUP_SIZE];
    for &byte in delim {
        delim_lookup[byte as usize] = true;
    }
    let mut pieces = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    let max_pieces = limit.unwrap_or(usize::MAX);
    for &byte in input {
        let is_delim = delim_lookup[byte as usize];
        let can_split = pieces.len() + 1 < max_pieces;
        if is_delim && can_split {
            pieces.push(std::mem::take(&mut current));
        } else {
            current.push(byte);
        }
    }
    pieces.push(current);
    pieces
}

/// `strjoin(delimiter, ...)` — concatenate the variadic string arguments
/// separated by `delimiter`.
pub fn strjoin(state: &mut LuaState) -> LuaResult<u32> {
    let delim = val_to_string_bytes(state, stack_val(state, 1)).unwrap_or_default();
    let nargs = (state.top as i32 - state.base as i32) as usize;
    let mut out: Vec<u8> = Vec::new();
    for index in 2..=nargs {
        if !out.is_empty() {
            out.extend_from_slice(&delim);
        }
        let slot = state.base + index - 1;
        if let Some(bytes) = val_to_string_bytes(state, state.stack_get(slot)) {
            out.extend_from_slice(&bytes);
        }
    }
    let joined = state.gc.intern_string(&out);
    state.push(Val::Str(joined));
    Ok(1)
}

fn val_to_string_bytes(state: &LuaState, val: Val) -> Option<Vec<u8>> {
    match val {
        Val::Str(s) => state.gc.string_arena.get(s).map(|s| s.data().to_vec()),
        Val::Num(n) => Some(format!("{n}").into_bytes()),
        Val::Bool(b) => Some(if b {
            b"true".to_vec()
        } else {
            b"false".to_vec()
        }),
        _ => None,
    }
}

// ── System API ───────────────────────────────────────────────────────────────

/// type(v) — return the Lua type name of v as a string.
pub fn type_fn(state: &mut LuaState) -> LuaResult<u32> {
    let val = stack_val(state, 1);
    let type_name: &str = match val {
        Val::Nil => "nil",
        Val::Bool(_) => "boolean",
        Val::Num(_) => "number",
        Val::Str(_) => "string",
        Val::Table(_) => "table",
        Val::Function(_) => "function",
        Val::Userdata(_) | Val::LightUserdata(_) | Val::Thread(_) => "userdata",
    };
    let s = state.gc.intern_string_static(type_name.as_bytes());
    state.push(Val::Str(s));
    Ok(1)
}

/// GetBuildOption(name) — return the value of a named client build option.
pub fn get_build_option(state: &mut LuaState) -> LuaResult<u32> {
    let is_restricted_aura_api = matches!(
        stack_val(state, 1),
        Val::Str(name)
            if state
                .gc
                .string_arena
                .get(name)
                .is_some_and(|name| name.data() == b"RestrictedAuraAPI")
    ) && cfg!(feature = "retail-12-1-0");

    state.push(if is_restricted_aura_api {
        Val::Bool(true)
    } else {
        Val::Nil
    });
    Ok(1)
}

/// IsPublicTestClient() — always false in the simulator.
pub fn is_public_test_client(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// IsBetaBuild() — always false in the simulator.
pub fn is_beta_build(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// IsPublicBuild() — always true in the simulator.
pub fn is_public_build(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

/// BNFeaturesEnabled() — always false (no Battle.net in sim).
pub fn bn_features_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// BNFeaturesEnabledAndConnected() — always false.
pub fn bn_features_enabled_and_connected(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// BNConnected() — always true (sim pretends connected).
pub fn bn_connected(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

/// IsGMClient() — always false.
pub fn is_gm_client(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

/// RegisterStaticConstants(t) — no-op stub.
pub fn register_static_constants(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

/// pcall(f, ...) — protected call.
pub fn pcall(state: &mut LuaState) -> LuaResult<u32> {
    let func = stack_val(state, 1);
    let args: Vec<Val> = ((state.base + 1)..state.top)
        .map(|index| state.stack_get(index))
        .collect();
    match protected_call_state(state, func, &args) {
        Ok(results) => {
            state.push(Val::Bool(true));
            for result in &results {
                state.push(*result);
            }
            Ok(1 + results.len() as u32)
        }
        Err(error) => {
            state.push(Val::Bool(false));
            state.push(error);
            Ok(2)
        }
    }
}

fn install_xpcall(lua: &mut rilua::Lua) -> LuaResult<()> {
    let native_xpcall = LuaApiMut::get_global_val(lua, "xpcall");
    if !matches!(native_xpcall, Val::Function(_)) {
        return Err(runtime_error("native xpcall missing"));
    }

    let factory = lua.state_mut().load(XPCALL_WRAPPER_FACTORY_LUA)?;
    let wrapper = call_function_state_multi(
        lua.state_mut(),
        Val::Function(factory.gc_ref()),
        &[native_xpcall],
    )?
    .into_iter()
    .next()
    .ok_or_else(|| runtime_error("xpcall wrapper factory returned no function"))?;
    LuaApiMut::set_global_val(lua, "xpcall", wrapper)
}

/// securecall(name_or_func, ...) — call a function by name in a secure context.
pub fn securecall(state: &mut LuaState) -> LuaResult<u32> {
    let func = resolve_securecall_function(state);

    if !matches!(func, Val::Function(_)) {
        state.push(Val::Nil);
        return Ok(1);
    }

    let args = collect_securecall_args(state);
    let results = call_function_with_secure_taint(state, func, &args);
    push_securecall_results(state, results)
}

fn resolve_securecall_function(state: &mut LuaState) -> Val {
    match stack_val(state, 1) {
        Val::Str(name_ref) => state
            .gc
            .tables
            .get(state.global)
            .map(|table| table.get_str(name_ref, &state.gc.string_arena))
            .unwrap_or(Val::Nil),
        value => value,
    }
}

fn collect_securecall_args(state: &LuaState) -> Vec<Val> {
    let arg_count = (state.top - state.base).saturating_sub(1);
    (0..arg_count)
        .map(|index| state.stack_get(state.base + 1 + index))
        .collect()
}

fn call_function_with_secure_taint(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
) -> Result<Vec<Val>, rilua::LuaError> {
    let saved_taints = clear_securecall_taint(state);
    let results = call_function_state_multi(state, func, args);
    restore_securecall_taint(state, saved_taints);
    results
}

fn push_securecall_results(
    state: &mut LuaState,
    results: Result<Vec<Val>, rilua::LuaError>,
) -> LuaResult<u32> {
    match results {
        Ok(results) if results.is_empty() => {
            state.push(Val::Nil);
            Ok(1)
        }
        Ok(results) => {
            let count = results.len() as u32;
            for value in results {
                state.push(value);
            }
            Ok(count)
        }
        Err(error) => {
            call_error_handler_state(state, &error.to_string());
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

fn clear_securecall_taint(state: &mut LuaState) -> Vec<Option<String>> {
    let active_depth = state.ci.saturating_add(1);
    let mut saved_taints = Vec::with_capacity(active_depth);
    for call_info in state.call_stack.iter_mut().take(active_depth) {
        saved_taints.push(call_info.taint.clone());
        call_info.taint = None;
    }
    saved_taints
}

fn restore_securecall_taint(state: &mut LuaState, saved_taints: Vec<Option<String>>) {
    for (call_info, taint) in state.call_stack.iter_mut().zip(saved_taints) {
        call_info.taint = taint;
    }
}

const ERROR_HANDLER_KEY: &str = "__error_handler";

const ASSERTSAFE_DEFAULT_MESSAGE: &str = "non-fatal assertion failed";
const ASSERTSAFE_PREFIX: &str = "assertsafe: ";

/// `assertsafe(cond, message)` — non-throwing assertion.
///
/// When `cond` is falsy, format the message and report it through the same
/// pipeline that records raised errors (`lua_error_records` + the active
/// error handler). When `cond` is truthy, this is a no-op. Either way the
/// function returns `cond` so callers can chain it like `assert`.
///
/// The reported message is prefixed with `"assertsafe: "` so it stays
/// distinguishable from real raised errors in `lua-errors` JSON output.
pub fn assertsafe(state: &mut LuaState) -> LuaResult<u32> {
    let cond = stack_val(state, 1);
    let is_truthy = !matches!(cond, Val::Nil | Val::Bool(false));
    if is_truthy {
        state.push(cond);
        return Ok(1);
    }
    let raw_message = stack_val(state, 2);
    let formatted = format_assertsafe_message(state, raw_message);
    let prefixed = format!("{ASSERTSAFE_PREFIX}{formatted}");
    call_error_handler_state(state, &prefixed);
    state.push(cond);
    Ok(1)
}

fn format_assertsafe_message(state: &mut LuaState, raw_message: Val) -> String {
    if let Val::Function(_) = raw_message {
        return invoke_message_callback(state, raw_message);
    }
    if matches!(raw_message, Val::Nil) {
        return ASSERTSAFE_DEFAULT_MESSAGE.to_string();
    }
    val_to_string_or_default(state, raw_message)
}

fn invoke_message_callback(state: &mut LuaState, callback: Val) -> String {
    let Ok(results) = call_function_state_multi(state, callback, &[]) else {
        return ASSERTSAFE_DEFAULT_MESSAGE.to_string();
    };
    let Some(first) = results.into_iter().next() else {
        return ASSERTSAFE_DEFAULT_MESSAGE.to_string();
    };
    val_to_string_or_default(state, first)
}

fn val_to_string_or_default(state: &LuaState, val: Val) -> String {
    val_to_string_bytes(state, val)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_else(|| ASSERTSAFE_DEFAULT_MESSAGE.to_string())
}

pub fn seterrorhandler(state: &mut LuaState) -> LuaResult<u32> {
    let handler = stack_val(state, 1);
    if !matches!(handler, Val::Function(_)) {
        return Err(runtime_error("seterrorhandler: expected function"));
    }
    let previous = registry_value(state, ERROR_HANDLER_KEY);
    set_registry_value(state, ERROR_HANDLER_KEY, handler);
    state.push(previous);
    Ok(1)
}

pub fn geterrorhandler(state: &mut LuaState) -> LuaResult<u32> {
    let handler = ensure_error_handler(state)?;
    state.push(handler);
    Ok(1)
}

fn ensure_error_handler(state: &mut LuaState) -> LuaResult<Val> {
    let existing = registry_value(state, ERROR_HANDLER_KEY);
    if existing != Val::Nil {
        return Ok(existing);
    }
    let default_handler = build_default_error_handler(state)?;
    set_registry_value(state, ERROR_HANDLER_KEY, default_handler);
    Ok(default_handler)
}

fn build_default_error_handler(state: &mut LuaState) -> LuaResult<Val> {
    let func = state.load("return function(_msg) end")?;
    let call_base = state.top;
    state.ensure_stack(call_base + 2);
    state.stack_set(call_base, Val::Function(func.gc_ref()));
    state.top = call_base + 1;
    state.call_function(call_base, 1)?;
    let result = state.stack_get(call_base);
    state.top = call_base;
    Ok(result)
}

// ── Registration ─────────────────────────────────────────────────────────────

/// Register all functions in this module as rilua globals.
///
/// Call this once after the rilua `Lua` instance is created.
pub fn register_all(lua: &mut rilua::Lua) -> rilua::LuaResult<()> {
    register_utility_globals(lua)?;
    register_system_globals(lua)?;
    spell_api::register_spell_globals(lua)?;
    crate::lua_api::globals::real::timerunning::register_all(lua)?;

    let state = lua.state_mut();
    register_system_tables(state)?;
    crate::c_api::register_utility_bootstrap_tables(state)?;
    c_addons::register_legacy_addon_globals(state)?;
    crate::lua_api::globals::real::container_legacy::register_legacy_container_globals(state)?;
    crate::lua_api::globals::real::item_legacy::register_legacy_item_globals(state)?;
    crate::lua_api::globals::real::spell_flyout_legacy::register_legacy_spell_flyout_globals(
        state,
    )?;
    crate::lua_api::globals::real::specialization_legacy::register_legacy_specialization_globals(
        state,
    )?;
    crate::lua_api::globals::real::ui_widget_container::register_widget_container_mixin(state)?;

    Ok(())
}

fn register_system_tables(state: &mut LuaState) -> LuaResult<()> {
    table_util::register_table_util(state)?;
    c_addons::register_c_addons(state)?;
    c_addon_profiler::register_c_addon_profiler(state)?;
    Ok(())
}

fn register_utility_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "wipe", wipe)?;
    LuaApiMut::register_function(lua, "tinsert", tinsert)?;
    LuaApiMut::register_function(lua, "tremove", tremove)?;
    LuaApiMut::register_function(lua, "tContains", t_contains)?;
    LuaApiMut::register_function(lua, "tIndexOf", t_index_of)?;
    LuaApiMut::register_function(lua, "tFilter", t_filter)?;
    LuaApiMut::register_function(lua, "tInvert", t_invert)?;
    LuaApiMut::register_function(lua, "getglobal", getglobal)?;
    LuaApiMut::register_function(lua, "setglobal", setglobal)?;
    LuaApiMut::register_function(lua, "nop", nop)?;
    LuaApiMut::register_function(lua, "strsplit", strsplit)?;
    LuaApiMut::register_function(lua, "strjoin", strjoin)?;
    Ok(())
}

fn register_system_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "type", type_fn)?;
    LuaApiMut::register_function(lua, "GetBuildOption", get_build_option)?;
    LuaApiMut::register_function(lua, "IsPublicTestClient", is_public_test_client)?;
    LuaApiMut::register_function(lua, "IsBetaBuild", is_beta_build)?;
    LuaApiMut::register_function(lua, "IsPublicBuild", is_public_build)?;
    LuaApiMut::register_function(lua, "BNFeaturesEnabled", bn_features_enabled)?;
    LuaApiMut::register_function(
        lua,
        "BNFeaturesEnabledAndConnected",
        bn_features_enabled_and_connected,
    )?;
    LuaApiMut::register_function(lua, "BNConnected", bn_connected)?;
    LuaApiMut::register_function(lua, "IsGMClient", is_gm_client)?;
    LuaApiMut::register_function(lua, "RegisterStaticConstants", register_static_constants)?;
    LuaApiMut::register_function(lua, "pcall", pcall)?;
    install_xpcall(lua)?;
    LuaApiMut::register_function(lua, "securecall", securecall)?;
    LuaApiMut::register_function(lua, "seterrorhandler", seterrorhandler)?;
    LuaApiMut::register_function(lua, "geterrorhandler", geterrorhandler)?;
    LuaApiMut::register_function(lua, "assertsafe", assertsafe)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;
    use rilua::{LuaApi, LuaApiMut};

    #[test]
    fn secure_taint_call_does_not_retry_matching_lua_error_text() {
        let mut lua = rilua::Lua::new().expect("lua should initialize");
        lua.exec("__secure_error_calls = 0")
            .expect("counter should initialize");
        let loader = lua
            .state_mut()
            .load(
                r#"
                    return function()
                        __secure_error_calls = __secure_error_calls + 1
                        error("expected Lua closure in execute: genuine secure failure")
                    end
                "#,
            )
            .expect("failing function source should compile");
        let failing =
            call_function_state_multi(lua.state_mut(), Val::Function(loader.gc_ref()), &[])
                .expect("failing function should load")[0];
        lua.state_mut().call_stack[0].taint = Some("SecureProbe".to_string());

        let error = call_function_with_secure_taint(lua.state_mut(), failing, &[])
            .expect_err("genuine Lua failure should be returned");
        assert!(error.to_string().contains("genuine secure failure"));
        assert_eq!(
            lua.state().call_stack[0].taint.as_deref(),
            Some("SecureProbe"),
            "securecall must restore caller taint"
        );

        let counter_loader = lua
            .state_mut()
            .load("return __secure_error_calls")
            .expect("counter source should compile");
        let calls =
            call_function_state_multi(lua.state_mut(), Val::Function(counter_loader.gc_ref()), &[])
                .expect("subsequent direct call should succeed");
        assert_eq!(calls, vec![Val::Num(1.0)], "failing function must run once");
    }

    #[test]
    fn type_global_returns_wow_type_names() {
        let env = WowLuaEnv::new().expect("fresh Lua env");
        let result: (String, String, String, String, String) = env
            .eval("return type('x'), type({}), type(function() end), type(1), type(nil)")
            .expect("type results");

        assert_eq!(
            result,
            (
                "string".to_string(),
                "table".to_string(),
                "function".to_string(),
                "number".to_string(),
                "nil".to_string(),
            )
        );
    }
}
