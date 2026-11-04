//! Merchant cursor presentation (WM86). Held cursor content remains separate.
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const POINT: u8 = 1;
const BUY: u8 = 3;
const INSPECT: u8 = 7;
const REPAIR: u8 = 17;
const UNABLE_BUY: u8 = 49;
const CUSTOM: u8 = 91;

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "ShowBuybackSellCursor", show_buyback_sell_cursor)?;
    LuaApiMut::register_function(lua, "ShowRepairCursor", show_repair_cursor)?;
    LuaApiMut::register_function(lua, "InRepairMode", in_repair_mode)?;
    LuaApiMut::register_function(lua, "SetCursor", set_cursor)?;
    LuaApiMut::register_function(lua, "ResetCursor", reset_cursor)
}

fn presentation_allowed(state: &LuaState) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(!sim.cursor_storage.presentation_gate_a && !sim.cursor_storage.presentation_gate_b)
}

fn display(state: &mut LuaState, selector: u8) -> LuaResult<()> {
    let notify = {
        let mut sim = borrow_state_mut(state)?;
        let cursor = &mut sim.cursor_storage;
        if selector == CUSTOM || cursor.presentation_display == selector {
            return Ok(());
        }
        cursor.presentation_display = selector;
        cursor.presentation_refresh && cursor.presentation_notifications
    };
    if notify {
        let mode = cursor_model::current_mode(state)?;
        let (empty, id) = {
            let sim = borrow_state(state)?;
            (sim.cursor_item.is_none(), sim.cursor_storage.stored_item_id)
        };
        let args = [
            Val::Bool(empty),
            Val::Num(f64::from(mode)),
            Val::Num(f64::from(mode)),
            Val::Num(f64::from(id)),
        ];
        // EMULATED: use existing simulator dispatch; native listener timing and re-entry are unknown.
        dispatch_event_now(state, "CURSOR_CHANGED", &args)?;
    }
    Ok(())
}

fn show_buyback_sell_cursor(state: &mut LuaState) -> LuaResult<u32> {
    if !presentation_allowed(state)?
        || borrow_state(state)?.cursor_storage.presentation_base != POINT
    {
        return Ok(0);
    }
    let input = stack_val(state, 1);
    let number = match input {
        Val::Num(value) => Some(value),
        Val::Str(_) => val_to_string(state, input).and_then(|text| text.parse::<f64>().ok()),
        _ => None,
    }
    .filter(|number| number.is_finite())
    .ok_or_else(|| rilua::runtime_error("Usage: ShowBuybackSellCursor(index)"))?;
    let index = (number.trunc().rem_euclid(4294967296.0) as u32).wrapping_sub(1) as usize;
    let selector = {
        let sim = borrow_state(state)?;
        sim.merchant_buyback_items
            .get(index)
            .filter(|item| item.item_id != 0)
            .map(|item| {
                if sim.player.money.max(0) as u64 >= item.price {
                    BUY
                } else {
                    UNABLE_BUY
                }
            })
    };
    if let Some(selector) = selector {
        display(state, selector)?;
    }
    Ok(0)
}

fn show_repair_cursor(state: &mut LuaState) -> LuaResult<u32> {
    if !presentation_allowed(state)? {
        return Ok(0);
    }
    let can_repair = {
        let sim = borrow_state(state)?;
        sim.merchant_frame_open && sim.merchant_repair_capable
    };
    if can_repair {
        cursor_model::clear(state)?;
        borrow_state_mut(state)?.cursor_storage.presentation_base = REPAIR;
        display(state, REPAIR)?;
    }
    Ok(0)
}

fn in_repair_mode(state: &mut LuaState) -> LuaResult<u32> {
    let repair = borrow_state(state)?.cursor_storage.presentation_base == REPAIR;
    state.push(Val::Bool(repair));
    Ok(1)
}

fn reset_cursor(state: &mut LuaState) -> LuaResult<u32> {
    let base = borrow_state(state)?.cursor_storage.presentation_base;
    display(state, base)?;
    Ok(0)
}

fn set_cursor(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if value == Val::Nil {
        reset_cursor(state)?;
        state.push(Val::Bool(true));
        return Ok(1);
    }
    let name = match value {
        Val::Str(_) | Val::Num(_) => val_to_string(state, value),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error("Usage: local result = SetCursor([name])"))?;
    let selector = match name.to_ascii_uppercase().as_str() {
        "POINT_CURSOR" => Some(POINT),
        "BUY_CURSOR" => Some(BUY),
        "INSPECT_CURSOR" => Some(INSPECT),
        "REPAIR_CURSOR" => Some(REPAIR),
        _ => None,
    };
    if let Some(selector) = selector {
        display(state, selector)?;
    }
    // EMULATED: unknown custom-cursor asset behavior retains the existing true result.
    state.push(Val::Bool(true));
    Ok(1)
}
