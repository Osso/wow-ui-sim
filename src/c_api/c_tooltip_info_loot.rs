//! Supplied loot and buyback tooltip lookups.
//!
//! Retail and PTR TooltipInfoDocumentation.lua:172,554 declare these getters.
use crate::c_api::tooltip_getter_args::one_based_index;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetLootItem", get_loot_item)?;
    table_set_rust_fn_static(state, namespace, "GetLootRollItem", get_loot_roll_item)?;
    table_set_rust_fn_static(state, namespace, "GetBuybackItem", get_buyback_item)
}

/// WM97 DQ7: select the current roll by signed ID and submit its stored descriptor.
fn get_loot_roll_item(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let descriptor = {
        let model = borrow_state(state)?;
        model
            .world
            .loot_rolls
            .get(&roll_id)
            .and_then(|roll| roll.descriptor.clone())
    };
    let Some(descriptor) = descriptor else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let player_level = {
        let model = borrow_state(state)?;
        let player = &model.player;
        if player.effective_level != 0 {
            player.effective_level
        } else {
            player.level
        }
    };
    let item = descriptor
        .resolve(player_level)
        .map_err(|error| rilua::runtime_error(format!("C_TooltipInfo.GetLootRollItem: {error}")))?;
    let tooltip = tooltip_for_item_context(state, descriptor.item_id, item.as_ref());
    state.push(tooltip);
    Ok(1)
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
    publish_item(state, item_id)
}

fn get_buyback_item(state: &mut LuaState) -> LuaResult<u32> {
    // Round 6 §GetBuybackItem(a): one-based, subtract-then-truncate index. Index 0
    // and every index at or past the live buyback count share the miss arm.
    let index = one_based_index(
        state,
        1,
        "Usage: local data = C_TooltipInfo.GetBuybackItem(index)",
    )?;
    let item_id = {
        let model = borrow_state(state)?;
        index.and_then(|index| {
            model
                .merchant_buyback_items
                .get(index)
                .map(|item| item.item_id)
        })
    };
    // Round 6 §GetBuybackItem(b): an unresolvable item is one of the same-arm
    // miss conditions, so a supplied entry never publishes an empty tooltip.
    let Some((item_id, item)) =
        item_id.and_then(|item_id| crate::items::get_item(item_id).map(|item| (item_id, item)))
    else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    state.push(tooltip);
    Ok(1)
}

fn publish_item(state: &mut LuaState, item_id: Option<u32>) -> LuaResult<u32> {
    let Some(item_id) = item_id else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context(state, item_id, crate::items::get_item(item_id));
    state.push(tooltip);
    Ok(1)
}
