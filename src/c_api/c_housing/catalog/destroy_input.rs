//! DestroyEntry-only AllowedWhenUntainted inputs; shared catalog guards stay public.

use super::super::HousingCatalogEntryVariantID;
use super::super::input::read_public_integer;
use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::FromStack;
use rilua::table_security::{check_table_access, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn read_input(state: &mut LuaState) -> LuaResult<(HousingCatalogEntryVariantID, bool)> {
    let saved_top = state.top;
    let result = read_authenticated_input(state);
    state.top = saved_top;
    result
}

fn read_authenticated_input(
    state: &mut LuaState,
) -> LuaResult<(HousingCatalogEntryVariantID, bool)> {
    let original_selector = Val::from_stack(state, 1)?;
    let original_all = Val::from_stack(state, 2)?;
    let selector = unwrap_secret(state, original_selector)?;
    state.push(selector);
    let destroy_all = unwrap_secret(state, original_all)?;
    state.push(destroy_all);

    // Authenticate both original arguments before any selector or boolean type error.
    let Val::Table(reference) = selector else {
        return Err(rilua::runtime_error(
            "housing catalog selector must be a table",
        ));
    };
    check_table_access(state, reference, None)?;
    let id = read_authenticated_variant_id(state, selector)?;
    let Val::Bool(destroy_all) = destroy_all else {
        return Err(rilua::runtime_error(
            "housing catalog destroyAll must be a boolean",
        ));
    };
    Ok((id, destroy_all))
}

fn read_authenticated_variant_id(
    state: &mut LuaState,
    selector: Val,
) -> LuaResult<HousingCatalogEntryVariantID> {
    // Root the underlying selector and original fields across all field-key allocations.
    let original_record = table_get_static(state, selector, "recordID");
    state.push(original_record);
    let original_type = table_get_static(state, selector, "entryType");
    state.push(original_type);
    let original_variant = table_get_static(state, selector, "variantIdentifier");
    state.push(original_variant);

    // Every field authenticates before any field type, integer range or domain check.
    let record = unwrap_secret(state, original_record)?;
    let entry_type = unwrap_secret(state, original_type)?;
    let variant = unwrap_secret(state, original_variant)?;
    let id = HousingCatalogEntryVariantID {
        record_id: read_public_integer(record, "recordID")?,
        entry_type: read_public_integer(entry_type, "entryType")?,
        variant_identifier: read_public_integer(variant, "variantIdentifier")?,
    };
    validate_nonnegative(id)?;
    Ok(id)
}

fn validate_nonnegative(id: HousingCatalogEntryVariantID) -> LuaResult<()> {
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
    Ok(())
}
