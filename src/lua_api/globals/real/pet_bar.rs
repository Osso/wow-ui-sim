//! State-backed pet action-bar globals consumed by `Blizzard_ActionBar/Shared/PetActionBar.lua`.
//!
//! Mirrors the live API shape:
//!
//! - `GetNumPetActions()` → `state.pet_actions.len()` (always 10).
//! - `GetPetActionInfo(index)` → 9-tuple
//!   `(name, texture, isToken, isActive, autoCastAllowed, autoCastEnabled,
//!   spellID, _unused, passive)`. Empty slots return
//!   `(nil, nil, false, false, false, false, nil, false, false)` to match
//!   the bootstrap stub's previous shape.
//! - `GetPetActionCooldown(index)` → `(start, duration, enable)` from
//!   `slot.cooldown`. Missing entries report `(0, 0, 1)`.
//! - `GetPetActionSlotUsable(index)` → true for bound pet action slots.
//! - `CastPetAction(index, target?)` → toggles `is_active` on the indexed
//!   slot, fires `PET_BAR_UPDATE`. Out-of-range / passive slots no-op.
//! - `TogglePetAutocast(index)` → flips `auto_cast_enabled` on the indexed
//!   slot if `auto_cast_allowed`; fires `PET_BAR_UPDATE`.
//! - `CancelPetPossess()` → clears the active flag on every slot, fires
//!   `PET_BAR_UPDATE`. Possession state itself is not modeled.
//! - `PetHasActionBar()` → true when any slot has `has_action = true`.
//! - `HasPetUI()` → `(hasPetUI, canGainXP)` from action slots / pet XP state.
//!
//! The `runtime_surface_bootstrap.lua` `if ... == nil` guards on
//! `GetPetActionInfo`, `GetPetActionCooldown`, `PetHasActionBar`, and
//! per-profile `HasPetUI` bootstrap guards are
//! now no-ops — registration here runs before the bootstrap script.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::PetActionSlot;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn slot_index_to_zero_based(index: i32) -> Option<usize> {
    usize::try_from(index.checked_sub(crate::lua_api::state::FIRST_PET_ACTION_SLOT)?).ok()
}

fn push_optional_string(state: &mut LuaState, value: Option<&str>) {
    match value {
        Some(s) => {
            let v = create_string(state, s);
            state.push(v);
        }
        None => state.push(Val::Nil),
    }
}

fn push_pet_action_info(state: &mut LuaState, slot: &PetActionSlot) -> u32 {
    push_optional_string(state, slot.name.as_deref());
    push_optional_string(state, slot.texture.as_deref());
    state.push(Val::Bool(slot.is_token));
    state.push(Val::Bool(slot.is_active));
    state.push(Val::Bool(slot.auto_cast_allowed));
    state.push(Val::Bool(slot.auto_cast_enabled));
    match slot.spell_id {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    state.push(Val::Bool(false));
    state.push(Val::Bool(slot.passive));
    9
}

fn push_empty_pet_action_info(state: &mut LuaState) -> u32 {
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    9
}

/// `GetNumPetActions()` — fixed at `state.pet_actions.len()` (10 by default).
fn get_num_pet_actions(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.pet_actions.len() as f64;
    state.push(Val::Num(count));
    Ok(1)
}

/// `GetPetActionInfo(index)` — 9-tuple. Out-of-range returns the empty
/// 9-tuple so consumers can pattern-match without nil-guarding the index.
fn get_pet_action_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = stack_i32(state, 1) else {
        return Ok(push_empty_pet_action_info(state));
    };
    let Some(zero_based) = slot_index_to_zero_based(index) else {
        return Ok(push_empty_pet_action_info(state));
    };
    let slot = borrow_state(state)?.pet_actions.get(zero_based).cloned();
    let Some(slot) = slot else {
        return Ok(push_empty_pet_action_info(state));
    };
    if !slot.has_action {
        return Ok(push_empty_pet_action_info(state));
    }
    Ok(push_pet_action_info(state, &slot))
}

/// `GetPetActionCooldown(index)` — `(start, duration, enable)`. Missing or
/// expired entries report `(0, 0, 1)`.
fn get_pet_action_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    let cooldown = slot_index_to_zero_based(index)
        .and_then(|zero_based| {
            borrow_state(state)
                .ok()?
                .pet_actions
                .get(zero_based)
                .cloned()
        })
        .and_then(|slot| slot.cooldown);
    let (start, duration) = cooldown
        .map(|cd| (cd.start, cd.duration))
        .unwrap_or((0.0, 0.0));
    state.push(Val::Num(start));
    state.push(Val::Num(duration));
    state.push(Val::Num(1.0));
    Ok(3)
}

/// `GetPetActionSlotUsable(index)` — true when the indexed slot has a
/// renderable/bound action. Classic pet bars use this only to dim icons.
fn get_pet_action_slot_usable(state: &mut LuaState) -> LuaResult<u32> {
    let usable = stack_i32(state, 1)
        .and_then(slot_index_to_zero_based)
        .and_then(|index| {
            borrow_state(state)
                .ok()
                .and_then(|sim| sim.pet_actions.get(index).map(|slot| slot.has_action))
        })
        .unwrap_or(false);
    state.push(Val::Bool(usable));
    Ok(1)
}

/// Apply `mutate` to the pet slot at the 1-based Lua `index` argument and
/// fire `PET_BAR_UPDATE` on success. `mutate` returns `true` to publish the
/// event, `false` to no-op (slot empty / unsupported / passive).
fn mutate_slot_and_fire<F>(state: &mut LuaState, mutate: F) -> LuaResult<u32>
where
    F: FnOnce(&mut PetActionSlot) -> bool,
{
    let Some(index) = stack_i32(state, 1) else {
        return Ok(0);
    };
    let Some(zero_based) = slot_index_to_zero_based(index) else {
        return Ok(0);
    };
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let Some(slot) = sim.pet_actions.get_mut(zero_based) else {
            return Ok(0);
        };
        mutate(slot)
    };
    if changed {
        fire_named_event_state(state, "PET_BAR_UPDATE", &[]);
    }
    Ok(0)
}

/// `CastPetAction(index, target?)` — toggles `is_active` on the indexed
/// slot and fires `PET_BAR_UPDATE`. Out-of-range / empty / passive slots
/// are silent no-ops.
fn cast_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    mutate_slot_and_fire(state, |slot| {
        if !slot.has_action || slot.passive {
            return false;
        }
        slot.is_active = !slot.is_active;
        true
    })
}

/// `TogglePetAutocast(index)` — flips `auto_cast_enabled` if the slot
/// supports auto-cast. Fires `PET_BAR_UPDATE` on change.
fn toggle_pet_autocast(state: &mut LuaState) -> LuaResult<u32> {
    mutate_slot_and_fire(state, |slot| {
        if !slot.has_action || !slot.auto_cast_allowed {
            return false;
        }
        slot.auto_cast_enabled = !slot.auto_cast_enabled;
        true
    })
}

/// `CancelPetPossess()` — clears the active flag on every slot. The
/// possession-state model is not part of the simulator (no vehicle/charm
/// driver), so this only resets the visual checked state on the bar.
fn cancel_pet_possess(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        for slot in sim.pet_actions.iter_mut() {
            slot.is_active = false;
        }
    }
    fire_named_event_state(state, "PET_BAR_UPDATE", &[]);
    Ok(0)
}

/// `PetHasActionBar()` — true when any slot has a bound action.
fn pet_has_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let any = borrow_state(state)?
        .pet_actions
        .iter()
        .any(|slot| slot.has_action);
    state.push(Val::Bool(any));
    Ok(1)
}

/// `HasPetUI()` — `(hasPetUI, canGainXP)`. The first return controls
/// CharacterFrame's pet tab visibility; the second controls XP/info widgets
/// in Cataclysm/Mists pet paper-doll code.
fn has_pet_ui(state: &mut LuaState) -> LuaResult<u32> {
    let (has_pet_ui, can_gain_xp) = {
        let sim = borrow_state(state)?;
        let has_pet_ui = sim.pet_actions.iter().any(|slot| slot.has_action);
        (has_pet_ui, has_pet_ui && sim.pet.xp_max > 0)
    };
    state.push(Val::Bool(has_pet_ui));
    state.push(Val::Bool(can_gain_xp));
    Ok(2)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetNumPetActions", get_num_pet_actions)?;
    LuaApiMut::register_function(lua, "GetPetActionInfo", get_pet_action_info)?;
    LuaApiMut::register_function(lua, "GetPetActionCooldown", get_pet_action_cooldown)?;
    LuaApiMut::register_function(lua, "GetPetActionSlotUsable", get_pet_action_slot_usable)?;
    LuaApiMut::register_function(lua, "CastPetAction", cast_pet_action)?;
    LuaApiMut::register_function(lua, "TogglePetAutocast", toggle_pet_autocast)?;
    LuaApiMut::register_function(lua, "CancelPetPossess", cancel_pet_possess)?;
    LuaApiMut::register_function(lua, "PetHasActionBar", pet_has_action_bar)?;
    LuaApiMut::register_function(lua, "HasPetUI", has_pet_ui)?;
    Ok(())
}
