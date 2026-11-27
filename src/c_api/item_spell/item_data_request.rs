//! `C_Item.RequestLoadItemDataByID(itemInfo)` over the simulator item catalog.
//!
//! Retail ItemDocumentation.lua:1673 and Forever :1710 declare the request;
//! the `ITEM_DATA_LOAD_RESULT(itemID, success)` event that Blizzard_ObjectAPI
//! `AsyncCallbackSystem.lua` listens for is declared `SynchronousEvent = true`.
//! INFERRED R88g9 (native evidence pending): the catalog is the only item
//! cache, so the result fires during the call and `success` reports whether the
//! catalog has the item.

use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn request_load_item_data_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item_id = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.RequestLoadItemDataByID: expected itemInfo"))?;
    let success = crate::items::get_item(item_id).is_some();
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "ITEM_DATA_LOAD_RESULT",
        &[Val::Num(f64::from(item_id)), Val::Bool(success)],
    )?;
    Ok(0)
}
