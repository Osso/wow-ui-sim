//! Admin storage input and inferred eligible-subset deletion share event publication.

use super::input::{read_public_integer, read_selector, read_variant_id};
use super::snapshot;
use super::{HousingCatalogEntryVariantID, HousingCatalogVariantRecord};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn set_variant_stored_count(state: &mut LuaState) -> LuaResult<u32> {
    let (id, count) = read_storage_input(state)?;
    let changed = update_stored_count(state, id, count)?;
    if changed {
        publish_storage_update(state, &id)?;
    }
    Ok(0)
}

/// Eligible-subset deletion is simulator policy, not native mixed-stack evidence.
pub(super) fn destroy_entry(state: &mut LuaState) -> LuaResult<u32> {
    let (id, destroy_all) = read_destroy_input(state)?;
    let changed = destroy_eligible_instances(state, id, destroy_all)?;
    if changed {
        publish_storage_update(state, &id)?;
    }
    Ok(0)
}

fn read_destroy_input(state: &mut LuaState) -> LuaResult<(HousingCatalogEntryVariantID, bool)> {
    let selector = read_selector(state)?;
    let id = read_variant_id(state, selector)?;
    for (field, value) in [
        ("recordID", id.record_id),
        ("entryType", id.entry_type),
        ("variantIdentifier", id.variant_identifier),
    ] {
        if value < 0 {
            return Err(rilua::runtime_error(format!(
                "housing catalog {field} must be nonnegative"
            )));
        }
    }
    let Val::Bool(destroy_all) = Val::from_stack(state, 2)? else {
        return Err(rilua::runtime_error(
            "housing catalog destroyAll must be a public boolean; secret access is not modeled",
        ));
    };
    Ok((id, destroy_all))
}

fn destroy_eligible_instances(
    state: &mut LuaState,
    id: HousingCatalogEntryVariantID,
    destroy_all: bool,
) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let Some(record) = sim.housing.catalog.variants.get_mut(&id) else {
        return Ok(false);
    };
    validate_destroy_counts(record)?;
    if record.destroyable_instance_count == 0 {
        return Ok(false);
    }
    let removed = if destroy_all {
        record.destroyable_instance_count
    } else {
        1
    };
    record.num_stored -= removed;
    record.destroyable_instance_count -= removed;
    Ok(true)
}

fn validate_destroy_counts(record: &HousingCatalogVariantRecord) -> LuaResult<()> {
    if record.num_stored < 0 {
        return Err(rilua::runtime_error(
            "housing catalog numStored must be nonnegative",
        ));
    }
    if record.destroyable_instance_count < 0
        || record.destroyable_instance_count > record.num_stored
    {
        return Err(rilua::runtime_error(
            "housing catalog destroyable instance count must be between zero and numStored",
        ));
    }
    Ok(())
}

fn read_storage_input(state: &mut LuaState) -> LuaResult<(HousingCatalogEntryVariantID, i32)> {
    let selector = read_selector(state)?;
    let id = read_variant_id(state, selector)?;
    let value = Val::from_stack(state, 2)?;
    let count = read_public_integer(value, "numStored")?;
    for (field, value) in [
        ("recordID", id.record_id),
        ("entryType", id.entry_type),
        ("variantIdentifier", id.variant_identifier),
        ("numStored", count),
    ] {
        if value < 0 {
            return Err(rilua::runtime_error(format!(
                "housing catalog {field} must be nonnegative"
            )));
        }
    }
    Ok((id, count))
}

fn update_stored_count(
    state: &mut LuaState,
    id: HousingCatalogEntryVariantID,
    count: i32,
) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    let record = sim.housing.catalog.variants.get_mut(&id).ok_or_else(|| {
        rilua::runtime_error("housing catalog storage input requires an existing full variant key")
    })?;
    if record.num_stored == count {
        return Ok(false);
    }
    record.num_stored = count;
    Ok(true)
}

fn publish_storage_update(
    state: &mut LuaState,
    id: &HousingCatalogEntryVariantID,
) -> LuaResult<()> {
    // Serializer roots the new table before field allocations; retain that root
    // through every listener and nested input, restoring it even on dispatch error.
    let saved_top = state.top;
    let payload = snapshot::push_id(state, id);
    let result = dispatch_event_now(state, "HOUSING_STORAGE_ENTRY_UPDATED", &[payload]);
    state.top = saved_top;
    result
}
