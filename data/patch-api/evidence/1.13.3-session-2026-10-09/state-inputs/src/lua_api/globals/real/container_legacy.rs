//! Legacy container globals.
//!
//! These plain globals mirror `C_Container` query methods. Register them from
//! the Lua globals layer so `c_api::item_spell` owns only C_* namespaces.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const LEGACY_CONTAINER_GLOBALS: &[(&str, rilua::RustFn)] = &[
    (
        "GetContainerNumSlots",
        crate::c_api::item_spell::c_container_get_num_slots,
    ),
    (
        "GetContainerItemID",
        crate::c_api::item_spell::c_container_get_item_id,
    ),
    (
        "GetContainerItemLink",
        crate::c_api::item_spell::c_container_get_item_link,
    ),
];

pub(crate) fn register_legacy_container_globals(state: &mut LuaState) -> LuaResult<()> {
    for &(name, rust_fn) in LEGACY_CONTAINER_GLOBALS {
        table_set_rust_fn_static(state, state.global, name, rust_fn)?;
    }
    #[cfg(feature = "client-wowforever")]
    table_set_rust_fn_static(state, state.global, "HasKey", has_key)?;
    Ok(())
}

#[cfg(feature = "client-wowforever")]
fn has_key(state: &mut LuaState) -> LuaResult<u32> {
    // Forever BagIndex.Keyring is -1. Membership, not item-name guessing,
    // defines a key in the simulator's modeled keyring inventory.
    const KEYRING_CONTAINER: i32 = -1;
    let has_key = crate::lua_api::methods::borrow_state(state)?
        .bag_items
        .iter()
        .any(|(&(bag, slot), item)| {
            bag == KEYRING_CONTAINER && slot > 0 && item.item_id != 0 && item.stack_count > 0
        });
    state.push(rilua::Val::Bool(has_key));
    Ok(1)
}
