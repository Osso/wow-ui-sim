//! Pending-only AllowedWhenUntainted selector boundary; catalog guards stay unchanged.

use super::super::catalog::HousingCatalogEntryVariantID;
use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::FromStack;
use rilua::table_security::{check_table_access, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn read_variant_id(state: &mut LuaState) -> LuaResult<HousingCatalogEntryVariantID> {
    let original = Val::from_stack(state, 1)?;
    let selector = unwrap_secret(state, original)?;
    let Val::Table(reference) = selector else {
        return Err(rilua::runtime_error(
            "housing pending selector must be a table",
        ));
    };
    check_table_access(state, reference, None)?;

    // Keep the underlying table and its original fields rooted across field-key allocations.
    let stack_top = state.top;
    state.push(selector);
    let result = read_authenticated_variant_id(state, selector);
    state.top = stack_top;
    result
}

fn read_authenticated_variant_id(
    state: &mut LuaState,
    selector: Val,
) -> LuaResult<HousingCatalogEntryVariantID> {
    let original_record = table_get_static(state, selector, "recordID");
    let original_type = table_get_static(state, selector, "entryType");
    let original_variant = table_get_static(state, selector, "variantIdentifier");

    // Authenticate every original field before malformed public fields can mask a denial.
    let record = unwrap_secret(state, original_record)?;
    let entry_type = unwrap_secret(state, original_type)?;
    let variant = unwrap_secret(state, original_variant)?;
    Ok(HousingCatalogEntryVariantID {
        record_id: read_integer(record, "recordID")?,
        entry_type: read_integer(entry_type, "entryType")?,
        variant_identifier: read_integer(variant, "variantIdentifier")?,
    })
}

fn read_integer(value: Val, field: &'static str) -> LuaResult<i32> {
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(format!(
            "housing pending {field} must be an integer"
        )));
    };
    let integer = number as i32;
    if f64::from(integer) != number {
        return Err(rilua::runtime_error(format!(
            "housing pending {field} is outside the integer input range"
        )));
    }
    Ok(integer)
}
