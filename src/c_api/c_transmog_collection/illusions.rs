//! Ordered illusion queries over explicit host inputs, not a native catalog.

use crate::lua_api::methods::{borrow_state, table_set, table_set_num};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::IllusionInfo;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetIllusions", get_illusions)
}

fn read_category(state: &LuaState) -> LuaResult<Option<u32>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Nil => Ok(None),
        Val::Num(number) if is_category_number(number) => Ok(Some(number as u32)),
        _ => Err(rilua::runtime_error(
            "category requires nil or a finite unsigned 32-bit integer",
        )),
    }
}

fn is_category_number(number: f64) -> bool {
    let is_integer = number.is_finite() && number.fract() == 0.0;
    let in_range = (0.0..=u32::MAX as f64).contains(&number);
    is_integer && in_range
}

fn get_illusions(state: &mut LuaState) -> LuaResult<u32> {
    let category = read_category(state)?;
    // Snapshot only selected primitive records; release the host borrow before VM allocation.
    let records: Vec<_> = borrow_state(state)?
        .transmog_illusions
        .iter()
        .filter(|record| category.is_none_or(|category| record.category == category))
        .cloned()
        .collect();
    let array = TableBuilder::new(state).build();
    state.push(array);
    let Val::Table(array_ref) = array else {
        unreachable!()
    };
    for (index, record) in records.iter().enumerate() {
        let row = TableBuilder::new(state).build();
        // The stack-rooted array owns each row before field keys allocate.
        table_set_num(state, array_ref, (index + 1) as f64, row);
        write_illusion_fields(state, row, record);
    }
    Ok(1)
}

fn write_illusion_fields(state: &mut LuaState, row: Val, record: &IllusionInfo) {
    for (key, value) in [
        ("visualID", record.visual_id),
        ("sourceID", record.source_id),
        ("icon", record.icon),
    ] {
        table_set(state, row, key, Val::Num(value as f64));
    }
    for (key, value) in [
        ("isCollected", record.is_collected),
        ("isUsable", record.is_usable),
        ("isHideVisual", record.is_hide_visual),
    ] {
        table_set(state, row, key, Val::Bool(value));
    }
}
