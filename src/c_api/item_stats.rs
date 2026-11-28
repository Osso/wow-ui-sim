//! Supplied exact-link item-stat queries; no catalog or equipment inference.

use std::collections::HashMap;

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Default)]
pub struct ItemStatsState {
    pub by_link: HashMap<String, HashMap<String, f64>>,
}

// Forever 70707 ItemDocumentation.lua:986, :1003.
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Item")?;
    table_set_rust_fn_static(state, namespace, "GetItemStats", c_item_get_item_stats)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetItemStatDelta",
        c_item_get_item_stat_delta,
    )
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(
    builder: crate::lua_bridge::TableBuilder,
) -> LuaResult<crate::lua_bridge::TableBuilder> {
    builder.set_function("SetItemStats", set_item_stats)
}

#[cfg(feature = "client-wowforever")]
fn set_item_stats(state: &mut LuaState) -> LuaResult<u32> {
    let key = stack_val(state, 1);
    if !matches!(key, Val::Str(_)) {
        return Err(runtime_error("SetItemStats: expected item link string"));
    }
    let link = require_link(state, key)?;
    let stats = read_stat_map(state, stack_val(state, 2))?;
    crate::lua_api::methods::borrow_state_mut(state)?
        .item_stats
        .by_link
        .insert(link, stats);
    Ok(0)
}

#[cfg(feature = "client-wowforever")]
fn read_stat_map(state: &LuaState, value: Val) -> LuaResult<HashMap<String, f64>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error("SetItemStats: expected stat table"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("SetItemStats: stat table unavailable"))?;
    if table.array_slice().iter().any(|v| !matches!(v, Val::Nil)) {
        return Err(runtime_error("SetItemStats: expected string keys"));
    }
    table
        .hash_entries()
        .iter()
        .filter(|(_, v)| !matches!(v, Val::Nil))
        .map(|(key, value)| decode_stat(state, *key, *value))
        .collect()
}

#[cfg(feature = "client-wowforever")]
fn decode_stat(state: &LuaState, key: Val, value: Val) -> LuaResult<(String, f64)> {
    let Val::Str(_) = key else {
        return Err(runtime_error("SetItemStats: expected string keys"));
    };
    let name = val_to_string(state, key)
        .ok_or_else(|| runtime_error("SetItemStats: invalid stat name"))?;
    let Val::Num(number) = value else {
        return Err(runtime_error("SetItemStats: expected numeric stat value"));
    };
    if !number.is_finite() {
        return Err(runtime_error("SetItemStats: stat value must be finite"));
    }
    Ok((name, number))
}

fn c_item_get_item_stats(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let link = require_link(state, value)?;
    let supplied = {
        let sim = borrow_state(state)?;
        sim.item_stats.by_link.get(&link).cloned()
    };
    push_stat_map(state, supplied)
}

fn c_item_get_item_stat_delta(state: &mut LuaState) -> LuaResult<u32> {
    // Authenticate both original operands before validating either. Wrappers
    // remain rooted on the argument stack and caller taint is never changed.
    let first_value = unwrap_secret(state, stack_val(state, 1))?;
    let second_value = unwrap_secret(state, stack_val(state, 2))?;
    let first_link = require_link(state, first_value)?;
    let second_link = require_link(state, second_value)?;
    let supplied = {
        let sim = borrow_state(state)?;
        sim.item_stats
            .by_link
            .get(&first_link)
            .zip(sim.item_stats.by_link.get(&second_link))
            .map(|(first, second)| (first.clone(), second.clone()))
    };
    let delta = supplied.map(|(first, second)| subtract_stat_maps(&first, &second));
    push_stat_map(state, delta)
}

fn require_link(state: &LuaState, value: Val) -> LuaResult<String> {
    val_to_string(state, value)
        .ok_or_else(|| runtime_error("C_Item: item link must be a string (UTF-8)"))
}

fn subtract_stat_maps(
    first: &HashMap<String, f64>,
    second: &HashMap<String, f64>,
) -> HashMap<String, f64> {
    let first_keys = first.keys();
    let second_only_keys = second.keys().filter(|key| !first.contains_key(*key));
    first_keys
        .chain(second_only_keys)
        .filter_map(|key| {
            let difference =
                first.get(key).copied().unwrap_or(0.0) - second.get(key).copied().unwrap_or(0.0);
            (difference != 0.0).then(|| (key.clone(), difference))
        })
        .collect()
}

fn push_stat_map(state: &mut LuaState, stats: Option<HashMap<String, f64>>) -> LuaResult<u32> {
    let Some(stats) = stats else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table_ref = state.gc.alloc_table(Table::with_sizes(0, stats.len()));
    // Root the result before interning keys, which can allocate and collect.
    state.push(Val::Table(table_ref));
    for (key, value) in stats {
        let key_ref = state.gc.intern_string(key.as_bytes());
        let table = state
            .gc
            .tables
            .get_mut(table_ref)
            .ok_or_else(|| runtime_error("item-stat result table has been collected"))?;
        table.raw_set(Val::Str(key_ref), Val::Num(value), &state.gc.string_arena)?;
        state.gc.barrier_back(table_ref);
    }
    Ok(1)
}
