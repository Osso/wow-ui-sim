//! Explicit set-filter state; native defaults and filtering effects are unmodeled.

mod catalog;
pub use catalog::{TransmogSetInfo, TransmogSets};

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let forever = crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        let namespace = ensure_namespace(state, "C_TransmogSets")?;
        table_set_rust_fn_static(state, namespace, "HasAnyValidSets", catalog::has_any_valid)?;
    }
    // Forever 70707 TransmogSetsDocumentation.lua and the pinned
    // Mists/Era/Anniversary TransmogSetsDocumentation.lua:325-332 declare
    // HasAvailableSets and GetSetInfo. The pinned 3.3.5 Wrath source has no
    // C_TransmogSets namespace.
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        let namespace = ensure_namespace(state, "C_TransmogSets")?;
        table_set_rust_fn_static(state, namespace, "GetSetInfo", catalog::get_info)?;
        table_set_rust_fn_static(state, namespace, "HasAvailableSets", catalog::has_available)?;
    }
    // WowForever's TransmogSetsDocumentation.lua:231-491 declares the same query
    // and filter surface as Retail/PTR, so it registers the whole block below.
    if !cfg!(feature = "retail-12-0-0") && !forever {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_TransmogSets")?;
    table_set_rust_fn_static(state, namespace, "GetVariantSets", catalog::get_variants)?;
    table_set_rust_fn_static(state, namespace, "GetAvailableSets", catalog::get_available)?;
    table_set_rust_fn_static(state, namespace, "GetSetsFilter", get_sets_filter)?;
    table_set_rust_fn_static(state, namespace, "SetSetsFilter", set_sets_filter)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsUsingDefaultSetsFilters",
        is_using_defaults,
    )?;
    table_set_rust_fn_static(state, namespace, "SetDefaultSetsFilters", set_defaults)
}

fn is_using_defaults(state: &mut LuaState) -> LuaResult<u32> {
    let defaults = borrow_state(state)?
        .transmog_set_filters
        .values()
        .all(|value| *value);
    state.push(Val::Bool(defaults));
    Ok(1)
}

fn set_defaults(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.transmog_set_filters.clear();
    Ok(0)
}

fn filter_index(state: &LuaState) -> LuaResult<i32> {
    let Val::Num(index) = stack_val(state, 1) else {
        return Err(rilua::runtime_error("SetsFilter requires a numeric index"));
    };
    Ok(index as i32)
}

fn get_sets_filter(state: &mut LuaState) -> LuaResult<u32> {
    let index = filter_index(state)?;
    let value = borrow_state(state)?
        .transmog_set_filters
        .get(&index)
        .copied();
    // INFERRED: collected/uncollected/PvE/PvP filters default checked.
    let value = match value {
        Some(value) => value,
        None if (1..=4).contains(&index) => true,
        None => return Ok(0),
    };
    state.push(Val::Bool(value));
    Ok(1)
}

fn set_sets_filter(state: &mut LuaState) -> LuaResult<u32> {
    let index = filter_index(state)?;
    let Val::Bool(value) = stack_val(state, 2) else {
        return Err(rilua::runtime_error("SetSetsFilter requires a boolean"));
    };
    borrow_state_mut(state)?
        .transmog_set_filters
        .insert(index, value);
    Ok(0)
}
