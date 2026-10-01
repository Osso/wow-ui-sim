//! Plain public selectors preserve caller taint; secret fields are not unwrapped.

use super::HousingCatalogEntryID;
use super::input::{read_entry_id, read_public_integer, read_selector, read_variant_id};
use super::snapshot;
use crate::c_api::helpers::ensure_namespace;
use crate::c_api::item_spell::parse_item_id_from_val;
use crate::lua_api::methods::{borrow_state, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaApiMut;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(in crate::c_api::c_housing) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_HousingCatalog")?;
    let functions: &[(&str, rilua::RustFn)] = &[
        ("GetCatalogCategoryInfo", super::categories::category_info),
        (
            "GetCatalogSubcategoryInfo",
            super::categories::subcategory_info,
        ),
        (
            "SearchCatalogCategories",
            super::category_search::search_categories,
        ),
        (
            "SearchCatalogSubcategories",
            super::category_search::search_subcategories,
        ),
        ("GetCatalogEntryInfo", entry_info),
        ("GetCatalogEntryInfoByItem", entry_info_by_item),
        ("GetCatalogEntryInfoByRecordID", entry_info_by_record_id),
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

fn entry_info_by_item(state: &mut LuaState) -> LuaResult<u32> {
    let selector = Val::from_stack(state, 1)?;
    // Inspect only public scalars. The shared parser never receives a secret value.
    if !matches!(selector, Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error(
            "housing catalog item selector must be a public number or string; secret access is not modeled",
        ));
    }
    let Some(item_id) =
        parse_item_id_from_val(state, selector).and_then(|item_id| i32::try_from(item_id).ok())
    else {
        // Item names have no resolver; catalog display names are not item names.
        state.push(Val::Nil);
        return Ok(1);
    };
    let id = {
        let sim = borrow_state(state)?;
        let mut matches = sim
            .housing
            .catalog
            .entries
            .iter()
            .filter(|(_, record)| record.item_id == Some(item_id));
        let id = matches.next().map(|(id, _)| *id);
        // No native winner is known: reject ambiguous input, not a hash-order winner.
        if matches.next().is_some() {
            return Err(rilua::runtime_error(
                "ambiguous explicit housing catalog item ID",
            ));
        }
        id
    };
    match id {
        Some(id) => push_entry_info(state, id),
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

fn entry_info_by_record_id(state: &mut LuaState) -> LuaResult<u32> {
    let entry_type = read_public_integer(Val::from_stack(state, 1)?, "entryType")?;
    let record_id = read_public_integer(Val::from_stack(state, 2)?, "recordID")?;
    push_entry_info(
        state,
        HousingCatalogEntryID {
            record_id,
            entry_type,
        },
    )
}

fn entry_info(state: &mut LuaState) -> LuaResult<u32> {
    let selector = read_selector(state)?;
    let id = read_entry_id(state, selector)?;
    push_entry_info(state, id)
}

fn push_entry_info(state: &mut LuaState, id: HousingCatalogEntryID) -> LuaResult<u32> {
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
