//! Each serializer leaves one stack root; nested rows never alias model state.

use super::{
    HousingCatalogEntryID, HousingCatalogEntryRecord, HousingCatalogEntryVariantID,
    HousingCatalogVariantRecord, HousingDecorDyeSlot,
};
use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{create_string, create_table, table_set_static};
use rilua::Val;
use rilua::vm::state::LuaState;

fn push_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

fn attach_child(state: &mut LuaState, parent: Val, field: &'static str, child: Val) {
    table_set_static(state, parent, field, child);
    state.top -= 1;
}

fn set_number(state: &mut LuaState, table: Val, field: &'static str, number: i32) {
    table_set_static(state, table, field, Val::Num(f64::from(number)));
}

fn set_text(state: &mut LuaState, table: Val, field: &'static str, text: &str) {
    let value = create_string(state, text);
    table_set_static(state, table, field, value);
}

fn push_sequence<T>(
    state: &mut LuaState,
    records: &[T],
    push_row: fn(&mut LuaState, &T) -> Val,
) -> Val {
    let sequence = push_table(state);
    for (index, record) in records.iter().enumerate() {
        let row = push_row(state, record);
        set_table_array(state, sequence, (index + 1) as i64, row);
        state.top -= 1;
    }
    sequence
}

pub(super) fn push_entry(
    state: &mut LuaState,
    id: &HousingCatalogEntryID,
    record: &HousingCatalogEntryRecord,
) -> Val {
    let row = push_table(state);
    set_number(state, row, "recordID", id.record_id);
    set_number(state, row, "entryType", id.entry_type);
    if let Some(item_id) = record.item_id {
        set_number(state, row, "itemID", item_id);
    }
    set_text(state, row, "name", &record.name);
    table_set_static(
        state,
        row,
        "isUniqueTrophy",
        Val::Bool(record.is_unique_trophy),
    );
    row
}

pub(super) fn push_id(state: &mut LuaState, id: &HousingCatalogEntryVariantID) -> Val {
    let row = push_table(state);
    set_number(state, row, "recordID", id.record_id);
    set_number(state, row, "entryType", id.entry_type);
    set_number(state, row, "variantIdentifier", id.variant_identifier);
    row
}

pub(super) fn push_variant(
    state: &mut LuaState,
    input: &(HousingCatalogEntryVariantID, HousingCatalogVariantRecord),
) -> Val {
    let (id, record) = input;
    let row = push_table(state);
    set_number(state, row, "numStored", record.num_stored);
    let identifier = push_id(state, id);
    attach_child(state, row, "entryVariantID", identifier);
    let dyes = push_sequence(state, &record.dye_slots, push_dye);
    attach_child(state, row, "dyeSlots", dyes);
    row
}

fn push_dye(state: &mut LuaState, dye: &HousingDecorDyeSlot) -> Val {
    let row = push_table(state);
    for (field, value) in [
        ("ID", dye.id),
        ("dyeColorCategoryID", dye.dye_color_category_id),
        ("orderIndex", dye.order_index),
        ("channel", dye.channel),
    ] {
        set_number(state, row, field, value);
    }
    if let Some(color_id) = dye.dye_color_id {
        set_number(state, row, "dyeColorID", color_id);
    }
    if let Some(name) = &dye.dye_color_name {
        set_text(state, row, "dyeColorName", name);
    }
    row
}

pub(super) fn push_variants(
    state: &mut LuaState,
    records: &[(HousingCatalogEntryVariantID, HousingCatalogVariantRecord)],
) -> Val {
    push_sequence(state, records, push_variant)
}

pub(super) fn push_ids(state: &mut LuaState, ids: &[HousingCatalogEntryVariantID]) -> Val {
    push_sequence(state, ids, push_id)
}
