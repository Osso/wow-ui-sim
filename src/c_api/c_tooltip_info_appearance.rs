//! Item tooltip selection from the existing supplied transmog source model.
//! Retail/PTR TooltipInfoDocumentation.lua:458 declares the getter.

use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static, val_to_i32};
use rilua::LuaResult;
use rilua::table_security::unwrap_secret;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetItemByItemModifiedAppearanceID",
        get_item_by_appearance,
    )
}

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
