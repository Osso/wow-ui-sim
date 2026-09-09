//! Rilua A_Admin handlers — Action bars, Bags.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::PetActionSlot;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Action bars ───────────────────────────────────────────────────────────────

pub(super) fn set_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let spell_id = u32::from_stack(state, 2)?;
    let mut state = borrow_state_mut(state)?;
    crate::c_api::action_macros::clear_slot(&mut state, slot);
    state.action_bars.insert(slot, spell_id);
    Ok(0)
}

pub(super) fn set_pet_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let texture = String::from_stack(state, 3)?;
    let spell_id = Option::<u32>::from_stack(state, 4)?;
    let passive = Option::<bool>::from_stack(state, 5)?.unwrap_or(false);
    let Some(index) = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok())
    else {
        return Ok(0);
    };

    let mut sim = borrow_state_mut(state)?;
    if let Some(action) = sim.pet_actions.get_mut(index) {
        *action = PetActionSlot {
            has_action: true,
            name: Some(name),
            texture: Some(texture),
            is_token: false,
            is_active: false,
            auto_cast_allowed: !passive,
            auto_cast_enabled: false,
            spell_id,
            passive,
            cooldown: None,
        };
    }
    drop(sim);

    fire_named_event_state(state, "PET_UI_UPDATE", &[]);
    fire_named_event_state(state, "PET_BAR_UPDATE", &[]);
    Ok(0)
}

pub(super) fn clear_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let mut state = borrow_state_mut(state)?;
    crate::c_api::action_macros::clear_slot(&mut state, slot);
    Ok(0)
}

pub(super) fn clear_action_bars(state: &mut LuaState) -> LuaResult<u32> {
    let mut state = borrow_state_mut(state)?;
    state.action_macros.clear();
    state.action_bars.clear();
    state.action_outfits.clear();
    state.equipped_gear_outfit_action_slots.clear();
    Ok(0)
}

// ── Bags ──────────────────────────────────────────────────────────────────────

pub(super) fn add_bag_item(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::BagItem;
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = u32::from_stack(state, 3)?;
    let stack = Option::<i32>::from_stack(state, 4)?;
    borrow_state_mut(state)?.bag_items.insert(
        (bag, slot),
        BagItem {
            item_id,
            stack_count: stack.unwrap_or(1),
            hyperlink: None,
        },
    );
    Ok(0)
}

pub(super) fn remove_bag_item(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    borrow_state_mut(state)?.bag_items.remove(&(bag, slot));
    Ok(0)
}

pub(super) fn clear_bags(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.bag_items.clear();
    Ok(0)
}

pub(super) fn set_merchant_items(state: &mut LuaState) -> LuaResult<u32> {
    let items = match stack_val(state, 1) {
        Val::Table(items_ref) => state
            .gc
            .tables
            .get(items_ref)
            .map(|table| {
                table
                    .array_slice()
                    .iter()
                    .filter_map(|item| match item {
                        Val::Num(value) if *value > 0.0 => Some(*value as u32),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Val::Num(value) if value > 0.0 => vec![value as u32],
        _ => Vec::new(),
    };

    borrow_state_mut(state)?.merchant_items = items;
    Ok(0)
}

pub(super) fn add_guild_bank_item(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::BagItem;
    let tab = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = u32::from_stack(state, 3)?;
    let stack = Option::<i32>::from_stack(state, 4)?;
    borrow_state_mut(state)?.guild_bank_items.insert(
        (tab, slot),
        BagItem {
            item_id,
            stack_count: stack.unwrap_or(1),
            hyperlink: None,
        },
    );
    Ok(0)
}

pub(super) fn clear_guild_bank(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.guild_bank_items.clear();
    Ok(0)
}
