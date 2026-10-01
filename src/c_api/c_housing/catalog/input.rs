//! Shared public catalog selector guards; never unwrap secrets or clear caller taint.

use super::{HousingCatalogEntryID, HousingCatalogEntryVariantID};
use crate::lua_api::methods::table_get_static;
use crate::lua_bridge::FromStack;
use rilua::table_security::check_table_access;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(in crate::c_api::c_housing) fn read_selector(state: &mut LuaState) -> LuaResult<Val> {
    let selector = Val::from_stack(state, 1)?;
    let Val::Table(reference) = selector else {
        return Err(rilua::runtime_error(
            "housing catalog selector must be a public table",
        ));
    };
    check_table_access(state, reference, None)?;
    Ok(selector)
}

pub(super) fn read_public_integer(value: Val, field: &'static str) -> LuaResult<i32> {
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(format!(
            "housing catalog {field} must be a public integer; secret access is not modeled"
        )));
    };
    let integer = number as i32;
    if f64::from(integer) != number {
        return Err(rilua::runtime_error(format!(
            "housing catalog {field} is outside the integer input range"
        )));
    }
    Ok(integer)
}

fn read_integer_field(state: &mut LuaState, selector: Val, field: &'static str) -> LuaResult<i32> {
    let value = table_get_static(state, selector, field);
    read_public_integer(value, field)
}

pub(super) fn read_entry_id(
    state: &mut LuaState,
    selector: Val,
) -> LuaResult<HousingCatalogEntryID> {
    Ok(HousingCatalogEntryID {
        record_id: read_integer_field(state, selector, "recordID")?,
        entry_type: read_integer_field(state, selector, "entryType")?,
    })
}

pub(in crate::c_api::c_housing) fn read_variant_id(
    state: &mut LuaState,
    selector: Val,
) -> LuaResult<HousingCatalogEntryVariantID> {
    let entry = read_entry_id(state, selector)?;
    Ok(HousingCatalogEntryVariantID {
        record_id: entry.record_id,
        entry_type: entry.entry_type,
        variant_identifier: read_integer_field(state, selector, "variantIdentifier")?,
    })
}
