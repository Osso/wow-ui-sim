//! Forever buyback reads; collection contents are an explicit simulator scenario.

use crate::items;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaError, LuaResult, RuntimeError, Val};

/// A sold stack's configured read snapshot, independent of merchant stock.
/// Price is total copper; availability/usability/binding are supplied, not inferred from item type.
#[derive(Debug, Clone)]
pub struct BuybackItem {
    pub item_id: u32,
    pub quantity: u32,
    pub price: u64,
    pub num_available: i32,
    pub is_usable: bool,
    pub is_bound: bool,
}

fn buyback_at(state: &LuaState) -> LuaResult<Option<BuybackItem>> {
    let index = i64::from_stack(state, 1)?;
    let slot = usize::try_from(index)
        .ok()
        .and_then(|index| index.checked_sub(1));
    let model = borrow_state(state)?;
    Ok(slot.and_then(|slot| model.merchant_buyback_items.get(slot).cloned()))
}

fn unknown_item(item_id: u32) -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: format!("unknown buyback item {item_id}: item metadata is not modeled"),
        level: 1,
        traceback: vec![],
    })
}

fn get_num_buyback_items(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.merchant_buyback_items.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_buyback_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(entry) = buyback_at(state)? else {
        return Ok(0);
    };
    let item = items::get_item(entry.item_id).ok_or_else(|| unknown_item(entry.item_id))?;
    let name = create_string(state, item.name);
    state.push(name);
    state.push(Val::Num(item.icon_file_data_id as f64));
    state.push(Val::Num(entry.price as f64));
    state.push(Val::Num(entry.quantity as f64));
    state.push(Val::Num(entry.num_available as f64));
    state.push(Val::Bool(entry.is_usable));
    state.push(Val::Bool(entry.is_bound));
    Ok(7)
}

fn get_buyback_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let Some(entry) = buyback_at(state)? else {
        return Ok(0);
    };
    let link = crate::c_api::item_spell::item_link_for_id(entry.item_id)
        .ok_or_else(|| unknown_item(entry.item_id))?;
    let link = create_string(state, &link);
    state.push(link);
    Ok(1)
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetNumBuybackItems", get_num_buyback_items)?;
    LuaApiMut::register_function(lua, "GetBuybackItemInfo", get_buyback_item_info)?;
    LuaApiMut::register_function(lua, "GetBuybackItemLink", get_buyback_item_link)
}
