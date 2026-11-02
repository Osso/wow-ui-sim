//! Item uniqueness lookup from item and limit-category records.

use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_uniqueness::{ITEM_LIMIT_CATEGORIES, ITEM_UNIQUENESS};

pub(super) fn get_item_uniqueness(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.GetItemUniqueness: expected itemInfo"))?;
    let Ok(index) = ITEM_UNIQUENESS.binary_search_by_key(&item, |entry| entry.0) else {
        return Ok(0);
    };
    let (_, category, bypass) = ITEM_UNIQUENESS[index];
    if !bypass && ITEM_LIMIT_CATEGORIES.binary_search(&category).is_err() {
        return Ok(0);
    }
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(2)
}
