//! Legacy item globals.
//!
//! Plain global wrappers stay in the Lua globals layer; their modeled C_Item
//! implementation belongs to `c_api::item_spell`.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_legacy_item_globals(state: &mut LuaState) -> LuaResult<()> {
    if crate::client_profile::ACTIVE.has_legacy_item_globals() {
        table_set_rust_fn_static(
            state,
            state.global,
            "GetItemSpell",
            crate::c_api::item_spell::get_item_spell,
        )?;
    }
    table_set_rust_fn_static(
        state,
        state.global,
        "GetItemID",
        crate::c_api::item_spell::c_item_get_item_id,
    )
}
