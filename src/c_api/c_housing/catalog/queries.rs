//! Plain public selectors preserve caller taint; secret fields are not unwrapped.

use super::input::{read_entry_id, read_selector, read_variant_id};
use super::snapshot;
use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaApiMut;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(in crate::c_api::c_housing) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_HousingCatalog")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("GetCatalogEntryInfo", entry_info),
        ("GetCatalogEntryVariantInfo", variant_info),
        ("GetDestroyableInstanceCount", destroyable_instance_count),
        ("DestroyEntry", super::storage::destroy_entry),
        ("GetAllVariantInfosForEntry", variant_infos),
    ];
    for &(name, function) in functions {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    install_searcher_factory(state, Val::Table(namespace))
}

fn install_searcher_factory(state: &mut LuaState, namespace: Val) -> LuaResult<()> {
    let factory = state.load(include_str!("searcher.lua"))?;
    let call_base = state.top;
    state.push(Val::Function(factory.gc_ref()));
    let publisher = Closure::Rust(RustClosure::new(search_items, "HousingCatalogSearchItems"));
    let publisher = state.gc.alloc_closure(publisher);
    state.push(Val::Function(publisher));
    state.call_function(call_base, 1)?;
    let create_searcher = state.stack_get(call_base);
    table_set_static(state, namespace, "CreateCatalogSearcher", create_searcher);
    state.top = call_base;
    Ok(())
}

fn destroyable_instance_count(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let id = read_variant_id(state, selector)?;
    let count = borrow_state(state)?
        .housing
        .catalog
        .variants
        .get(&id)
        .map_or(0, |record| record.destroyable_instance_count);
    state.push(Val::Num(f64::from(count)));
    Ok(1)
}

fn entry_info(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let id = read_entry_id(state, selector)?;
    let record = borrow_state(state)?
        .housing
        .catalog
        .entries
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            snapshot::push_entry(state, &id, &record);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn variant_info(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let id = read_variant_id(state, selector)?;
    let record = borrow_state(state)?
        .housing
        .catalog
        .variants
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            snapshot::push_variant(state, &(id, record));
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn variant_infos(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let entry = read_entry_id(state, selector)?;
    let records: Vec<_> = borrow_state(state)?
        .housing
        .catalog
        .variants
        .iter()
        .filter(|(id, _)| id.record_id == entry.record_id && id.entry_type == entry.entry_type)
        .map(|(id, record)| (*id, record.clone()))
        .collect();
    snapshot::push_variants(state, &records);
    Ok(1)
}

fn search_items(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<_> = borrow_state(state)?
        .housing
        .catalog
        .variants
        .keys()
        .copied()
        .collect();
    snapshot::push_ids(state, &ids);
    Ok(1)
}
