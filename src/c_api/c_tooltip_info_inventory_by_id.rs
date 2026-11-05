//! WM94 §1: `GetInventoryItemByID` selects an owned instance and composes the
//! ordinary item tooltip from it.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:407` declare the getter.
//! WM94 establishes the search (bag candidates before the equipment range) and
//! the instance-backed request. Instance identity (`guid`) and per-instance gem
//! population are not modelled by this simulator, so the producer publishes the
//! shared catalog item lines; that gap is recorded in the lane report.

use crate::c_api::tooltip_getter_args::unsigned;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetInventoryItemByID",
        get_inventory_item_by_id,
    )
}

fn get_inventory_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = unsigned(state, 1)?;
    let owned = {
        let sim = borrow_state(state)?;
        sim.bag_items
            .values()
            .any(|bag_item| bag_item.item_id == item_id)
            || sim
                .player
                .equipped_items
                .values()
                .any(|equipped| equipped.item_id == item_id)
    };
    let Some(item) = owned.then(|| crate::items::get_item(item_id)).flatten() else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    state.push(tooltip);
    Ok(1)
}
