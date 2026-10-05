//! Spell-state probe globals backed by `SimState`.
//!
//! Migrates 10 entries off `GLOBAL_FALSE_STUBS`:
//!
//! - `IsCurrentSpell(id)`                 — current cast matches id
//! - `IsCurrentAction(slot)`              — action_bars[slot] matches
//!                                            the current cast
//! - `IsSpellKnown(id)`                   — known_spells.contains(id)
//! - `IsSpellKnownOrOverridesKnown(id)`   — alias of IsSpellKnown (sim
//!                                            has no override tree)
//! - `IsSpellInRange(idOrName, unit)`     — unit in range AND spell known
//! - `IsItemInRange(itemId, unit)`        — unit in range (sim has no
//!                                            per-item range data)
//! - `IsUsableSpell(idOrName)`            — known AND not on cooldown
//! - `IsHarmfulSpell(id)`                 — harmful_spells.contains(id)
//! - `IsHelpfulSpell(id)`                 — helpful_spells.contains(id)
//! - `HasPetSpells()`                     — pet_spells non-empty

use crate::c_api::item_spell::c_item_is_item_in_range;
use crate::c_api::item_spell::helpers::unit_is_reachable;
use crate::lua_api::SimState;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_u32(state: &mut LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

/// `IsCurrentSpell(spellId)` — true when the active cast's spell_id
/// matches. Nil cast slot returns false.
fn is_current_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let match_cast = borrow_state(state)?
        .casting
        .as_ref()
        .is_some_and(|c| c.spell_id == id);
    state.push(Val::Bool(match_cast));
    Ok(1)
}

/// `IsCurrentAction(slot)` — true when the action bar slot's spell id
/// matches the active cast.
fn is_current_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let match_cast = {
        let st = borrow_state(state)?;
        st.casting
            .as_ref()
            .zip(st.action_bars.get(&slot).copied())
            .is_some_and(|(cast, bar_spell)| cast.spell_id == bar_spell)
    };
    state.push(Val::Bool(match_cast));
    Ok(1)
}

/// `IsSpellKnown(spellId)` — `known_spells` set membership.
fn is_spell_known(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let known = {
        let st = borrow_state(state)?;
        is_known_spell(&st, id)
    };
    state.push(Val::Bool(known));
    Ok(1)
}

fn is_known_spell(st: &SimState, spell_id: u32) -> bool {
    st.known_spells.contains(&spell_id)
}

/// `IsSpellKnownOrOverridesKnown(spellId)` — alias of `IsSpellKnown`.
fn is_spell_known_or_overrides(state: &mut LuaState) -> LuaResult<u32> {
    is_spell_known(state)
}

/// `IsSpellInRange(idOrName, unit)` — true when the unit exists and
/// the spell is known. Per-spell range tables are not modelled.
fn is_spell_in_range(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let in_range = {
        let st = borrow_state(state)?;
        is_known_spell(&st, id) && unit_is_reachable(&st, &unit)
    };
    state.push(Val::Bool(in_range));
    Ok(1)
}

/// `IsUsableSpell(idOrName)` — known AND no active cooldown.
#[cfg(not(feature = "retail-12-0-0"))]
fn is_usable_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        state.push(Val::Bool(false));
        return Ok(2);
    };
    let (usable, no_mana) = {
        let st = borrow_state(state)?;
        let known = is_known_spell(&st, id);
        let on_cooldown = st
            .spell_cooldowns
            .get(&id)
            .is_some_and(|cd| cd.duration > 0.0);
        (known && !on_cooldown, false)
    };
    state.push(Val::Bool(usable));
    state.push(Val::Bool(no_mana));
    Ok(2)
}

/// `IsHarmfulSpell(spellId)` — `harmful_spells` set membership.
fn is_harmful_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let harmful = {
        let st = borrow_state(state)?;
        is_harmful_spell_id(&st, id)
    };
    state.push(Val::Bool(harmful));
    Ok(1)
}

fn is_harmful_spell_id(st: &SimState, spell_id: u32) -> bool {
    st.harmful_spells.contains(&spell_id)
}

/// `IsHelpfulSpell(spellId)` — `helpful_spells` set membership.
fn is_helpful_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = stack_u32(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let helpful = {
        let st = borrow_state(state)?;
        is_helpful_spell_id(&st, id)
    };
    state.push(Val::Bool(helpful));
    Ok(1)
}

fn is_helpful_spell_id(st: &SimState, spell_id: u32) -> bool {
    st.helpful_spells.contains(&spell_id)
}

/// `HasPetSpells()` — retail returns `(count, petType)` when a pet is
/// active, `nil` otherwise. The sim returns `count` when `pet_spells`
/// is non-empty else `nil` (callers only check truthiness).
fn has_pet_spells(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.pet_spells.len();
    if count == 0 {
        state.push(Val::Nil);
    } else {
        state.push(Val::Num(count as f64));
    }
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "IsCurrentSpell", is_current_spell)?;
    LuaApiMut::register_function(lua, "IsCurrentAction", is_current_action)?;
    LuaApiMut::register_function(lua, "IsSpellKnown", is_spell_known)?;
    LuaApiMut::register_function(
        lua,
        "IsSpellKnownOrOverridesKnown",
        is_spell_known_or_overrides,
    )?;
    LuaApiMut::register_function(lua, "IsSpellInRange", is_spell_in_range)?;
    LuaApiMut::register_function(lua, "IsItemInRange", c_item_is_item_in_range)?;
    #[cfg(not(feature = "retail-12-0-0"))]
    LuaApiMut::register_function(lua, "IsUsableSpell", is_usable_spell)?;
    LuaApiMut::register_function(lua, "IsHarmfulSpell", is_harmful_spell)?;
    LuaApiMut::register_function(lua, "IsHelpfulSpell", is_helpful_spell)?;
    LuaApiMut::register_function(lua, "HasPetSpells", has_pet_spells)?;
    Ok(())
}
