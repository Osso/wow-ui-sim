//! Forever 70532 GetOutfitsInfo query over the shared outfit catalog.
//! Declaration: TransmogOutfitInfoDocumentation.lua:383–392, entry:896–907.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_TransmogOutfitInfo")?;
    table_set_rust_fn_static(state, namespace, "GetOutfitsInfo", get_outfits_info)?;
    table_set_rust_fn_static(state, namespace, "GetOutfitInfo", get_outfit_info)
}

fn get_outfit_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = i64::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.outfit_id == id)
        .cloned();
    let Some(entry) = entry else { return Ok(0) };
    let snapshot = super::outfit_entry::build_entry_table(state, &entry);
    state.push(snapshot);
    Ok(1)
}

fn get_outfits_info(state: &mut LuaState) -> LuaResult<u32> {
    let entries = borrow_state(state)?.transmog_outfit_catalog.entries.clone();
    if entries.is_empty() {
        return Ok(0);
    }
    let snapshot = super::outfit_entry::build_entry_array(state, &entries);
    state.push(snapshot);
    Ok(1)
}
