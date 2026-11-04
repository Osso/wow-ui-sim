//! Supplied loot-slot tooltip lookup.
//!
//! Retail and PTR TooltipInfoDocumentation.lua:554 declares GetLootItem.
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetLootItem", get_loot_item)
}

fn get_loot_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let index = usize::try_from(slot)
        .ok()
        .and_then(|slot| slot.checked_sub(1));
    let item_id = {
        let model = borrow_state(state)?;
        index.and_then(|index| model.loot_slots.get(index).map(|item| item.item_id))
    };
    let Some(item_id) = item_id else {
        return emulated_missing_tooltip();
    };
    let tooltip = tooltip_for_item_context(state, item_id, crate::items::get_item(item_id));
    state.push(tooltip);
    Ok(1)
}

/// EMULATED: WM82 leaves native missing-result arity UNKNOWN.
fn emulated_missing_tooltip() -> LuaResult<u32> {
    Ok(0)
}
