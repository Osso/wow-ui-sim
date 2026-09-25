//! rilua-side frame method helpers for Phase 3 migration.
//!
//! Production frame methods need `SimState` access (behind `Rc<RefCell<>>`),
//! which doesn't fit the `define_methods!` macro's `FrameArena` pattern.
//! Instead, methods are raw `RustFn`s that use these helpers to extract
//! the frame ID from a rilua-backed table and borrow `SimState`.

use super::SimState;
use super::env::WowLuaAppData;
use super::hot_literals::{hot_metatable_key, metatable_idx};
use crate::lua_bridge::stack_val;
use crate::lua_bridge::{FrameIdentity, create_frame_table};
use crate::widget::WidgetType;
use rilua::vm::callinfo::LUA_MULTRET;
use rilua::vm::execute::{CallResult, execute};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};
use std::cell::{Ref, RefMut};
use std::rc::Rc;

mod frame_fields;
mod frame_metatable;
mod hot_static_literals;
pub use frame_fields::get_or_create_frame_fields;
use frame_metatable::frame_metatable_for_widget_type;

/// Extract the frame ID (u64) from a Lua argument (a backed table).
///
/// The table's backing `(index, generation)` encodes the widget ID as
/// `(id as u32, (id >> 32) as u32)`.
pub fn frame_id_from_stack(state: &LuaState, index: i32) -> LuaResult<u64> {
    let val = stack_val(state, index);
    frame_id_from_val(state, val)
}

fn frame_id_from_val(state: &LuaState, val: Val) -> LuaResult<u64> {
    dispatch_frame_id_from_val(state, val)
        .ok_or_else(|| runtime_error("expected frame-backed table"))
}

fn dispatch_frame_id_from_val(state: &LuaState, val: Val) -> Option<u64> {
    let Val::Table(table_ref) = val else {
        return None;
    };
    let table = state.gc.tables.get(table_ref)?;
    let backing = frame_identity_backing(state, table).or_else(|| table.backing())?;
    Some(pack_id(backing.0, backing.1))
}

fn frame_identity_backing(state: &LuaState, table: &Table) -> Option<(u32, u32)> {
    frame_identity_backing_from_val(state, table.get_int(0))
}

fn frame_identity_backing_from_val(state: &LuaState, val: Val) -> Option<(u32, u32)> {
    match val {
        Val::Userdata(identity_ref) => state
            .gc
            .userdata
            .get(identity_ref)
            .and_then(|identity| identity.downcast_ref::<FrameIdentity>())
            .map(|identity| (identity.index, identity.generation)),
        Val::Table(identity_ref) => state
            .gc
            .tables
            .get(identity_ref)
            .and_then(|identity| identity.backing()),
        _ => None,
    }
}

/// Borrow `SimState` immutably from rilua's app_data.
pub fn borrow_state(state: &LuaState) -> LuaResult<Ref<'_, SimState>> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    Ok(app.sim_state.borrow())
}

/// Borrow `SimState` mutably from rilua's app_data.
pub fn borrow_state_mut(state: &LuaState) -> LuaResult<RefMut<'_, SimState>> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    Ok(app.sim_state.borrow_mut())
}

/// Clone the owning SimState handle from app_data.
pub fn state_handle(state: &LuaState) -> LuaResult<Rc<std::cell::RefCell<SimState>>> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    Ok(Rc::clone(&app.sim_state))
}

/// Clone the owning rilua cell from app_data for runtime loader helpers.
pub fn borrow_lua(state: &LuaState) -> LuaResult<Rc<std::cell::RefCell<rilua::Lua>>> {
    let app = state
        .app_data::<WowLuaAppData>()
        .ok_or_else(|| runtime_error("missing WowLuaAppData"))?;
    app.lua
        .as_ref()
        .cloned()
        .ok_or_else(|| runtime_error("missing WowLuaEnv lua handle"))
}

// ── Frame ref creation and caching ───────────────────────────────────

const FRAME_MT_CACHE_KEY: &str = "__rilua_frame_mt_cache";

/// Get or create a rilua frame-backed table for the given widget ID.
///
/// Returns a cached `Val::Table` if one exists for this ID, otherwise creates
/// a new backed table with the shared frame metatable and caches it.
///
/// The metatable must be pre-registered in the registry as `__rilua_frame_mt`
/// before calling this function. If absent, the table has no metatable.
pub fn frame_ref(state: &mut LuaState, id: u64) -> LuaResult<Val> {
    let cache = frame_ref_cache(state);
    let cached = table_get_num(state, cache, id as f64);
    if cached != Val::Nil {
        return Ok(cached);
    }
    let (lo, hi) = unpack_id(id);
    let table_ref = create_frame_table(state, lo, hi);
    attach_frame_metatable(state, table_ref, id);
    // Frame backing tables are never deleted — wow-sim never removes a
    // frame from the registry. They must still be traversed by GC because
    // addons can store live Lua functions and tables directly on frame refs.
    state.gc.pin_object(Val::Table(table_ref));
    let val = Val::Table(table_ref);
    table_set_num(state, cache, id as f64, val);
    Ok(val)
}

/// Extract the WoW dispatch frame ID from a `Val`.
///
/// Prefer the `frame[0]` identity token and fall back to the table's native
/// backing only when the identity slot is missing or invalid.
pub fn extract_frame_id(state: &LuaState, val: Val) -> Option<u64> {
    dispatch_frame_id_from_val(state, val)
}

/// Extract the native frame table ID, ignoring `frame[0]` dispatch overrides.
pub fn native_frame_id_from_val(state: &LuaState, val: Val) -> Option<u64> {
    let Val::Table(table_ref) = val else {
        return None;
    };
    let (lo, hi) = state.gc.tables.get(table_ref)?.backing()?;
    Some(pack_id(lo, hi))
}

pub(crate) fn get_frame_env_for_debug(state: &mut LuaState) -> LuaResult<u32> {
    let frame_id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, frame_id);
    let env = create_table(state);
    if let Val::Table(env_ref) = env {
        if let Some(table) = state.gc.tables.get_mut(env_ref) {
            let _ = table.raw_set(Val::Num(1.0), fields, &state.gc.string_arena);
        }
        state.gc.barrier_back(env_ref);
    }
    state.push(env);
    Ok(1)
}

/// Sync a child frame ref into a parent frame's table.
///
/// Sets `parent_table[key] = child_frame_ref` so that Lua-side `parent.key`
/// resolves to the child frame.
pub fn sync_child_to_rilua(
    state: &mut LuaState,
    parent_id: u64,
    key: &str,
    child_id: u64,
) -> LuaResult<()> {
    let parent_val = frame_ref(state, parent_id)?;
    let child_val = frame_ref(state, child_id)?;
    let Val::Table(parent_ref) = parent_val else {
        return Ok(());
    };
    #[cfg(feature = "forbidden-aspects")]
    {
        let private_parent =
            crate::lua_api::script_object_transfer::project_forbidden(state, parent_val)?;
        if private_parent != parent_val {
            let private_child =
                crate::lua_api::script_object_transfer::project_forbidden(state, child_val)?;
            if table_get(state, private_parent, key) != private_child {
                table_set(state, private_parent, key, private_child);
            }
        }
    }
    let key_ref = intern_string_maybe_static(state, key);
    // Short-circuit if parent[key] already points at this exact child.
    // Startup loops (e.g. PartyFrame:InitializePartyMemberFrames) call
    // SetParentKey on every OnShow pass; a no-op second pass must not
    // redundantly re-write the same table slot and re-barrier the parent.
    let already_wired = state
        .gc
        .tables
        .get(parent_ref)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena) == child_val)
        .unwrap_or(false);
    if already_wired {
        return Ok(());
    }
    if let Some(t) = state.gc.tables.get_mut(parent_ref) {
        let _ = t.raw_set(Val::Str(key_ref), child_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(parent_ref);
    Ok(())
}

/// Remove `parent_table[key]` when it still points at the requested child.
pub fn clear_child_from_rilua_parent_key(
    state: &mut LuaState,
    parent_id: u64,
    key: &str,
    child_id: u64,
) -> LuaResult<()> {
    let parent_val = frame_ref(state, parent_id)?;
    let child_val = frame_ref(state, child_id)?;
    let Val::Table(parent_ref) = parent_val else {
        return Ok(());
    };
    let key_ref = intern_string_maybe_static(state, key);
    let points_at_child = state
        .gc
        .tables
        .get(parent_ref)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena) == child_val)
        .unwrap_or(false);
    if !points_at_child {
        return Ok(());
    }
    if let Some(t) = state.gc.tables.get_mut(parent_ref) {
        let _ = t.raw_set(Val::Str(key_ref), Val::Nil, &state.gc.string_arena);
    }
    state.gc.barrier_back(parent_ref);
    Ok(())
}

/// Publish the frame-ref cache under the legacy debug alias.
pub fn publish_frame_ref_cache_alias(state: &mut LuaState) {
    let cache = frame_ref_cache(state);
    registry_set(state, "__frame_refs", Val::Table(cache));
}

/// Get or create the frame ref cache table in the registry.
/// Pre-sized for ~3000 frames (typical Blizzard UI load).
fn frame_ref_cache(state: &mut LuaState) -> GcRef<Table> {
    let key_ref = hot_metatable_key(state, metatable_idx::RILUA_FRAME_REFS);
    let registry = state.gc.tables.get(state.registry);
    if let Some(reg) = registry
        && let Val::Table(cache) = reg.get_str(key_ref, &state.gc.string_arena)
    {
        return cache;
    }
    // Pre-allocate for typical frame count to avoid rehashing
    let cache = state.gc.alloc_table(Table::with_sizes(4096, 0));
    let registry = state.registry;
    if let Some(reg) = state.gc.tables.get_mut(registry) {
        let _ = reg.raw_set(Val::Str(key_ref), Val::Table(cache), &state.gc.string_arena);
    }
    state.gc.barrier_back(registry);
    cache
}

/// Fetch the pre-interned `__rilua_frame_mt` registry key via the hot-literal
/// registry installed at bootstrap. Falls back to `intern_string_static` if
/// the registry hasn't been installed yet (e.g. tests that skip the full
/// bootstrap pass) — the static cache still short-circuits on subsequent
/// calls, so correctness is identical either way.
fn frame_mt_registry_key(state: &mut LuaState) -> GcRef<rilua::vm::string::LuaString> {
    hot_metatable_key(state, metatable_idx::RILUA_FRAME_MT)
}

/// Attach the shared frame metatable to a table (if registered).
///
/// Methods are accessed via `__index` in the metatable, not copied directly.
/// This avoids ~636 raw_set calls per frame and reduces memory/GC pressure.
fn attach_frame_metatable(state: &mut LuaState, table_ref: GcRef<Table>, id: u64) {
    let mt_key = frame_mt_registry_key(state);
    let mt_val = state
        .gc
        .tables
        .get(state.registry)
        .map(|reg| reg.get_str(mt_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if let Val::Table(mt_ref) = mt_val {
        let widget_type = borrow_state(state)
            .ok()
            .and_then(|sim| sim.widgets.get(id).map(|frame| frame.widget_type))
            .unwrap_or(WidgetType::Frame);
        let type_mt = frame_metatable_for_widget_type(state, mt_ref, widget_type);
        if let Some(t) = state.gc.tables.get_mut(table_ref) {
            if let Val::Table(type_mt_ref) = type_mt {
                t.set_metatable(Some(type_mt_ref));
            } else {
                t.set_metatable(Some(mt_ref));
            }
        }
        // Methods accessed via __index, no need to copy
    }
}

/// Get a numeric-keyed value from a table.
pub(super) fn table_get_num(state: &LuaState, table: GcRef<Table>, key: f64) -> Val {
    state
        .gc
        .tables
        .get(table)
        .map(|t| {
            let int_key = key as i64;
            if int_key > 0 && int_key as f64 == key {
                t.get_int(int_key)
            } else {
                t.get(Val::Num(key), &state.gc.string_arena)
            }
        })
        .unwrap_or(Val::Nil)
}

/// Set a numeric-keyed value in a table.
#[cfg_attr(feature = "rehash-stats", track_caller)]
pub(crate) fn table_set_num(state: &mut LuaState, table: GcRef<Table>, key: f64, value: Val) {
    if let Some(t) = state.gc.tables.get_mut(table) {
        let int_key = key as i64;
        if int_key > 0 && int_key as f64 == key {
            let _ = t.raw_set(Val::Num(int_key as f64), value, &state.gc.string_arena);
        } else {
            let _ = t.raw_set(Val::Num(key), value, &state.gc.string_arena);
        }
    }
    state.gc.barrier_back(table);
}

// ── ID encoding ─────────────────────────────────────────────────────

/// Pack a u64 widget ID into the (u32, u32) backing slot format.
pub fn pack_id(lo: u32, hi: u32) -> u64 {
    (lo as u64) | ((hi as u64) << 32)
}

/// Unpack a u64 widget ID into (lo, hi) for table backing.
pub fn unpack_id(id: u64) -> (u32, u32) {
    (id as u32, (id >> 32) as u32)
}

// ── String creation ─────────────────────────────────────────────────

/// Create a Lua string Val from a Rust &str.
pub fn create_string(state: &mut LuaState, s: &str) -> Val {
    Val::Str(intern_string_maybe_static(state, s))
}

fn intern_string_maybe_static(
    state: &mut LuaState,
    value: &str,
) -> GcRef<rilua::vm::string::LuaString> {
    if let Some(static_value) = hot_static_literals::get(value) {
        return state.gc.intern_string_static(static_value.as_bytes());
    }
    state.gc.intern_string(value.as_bytes())
}

/// Like [`create_string`] but interns `s` through the pointer-keyed
/// static cache. Use when the string is a compile-time literal hit on a
/// hot path (`"CENTER"`, `"MIDDLE"`, `"LEFT"` / `"TOP"` fallbacks from
/// small `&'static str`-returning methods, etc.).
pub fn create_string_static(state: &mut LuaState, s: &'static str) -> Val {
    Val::Str(state.gc.intern_string_static(s.as_bytes()))
}

/// Create a Lua string Val from raw bytes.
pub fn create_string_bytes(state: &mut LuaState, bytes: &[u8]) -> Val {
    Val::Str(state.gc.intern_string(bytes))
}

/// Extract a Rust String from a Lua string Val.
pub fn val_to_string(state: &LuaState, val: Val) -> Option<String> {
    let Val::Str(str_ref) = val else { return None };
    let lua_str = state.gc.string_arena.get(str_ref)?;
    std::str::from_utf8(lua_str.data()).ok().map(str::to_owned)
}

// ── Function calling ────────────────────────────────────────────────

/// Call a Lua function stored as a Val with the given arguments.
///
/// Wraps `rilua::Lua::call_function` for use in frame method dispatch.
/// Returns the first return value, or Val::Nil if no returns.
pub fn call_function(lua: &mut rilua::Lua, func: Val, args: &[Val]) -> LuaResult<Val> {
    let Val::Function(func_ref) = func else {
        return Err(runtime_error("expected function"));
    };
    let func_handle = rilua::Function::from_gc_ref(func_ref);
    let results = lua.call_function(&func_handle, args)?;
    Ok(results.into_iter().next().unwrap_or(Val::Nil))
}

pub fn call_function_state(state: &mut LuaState, func: Val, args: &[Val]) -> LuaResult<Val> {
    let results = call_function_state_multi(state, func, args)?;
    Ok(results.into_iter().next().unwrap_or(Val::Nil))
}

pub fn call_function_state_multi(
    state: &mut LuaState,
    func: Val,
    args: &[Val],
) -> LuaResult<Vec<Val>> {
    let Val::Function(_) = func else {
        return Err(runtime_error("expected function"));
    };
    let func_idx = state.top;
    state.ensure_stack(func_idx + 1 + args.len());
    state.stack_set(func_idx, func);
    state.top = func_idx + 1;

    for arg in args {
        let top = state.top;
        state.stack_set(top, *arg);
        state.top = top + 1;
    }

    let saved_base = state.base;
    let saved_ci = state.ci;
    state.base = func_idx + 1;

    let result = match state.precall(func_idx, LUA_MULTRET) {
        Ok(CallResult::Lua) => execute(state),
        Ok(CallResult::Rust) => Ok(()),
        Err(err) => Err(err),
    };

    if let Err(error) = result {
        state.top = func_idx;
        state.base = saved_base;
        state.ci = saved_ci;
        if state.ci < rilua::vm::state::MAXCALLS {
            state.ci_overflow = false;
        }
        return Err(error);
    }

    let results = (func_idx..state.top)
        .map(|index| state.stack_get(index))
        .collect();
    state.top = func_idx;
    state.base = saved_base;
    Ok(results)
}

/// Call a Lua function with error handling (pcall semantics).
///
/// Returns the results on success, or logs the error and returns an empty vec on failure.
pub fn pcall_function(lua: &mut rilua::Lua, func: Val, args: &[Val]) -> Vec<Val> {
    let Val::Function(func_ref) = func else {
        return vec![];
    };
    let func_handle = rilua::Function::from_gc_ref(func_ref);
    match lua.call_function(&func_handle, args) {
        Ok(results) => results,
        Err(e) => {
            super::script_helpers::call_error_handler(lua, &e.to_string());
            vec![]
        }
    }
}

// ── Registry value storage ──────────────────────────────────────────

/// Get a named value from rilua's registry table.
///
/// Equivalent to mlua's `lua.named_registry_value(key)`.
pub fn registry_get(state: &mut LuaState, key: &'static str) -> Val {
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    state
        .gc
        .tables
        .get(state.registry)
        .map(|reg| reg.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

/// Set a named value in rilua's registry table.
///
/// Equivalent to mlua's `lua.set_named_registry_value(key, value)`.
pub fn registry_set(state: &mut LuaState, key: &'static str, value: Val) {
    let key_ref = state.gc.intern_string_static(key.as_bytes());
    let registry = state.registry;
    if let Some(reg) = state.gc.tables.get_mut(registry) {
        let _ = reg.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    // Required: rilua's GC needs a back-barrier whenever a value is
    // raw-written into a table that may already be black this cycle.
    // Without it, mid-Propagate writes leave the new value un-marked
    // and it gets swept (see investigations/intern-string-ranking.md).
    state.gc.barrier_back(registry);
}

/// Get or create a named table in rilua's registry.
///
/// Equivalent to mlua's pattern: `lua.named_registry_value(key).unwrap_or_else(|| { create + set })`.
pub fn registry_table_or_create(state: &mut LuaState, key: &'static str) -> Val {
    let existing = registry_get(state, key);
    if let Val::Table(_) = existing {
        return existing;
    }
    let table = Val::Table(state.gc.alloc_table(Table::new()));
    registry_set(state, key, table);
    table
}

// ── Table creation ──────────────────────────────────────────────────

/// Create a new empty table in rilua's GC heap.
pub fn create_table(state: &mut LuaState) -> Val {
    Val::Table(state.gc.alloc_table(Table::new()))
}

/// Create a new table with pre-allocated hash capacity.
/// `hash_capacity` is rounded up to the next power of 2.
pub fn create_table_with_capacity(state: &mut LuaState, hash_capacity: usize) -> Val {
    Val::Table(state.gc.alloc_table(Table::with_sizes(0, hash_capacity)))
}

/// Create a new table and set string-keyed fields on it.
/// Pre-sizes the hash part to avoid rehashing.
pub fn create_table_with_fields(state: &mut LuaState, fields: &[(&'static str, Val)]) -> Val {
    let table_ref = state.gc.alloc_table(Table::with_sizes(0, fields.len()));
    for &(key, value) in fields {
        let key_ref = state.gc.intern_string_static(key.as_bytes());
        if let Some(t) = state.gc.tables.get_mut(table_ref) {
            let _ = t.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
        }
    }
    Val::Table(table_ref)
}

/// Set a string key on an existing table Val.
#[cfg_attr(feature = "rehash-stats", track_caller)]
pub fn table_set(state: &mut LuaState, table: Val, key: &str, value: Val) {
    table_set_with_key(state, table, value, |state| {
        intern_string_maybe_static(state, key)
    });
}

/// Like [`table_set`] but interns `key` through the pointer-keyed static
/// cache. Callers must pass the same `&'static str` pointer that the
/// matching `table_get_static` uses for lookup — content-dedupe works
/// too, but the fast path only triggers when the pointer matches.
#[cfg_attr(feature = "rehash-stats", track_caller)]
pub fn table_set_static(state: &mut LuaState, table: Val, key: &'static str, value: Val) {
    table_set_with_key(state, table, value, |state| {
        state.gc.intern_string_static(key.as_bytes())
    });
}

#[cfg_attr(feature = "rehash-stats", track_caller)]
fn table_set_with_key<F>(state: &mut LuaState, table: Val, value: Val, intern_key: F)
where
    F: FnOnce(&mut LuaState) -> GcRef<rilua::vm::string::LuaString>,
{
    let Val::Table(table_ref) = table else { return };
    let stack_slot = state.top;
    state.ensure_stack(stack_slot + 1);
    state.stack_set(stack_slot, value);
    state.top = stack_slot + 1;

    let key_ref = intern_key(state);
    if let Some(t) = state.gc.tables.get_mut(table_ref) {
        let _ = t.raw_set(Val::Str(key_ref), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
    state.top = stack_slot;
}

/// Get a string key from a table Val.
pub fn table_get(state: &mut LuaState, table: Val, key: &str) -> Val {
    table_get_with_key(state, table, |state| intern_string_maybe_static(state, key))
}

/// Like [`table_get`] but interns `key` through the pointer-keyed static
/// cache. Use when the key is a compile-time literal hit on a hot path
/// (script handler dispatch, XML attribute lookup, etc.).
pub fn table_get_static(state: &mut LuaState, table: Val, key: &'static str) -> Val {
    table_get_with_key(state, table, |state| {
        state.gc.intern_string_static(key.as_bytes())
    })
}

fn table_get_with_key<F>(state: &mut LuaState, table: Val, intern_key: F) -> Val
where
    F: FnOnce(&mut LuaState) -> GcRef<rilua::vm::string::LuaString>,
{
    let Val::Table(table_ref) = table else {
        return Val::Nil;
    };
    let key_ref = intern_key(state);
    state
        .gc
        .tables
        .get(table_ref)
        .map(|t| t.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

#[cfg(test)]
mod tests {
    use super::{
        borrow_state, create_string_bytes, create_table, frame_ref, table_get, table_get_static,
        table_set, table_set_static, val_to_string,
    };
    use crate::lua_api::WowLuaEnv;
    use rilua::LuaApiMut;
    use rilua::Val;

    #[test]
    fn frame_ref_returns_same_table_for_same_widget_id() {
        let env = WowLuaEnv::new().expect("env");
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let ui_parent_id = borrow_state(state)
            .expect("borrow state")
            .widgets
            .get_id_by_name("UIParent")
            .expect("UIParent");

        let first = frame_ref(state, ui_parent_id).expect("first frame ref");
        let second = frame_ref(state, ui_parent_id).expect("second frame ref");

        assert_eq!(first, second, "frame_ref should reuse cached table refs");
    }

    #[test]
    fn table_set_variants_store_string_keys() {
        let env = WowLuaEnv::new().expect("env");
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let table = create_table(state);

        table_set(state, table, "dynamicKey", Val::Num(10.0));
        table_set_static(state, table, "staticKey", Val::Num(20.0));

        assert_eq!(table_get(state, table, "dynamicKey"), Val::Num(10.0));
        assert_eq!(table_get_static(state, table, "staticKey"), Val::Num(20.0));
    }

    #[test]
    fn table_get_variants_return_values_or_nil() {
        let env = WowLuaEnv::new().expect("env");
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let table = create_table(state);

        table_set(state, table, "dynamicKey", Val::Num(30.0));
        table_set_static(state, table, "staticKey", Val::Num(40.0));

        assert_eq!(table_get(state, table, "dynamicKey"), Val::Num(30.0));
        assert_eq!(table_get_static(state, table, "staticKey"), Val::Num(40.0));
        assert_eq!(table_get(state, table, "missingKey"), Val::Nil);
        assert_eq!(table_get_static(state, Val::Nil, "staticKey"), Val::Nil);
    }

    #[test]
    fn val_to_string_validates_lua_string_bytes_before_allocating_result() {
        let env = WowLuaEnv::new().expect("env");
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();

        let valid = create_string_bytes(state, b"hello");
        let invalid = create_string_bytes(state, &[0xff]);

        assert_eq!(val_to_string(state, valid).as_deref(), Some("hello"));
        assert_eq!(val_to_string(state, invalid), None);
    }
}
