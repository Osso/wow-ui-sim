//! R76/R76b: save the host order without changing the live roster.

use crate::c_api::c_login::character_roster::slots_for_sim;
#[cfg(feature = "client-retail")]
use crate::lua_api::methods::val_to_string;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const USAGE: &str = "Usage: SaveCharacterOrder(characterTranslationOrder)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if borrow_state(state)?.screen_kind.is_glue() {
        table_set_rust_fn_static(
            state,
            state.global,
            "SaveCharacterOrder",
            save_character_order,
        )
    } else {
        table_set(
            state,
            Val::Table(state.global),
            "SaveCharacterOrder",
            Val::Nil,
        );
        Ok(())
    }
}

fn entry(_state: &LuaState, value: Val) -> Option<i32> {
    let number = match value {
        Val::Num(n) => n,
        #[cfg(feature = "client-retail")]
        Val::Str(_) => val_to_string(_state, value)?.trim().parse().ok()?,
        _ => return None,
    };
    // Numeric conversion policy; native coercion edge cases remain unasserted.
    let number = number.trunc();
    (number >= i32::MIN as f64 && number <= i32::MAX as f64).then_some(number as i32)
}

fn save_character_order(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(table) = stack_val(state, 1) else {
        return Err(runtime_error(USAGE));
    };
    let table = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error(USAGE))?;
    let (roster_count, selected) = {
        let sim = borrow_state(state)?;
        (
            slots_for_sim(&sim).len(),
            sim.glue_login.selected_character_index,
        )
    };
    let mut order = Vec::new();
    let mut selected_slot = None;
    for raw_slot in 0..table.len(&state.gc.string_arena) {
        let value = table.get_int((raw_slot + 1) as i64);
        let index = entry(state, value).ok_or_else(|| runtime_error(USAGE))?;
        if index == 0 {
            continue;
        }
        if index < 0 || index as usize > roster_count {
            // The selected-before-abort CVar case is not established. Do not
            // publish an order or make an assertion about that ordering.
            return Ok(0);
        }
        if i64::from(index) - 1 == selected {
            selected_slot = Some(raw_slot);
        }
        order.push(index);
    }
    let mut sim = borrow_state_mut(state)?;
    if let Some(slot) = selected_slot {
        sim.cvars.set("lastCharacterIndex", &slot.to_string());
    }
    sim.glue_login.saved_character_order = order;
    Ok(0)
}
