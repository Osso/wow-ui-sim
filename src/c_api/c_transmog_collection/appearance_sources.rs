//! Exact source-ID lookup with authenticated secret-selector access.

use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::AppearanceSourceInfo;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAppearanceSourceInfo",
        get_appearance_source_info,
    )
}

fn read_source_id(state: &LuaState) -> LuaResult<i64> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(
            "itemModifiedAppearanceID requires a number",
        ));
    };
    // Reject non-integral/out-of-range keys rather than truncating into another record.
    if !number.is_finite()
        || number.fract() != 0.0
        || number < i64::MIN as f64
        || number >= -(i64::MIN as f64)
    {
        return Err(rilua::runtime_error(
            "itemModifiedAppearanceID requires an integer representable as i64",
        ));
    }
    Ok(number as i64)
}

fn get_appearance_source_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_source_id(state)?;
    let record = borrow_state(state)?
        .transmog_appearance_sources
        .get(&id)
        .cloned();
    let Some(record) = record else { return Ok(0) };
    let table = build_source_table(state, &record);
    state.push(table);
    Ok(1)
}

fn build_source_table(state: &mut LuaState, record: &AppearanceSourceInfo) -> Val {
    let table = create_table(state);
    for (key, value) in [
        ("category", record.category as f64),
        ("itemAppearanceID", record.item_appearance_id as f64),
        ("icon", record.icon as f64),
        ("itemSubclass", record.item_subclass as f64),
    ] {
        table_set(state, table, key, Val::Num(value));
    }
    for (key, value) in [
        ("canHaveIllusion", record.can_have_illusion),
        ("isCollected", record.is_collected),
        (
            "ignoreModelAttachmentChecksForIllusion",
            record.ignore_model_attachment_checks_for_illusion,
        ),
    ] {
        table_set(state, table, key, Val::Bool(value));
    }
    for (key, value) in [
        ("itemLink", record.item_link.as_str()),
        ("transmoglink", record.transmoglink.as_str()),
    ] {
        let value = create_string(state, value);
        table_set(state, table, key, value);
    }
    if let Some(source_type) = record.source_type {
        table_set(state, table, "sourceType", Val::Num(source_type as f64));
    }
    table
}
