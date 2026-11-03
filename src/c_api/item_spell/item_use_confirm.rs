//! Retail item-use confirmation actions (WM44-C, WM44-D).
//!
//! Confirmed use enters the same container-use request model as
//! `C_Container.UseContainerItem`; casting and server results are not modeled.

use super::item_equip::find_identity_slot;
use crate::c_api::container_inventory::submit_container_use;
use crate::c_api::item_upgrade_eligibility::ItemSlot;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

/// `ConfirmOnUse` and `ConfirmNoRefundOnUse` share one helper and read no arguments.
pub(super) fn confirm_on_use(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_pending_use(state)?;
    Ok(0)
}

/// Only `itemconsume` uses the pending item. The `itemenchant`, `gem` and
/// `spellenchant` branches read enchant, socket and spell pending state that no
/// simulator action produces, so they find nothing to submit (WM44-D).
pub(super) fn end_bound_tradeable(state: &mut LuaState) -> LuaResult<u32> {
    let branch = String::from_stack(state, 1)?;
    if branch == "itemconsume" {
        dispatch_pending_use(state)?;
    }
    Ok(0)
}

/// Dispatch the resolved pending item, then clear it even after a miss or
/// refusal (WM44-C). Equipped-item use is not modeled.
fn dispatch_pending_use(state: &mut LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let slot = sim
        .pending_item_use
        .and_then(|identity| find_identity_slot(&sim, identity));
    if let Some(ItemSlot::Bag(bag, index)) = slot {
        submit_container_use(&mut sim, bag, index);
    }
    sim.pending_item_use = None;
    Ok(())
}
