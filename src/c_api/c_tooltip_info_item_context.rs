//! Retail 12.0.5 item-context lookup with AllowedWhenUntainted authentication.
//! Exact-key, numeric-domain and miss policies are inferred, not native semantics.

use super::ItemTooltipContext;
use crate::items::{self, ItemInfo};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const API_NAME: &str = "C_TooltipInfo.GetItemByID";

/// Publish only into the existing, globally rooted namespace.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetItemByID", get_item_by_id)
}

fn get_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let arguments = authenticate_arguments(state)?;
    let key = ItemTooltipContext {
        item_id: parse_u32(arguments[0], 1)?,
        item_context: parse_optional_u32(arguments[2], 3)?,
        treasure_context_level: parse_optional_u32(arguments[3], 4)?,
    };
    // Quality is authenticated above, then ignored just as in the prior producer.
    let item = select_item(state, key)?;
    let tooltip = tooltip_for_item_context(state, key.item_id, item.as_ref());
    state.push(tooltip);
    Ok(1)
}

fn authenticate_arguments(state: &LuaState) -> LuaResult<[Val; 4]> {
    let mut arguments = [Val::Nil; 4];
    for (offset, argument) in arguments.iter_mut().enumerate() {
        let position = offset as i32 + 1;
        // Original wrappers remain rooted on the existing call stack.
        *argument = unwrap_secret(state, stack_val(state, position)).map_err(|error| {
            rilua::runtime_error(format!("{API_NAME}: argument {position}: {error}"))
        })?;
    }
    Ok(arguments)
}

fn parse_optional_u32(value: Val, position: i32) -> LuaResult<Option<u32>> {
    if value == Val::Nil {
        return Ok(None);
    }
    parse_u32(value, position).map(Some)
}

fn parse_u32(value: Val, position: i32) -> LuaResult<u32> {
    if let Val::Num(number) = value {
        if is_exact_u32(number) {
            return Ok(number as u32);
        }
    }
    Err(rilua::runtime_error(format!(
        "{API_NAME}: argument {position} requires a finite integral u32 number"
    )))
}

fn is_exact_u32(number: f64) -> bool {
    number.is_finite() && f64::from(number as u32) == number
}

fn select_item(state: &LuaState, key: ItemTooltipContext) -> LuaResult<Option<ItemInfo>> {
    // Copy the exact host input and release the borrow before any VM allocation.
    let level = borrow_state(state)?.item_tooltip_levels.get(&key).copied();
    let Some(catalog_item) = items::get_item(key.item_id) else {
        return Ok(None);
    };
    if level.is_none() && (key.item_context.is_some() || key.treasure_context_level.is_some()) {
        return Ok(None);
    }
    let mut item = catalog_item.clone();
    if let Some(level) = level {
        item.item_level = level;
    }
    Ok(Some(item))
}
