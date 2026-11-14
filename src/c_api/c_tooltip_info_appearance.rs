//! Retail/PTR use supplied transmog sources; Forever uses supplied item mappings.
//! TooltipInfoDocumentation.lua:458 declares the getter on Retail, PTR and Forever.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_bridge::{stack_val, val_to_i32};
use rilua::LuaResult;
#[cfg(feature = "retail-12-0-5")]
use rilua::table_security::unwrap_secret;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

#[cfg(feature = "client-wowforever")]
const USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetItemByItemModifiedAppearanceID(itemModifiedAppearanceID)";

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetItemByItemModifiedAppearanceID",
        get_item_by_appearance,
    )
}

#[cfg(feature = "client-wowforever")]
fn get_item_by_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let appearance_id = crate::c_api::tooltip_getter_args::round7_signed(state, 1, USAGE)?;
    let item_id = borrow_state(state)?
        .forever_appearance_item_ids
        .get(&appearance_id)
        .copied();
    let Some(item_id) = item_id else {
        return Ok(0);
    };
    let Some(item) = crate::items::get_item(item_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    state.push(tooltip);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn get_item_by_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let source_id = val_to_i32(unwrap_secret(state, stack_val(state, 1))?, 1)?;
    let source = {
        let sim = borrow_state(state)?;
        crate::c_api::c_transmog_collection::read_source_info(&sim, i64::from(source_id))
    };
    let item = source
        .and_then(|source| crate::c_api::item_spell::parse_prefixed_id(&source.item_link, "item"))
        .and_then(|id| crate::items::get_item(id).map(|item| (id, item)));
    let Some((id, item)) = item else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, id, Some(item));
    state.push(tooltip);
    Ok(1)
}
