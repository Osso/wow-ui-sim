//! Retail `C_Item.UseItemByName` ordinary path (WM34-B, WM68).
//!
//! The optional target is read but not modeled; the special-item and early
//! `END_BOUND_TRADEABLE` paths have UNKNOWN conditions.

use super::item_equip::{UsePromptInputs, ensure_identity, find_carried_item, find_root_item};
use crate::c_api::container_inventory::submit_container_use;
use crate::c_api::item_upgrade_eligibility::ItemSlot;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state_mut, create_string};
use crate::lua_api::state::SimState;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn use_item_by_name(state: &mut LuaState) -> LuaResult<u32> {
    let value = crate::lua_bridge::stack_val(state, 1);
    let item = rilua::table_security::unwrap_secret(state, value)?;
    let item_id = super::parse_item_id_key(state, item)
        .ok_or_else(|| rilua::runtime_error("C_Item.UseItemByName: expected itemInfo"))?;
    let prompt = {
        let mut sim = borrow_state_mut(state)?;
        let Some(slot) = find_usable_copy(&sim, item_id) else {
            return Ok(0);
        };
        let Some(identity) = ensure_identity(&mut sim, slot) else {
            return Ok(0);
        };
        let prompts = sim
            .item_equip
            .inputs
            .get(&identity)
            .map(|inputs| inputs.use_prompts)
            .unwrap_or_default();
        let prompt = select_prompt(prompts);
        match (prompt, slot) {
            (Some(_), _) => sim.pending_item_use = Some(identity),
            (None, ItemSlot::Bag(bag, index)) => submit_container_use(&mut sim, bag, index),
            // Equipped-item use is not modeled.
            (None, ItemSlot::Equipment(_)) => {}
        }
        prompt
    };
    if let Some(prompt) = prompt {
        publish_prompt(state, prompt)?;
    }
    Ok(0)
}

/// Player, nonzero ID and metadata gates, then player-root slots before
/// carried bags; the first match wins (WM68).
fn find_usable_copy(sim: &SimState, item_id: u32) -> Option<ItemSlot> {
    let has_metadata = super::item_spec_derive::item_spec_input(item_id).is_some();
    if !sim.player.present || item_id == 0 || !has_metadata {
        return None;
    }
    find_root_item(sim, item_id).or_else(|| find_carried_item(sim, item_id))
}

#[derive(Debug, Clone, Copy)]
enum UsePrompt {
    Bind,
    BeforeUse,
    NoRefund,
    EndBoundTradeable,
}

fn select_prompt(prompts: UsePromptInputs) -> Option<UsePrompt> {
    [
        (prompts.bind, UsePrompt::Bind),
        (prompts.before_use, UsePrompt::BeforeUse),
        (prompts.no_refund, UsePrompt::NoRefund),
        (prompts.end_bound_tradeable, UsePrompt::EndBoundTradeable),
    ]
    .into_iter()
    .find_map(|(selected, prompt)| selected.then_some(prompt))
}

/// The first three prompts have no arguments; END_BOUND_TRADEABLE has one.
fn publish_prompt(state: &mut LuaState, prompt: UsePrompt) -> LuaResult<()> {
    let event = match prompt {
        UsePrompt::Bind => "USE_BIND_CONFIRM",
        UsePrompt::BeforeUse => "CONFIRM_BEFORE_USE",
        UsePrompt::NoRefund => "USE_NO_REFUND_CONFIRM",
        UsePrompt::EndBoundTradeable => {
            let branch = create_string(state, "itemconsume");
            return dispatch_event_now(state, "END_BOUND_TRADEABLE", &[branch]);
        }
    };
    dispatch_event_now(state, event, &[] as &[Val])
}
