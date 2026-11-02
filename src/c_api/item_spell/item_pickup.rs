//! `C_Item.PickupItem` places an item ID on the native ActionBar cursor
//! (mode 6), per WM44-A and its WM46 erratum.

use crate::c_api::item_upgrade_eligibility;
use crate::lua_api::globals::real::cursor_model;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::CursorInfo;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(super) fn pickup_item(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.PickupItem: expected itemInfo"))?;
    item_upgrade_eligibility::discard_cursor(&mut *borrow_state_mut(state)?);
    if item == 0 {
        cursor_model::clear(state)?;
    } else {
        cursor_model::place(state, CursorInfo::ActionBar { item_id: item })?;
    }
    Ok(0)
}
