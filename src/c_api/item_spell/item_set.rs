//! Item set membership and names from ItemSparse and ItemSet records.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_sets::{ITEM_SET_MEMBERS, ITEM_SET_NAMES};

/// Retail ItemSparse.ItemSet for an item; items outside a set have no ID.
pub(crate) fn item_set_id(item_id: u32) -> Option<u32> {
    if ACTIVE != ClientProfile::Retail {
        return None;
    }
    ITEM_SET_MEMBERS
        .binary_search_by_key(&item_id, |entry| entry.0)
        .ok()
        .map(|index| ITEM_SET_MEMBERS[index].1)
}

fn parse_set_id(state: &LuaState, value: Val) -> Option<i32> {
    match value {
        Val::Num(number)
            if number.is_finite() && number >= i32::MIN as f64 && number <= i32::MAX as f64 =>
        {
            Some(number as i32)
        }
        Val::Str(_) => val_to_string(state, value)?
            .trim()
            .parse::<f64>()
            .ok()
            .and_then(|number| {
                (number.is_finite() && number >= i32::MIN as f64 && number <= i32::MAX as f64)
                    .then_some(number as i32)
            }),
        _ => None,
    }
}

pub(super) fn get_item_set_info(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let set_id = parse_set_id(state, value)
        .ok_or_else(|| runtime_error("C_Item.GetItemSetInfo: expected setID"))?;
    let name = u32::try_from(set_id).ok().and_then(|set_id| {
        ITEM_SET_NAMES
            .binary_search_by_key(&set_id, |entry| entry.0)
            .ok()
            .map(|index| ITEM_SET_NAMES[index].1)
    });
    let result = name.map_or(Val::Nil, |name| create_string(state, name));
    state.push(result);
    Ok(1)
}
