//! Applied transmog supplied on individual inventory items.

use crate::lua_api::methods::{borrow_state, create_table, table_get, table_set};
use crate::lua_api::state::AppliedTransmog;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

fn index(value: Val) -> Option<i32> {
    match value {
        Val::Num(n) if n.is_finite() && n >= i32::MIN as f64 && n <= i32::MAX as f64 => {
            Some(n as i32)
        }
        _ => None,
    }
}

pub(crate) fn get_applied_item_transmog_info(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    if !matches!(location, Val::Table(_)) {
        return Err(runtime_error(
            "Usage: local info = C_Item.GetAppliedItemTransmogInfo(itemLoc)",
        ));
    }
    let equipment = index(table_get(state, location, "equipmentSlotIndex"));
    let bag = index(table_get(state, location, "bagID"));
    let slot = index(table_get(state, location, "slotIndex"));
    let info = {
        let sim = borrow_state(state)?;
        if let Some(slot) = equipment {
            sim.player
                .equipped_items
                .get(&slot)
                .map(|item| item.applied_transmog)
        } else if let (Some(bag), Some(slot)) = (bag, slot) {
            sim.bag_items
                .get(&(bag, slot))
                .map(|item| item.applied_transmog)
        } else {
            None
        }
    }
    .ok_or_else(|| {
        runtime_error("Usage: local info = C_Item.GetAppliedItemTransmogInfo(itemLoc)")
    })?;
    let AppliedTransmog {
        appearance_id,
        secondary_appearance_id,
        illusion_id,
    } = info.unwrap_or_default();
    let result = create_table(state);
    copy_mixin(state, result)?;
    table_set(
        state,
        result,
        "appearanceID",
        Val::Num(appearance_id as f64),
    );
    table_set(
        state,
        result,
        "secondaryAppearanceID",
        Val::Num(secondary_appearance_id as f64),
    );
    table_set(state, result, "illusionID", Val::Num(illusion_id as f64));
    state.push(result);
    Ok(1)
}

fn copy_mixin(state: &mut LuaState, result: Val) -> LuaResult<()> {
    let key = state.gc.intern_string(b"ItemTransmogInfoMixin");
    let mixin = state
        .gc
        .tables
        .get(state.global)
        .map(|t| t.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let (Val::Table(source), Val::Table(target)) = (mixin, result) else {
        return Err(runtime_error("ItemTransmogInfoMixin is unavailable"));
    };
    let mut entries = Vec::new();
    let table = state
        .gc
        .tables
        .get(source)
        .ok_or_else(|| runtime_error("ItemTransmogInfoMixin is unavailable"))?;
    let mut key = Val::Nil;
    while let Some((next, value)) = table.next(key, &state.gc.string_arena)? {
        entries.push((next, value));
        key = next;
    }
    let table = state
        .gc
        .tables
        .get_mut(target)
        .ok_or_else(|| runtime_error("transmog result is unavailable"))?;
    for (key, value) in entries {
        table.raw_set(key, value, &state.gc.string_arena)?;
    }
    state.gc.barrier_back(target);
    Ok(())
}
