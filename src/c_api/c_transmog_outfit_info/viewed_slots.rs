//! Exact viewed-slot tuple lookup with authenticated secret-selector access.

use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

use super::ViewedOutfitSlotInfo;

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetViewedOutfitSlotInfo",
        get_viewed_outfit_slot_info,
    )
}

fn read_selector(state: &LuaState, index: i32) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(format!(
            "viewed outfit selector #{index} requires a number"
        )));
    };
    if !number.is_finite()
        || number.fract() != 0.0
        || number < i32::MIN as f64
        || number > i32::MAX as f64
    {
        return Err(rilua::runtime_error(format!(
            "viewed outfit selector #{index} requires an integer representable as i32"
        )));
    }
    Ok(number as i32)
}

fn get_viewed_outfit_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let key = (
        read_selector(state, 1)?,
        read_selector(state, 2)?,
        read_selector(state, 3)?,
    );
    let record = {
        let sim = borrow_state(state)?;
        sim.transmog_outfits
            .pending_slots
            .get(&key)
            .or_else(|| sim.viewed_outfit_slots.get(&key))
            .cloned()
    };
    let Some(record) = record else { return Ok(0) };
    let table = build_slot_table(state, &record);
    state.push(table);
    Ok(1)
}

fn build_slot_table(state: &mut LuaState, record: &ViewedOutfitSlotInfo) -> Val {
    let table = create_table(state);
    for (key, value) in [
        ("transmogID", record.transmog_id as f64),
        ("displayType", record.display_type as f64),
        ("warning", record.warning as f64),
        ("error", record.error as f64),
        ("sheatheCategory", record.sheathe_category as f64),
    ] {
        table_set(state, table, key, Val::Num(value));
    }
    for (key, value) in [
        ("isTransmogrified", record.is_transmogrified),
        ("hasPending", record.has_pending),
        ("isPendingCollected", record.is_pending_collected),
        ("canTransmogrify", record.can_transmogrify),
    ] {
        table_set(state, table, key, Val::Bool(value));
    }
    for (key, value) in [
        ("warningText", record.warning_text.as_str()),
        ("errorText", record.error_text.as_str()),
    ] {
        let value = create_string(state, value);
        table_set(state, table, key, value);
    }
    if let Some(texture) = record.texture {
        table_set(state, table, "texture", Val::Num(texture as f64));
    }
    table
}
