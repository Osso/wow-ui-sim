//! Cooldown / spell-stat probe globals.
//!
//! Migrates 7 entries off `GLOBAL_ZERO_STUBS`:
//!
//! - `GetSpellCooldown(spellID)`           → `(start, duration, enable, modRate)`
//!   from `SimState.spell_cooldowns[spellID]`. GCD bleeds in via the
//!   existing `spell_cooldown_times` helper so action bars show the
//!   GCD sweep.
//! - `GetActionCooldown(slot)`             → same shape, resolving
//!   `action_bars[slot]` to a spell id first.
//! - `GetInventoryItemCooldown(unit, slot)`→ `(start, duration, enable)`
//!   from a new `SimState.inventory_item_cooldowns` map (keyed by
//!   equipment slot id).
//! - `GetSpellBonusDamage(school)`         → integer SP derived from
//!   `PlayerState.stats.intellect`. School arg is informational — the
//!   sim has one bucket.
//! - `GetSpellBonusHealing()`              → same helper, same shape.
//! - `GetSpellAutocast(spellID)`           → `(false, false)` — pet
//!   autocast isn't modelled.
//! - `GetSpellLevelLearned(spellID)`       → 1 for known spells, else 0.

use crate::c_api::c_secrets::push_stat_number;
use crate::lua_api::globals::action_bar_api::spell_cooldown_times;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::collections::HashSet;

fn stack_u32(state: &LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn push_cooldown_quad(state: &mut LuaState, start: f64, duration: f64) {
    state.push(Val::Num(start));
    state.push(Val::Num(duration));
    state.push(Val::Num(1.0)); // enable = 1 (active). 0 = disabled.
    state.push(Val::Num(1.0)); // modRate = 1x (no talent acceleration).
}

/// `GetSpellCooldown(spellID)` — retail: `(start, duration, enable, modRate)`.
fn get_spell_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = stack_u32(state, 1).unwrap_or(0);
    let (start, duration) = {
        let sim = borrow_state(state)?;
        let now = sim.start_time.elapsed().as_secs_f64();
        spell_cooldown_times(&sim, spell_id, now)
    };
    push_cooldown_quad(state, start, duration);
    Ok(4)
}

/// `GetActionCooldown(slot)` — retail: `(start, duration, enable, modRate)`.
/// Resolves `action_bars[slot]` to a spell id and reuses the spell-cooldown
/// helper (which already blends GCD).
fn get_action_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_u32(state, 1).unwrap_or(0);
    let (start, duration) = {
        let sim = borrow_state(state)?;
        match sim.action_bars.get(&slot).copied() {
            Some(spell_id) => {
                let now = sim.start_time.elapsed().as_secs_f64();
                spell_cooldown_times(&sim, spell_id, now)
            }
            None => (0.0, 0.0),
        }
    };
    push_cooldown_quad(state, start, duration);
    Ok(4)
}

/// `GetInventoryItemCooldown(unit, slot)` — retail: `(start, duration, enable)`.
/// The sim only models cooldowns for the player's equipment.
fn get_inventory_item_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_i32(state, 2).unwrap_or(0);
    let sim = borrow_state(state)?;
    let entry = sim.inventory_item_cooldowns.get(&slot).cloned();
    drop(sim);
    let (start, duration) = entry
        .map(|cd| (cd.start, cd.duration))
        .unwrap_or((0.0, 0.0));
    state.push(Val::Num(start));
    state.push(Val::Num(duration));
    state.push(Val::Num(1.0));
    Ok(3)
}

/// Spell-power proxy: intellect is the sim's only spell-scaling stat, so
/// treat it as the "spell bonus" value for both damage and healing.
fn spell_power(state: &mut LuaState) -> f64 {
    borrow_state(state)
        .map(|sim| sim.player.stats.intellect)
        .unwrap_or(0.0)
}

/// `GetSpellBonusDamage(school)` — retail returns the player's
/// spell-power for a given damage school. The sim models a single SP
/// bucket, so the school arg is informational.
fn get_spell_bonus_damage(state: &mut LuaState) -> LuaResult<u32> {
    let sp = spell_power(state);
    push_stat_number(state, sp)?;
    Ok(1)
}

/// `GetSpellBonusHealing()` — retail returns healing spell power.
/// Same single-bucket approximation as `GetSpellBonusDamage`.
fn get_spell_bonus_healing(state: &mut LuaState) -> LuaResult<u32> {
    let sp = spell_power(state);
    push_stat_number(state, sp)?;
    Ok(1)
}

/// `GetSpellAutocast(spellID)` — retail: `(isAutoCastable, isAutoCasting)`.
/// Pet autocast isn't modelled in the sim, so always `(false, false)`.
fn get_spell_autocast(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    Ok(2)
}

/// `GetSpellLevelLearned(spellID)` — retail returns the level at which
/// the player learned the spell. The sim doesn't store per-spell learn
/// levels; report 1 for any known spell, 0 for unknown ones.
fn get_spell_level_learned(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = stack_u32(state, 1).unwrap_or(0);
    let level = match borrow_state(state) {
        Ok(sim) if is_known_spell(&sim.known_spells, spell_id) => 1.0,
        _ => 0.0,
    };
    state.push(Val::Num(level));
    Ok(1)
}

fn is_known_spell(known_spells: &HashSet<u32>, spell_id: u32) -> bool {
    known_spells.contains(&spell_id)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetSpellCooldown", get_spell_cooldown)?;
    LuaApiMut::register_function(lua, "GetActionCooldown", get_action_cooldown)?;
    LuaApiMut::register_function(lua, "GetInventoryItemCooldown", get_inventory_item_cooldown)?;
    LuaApiMut::register_function(lua, "GetSpellBonusDamage", get_spell_bonus_damage)?;
    LuaApiMut::register_function(lua, "GetSpellBonusHealing", get_spell_bonus_healing)?;
    LuaApiMut::register_function(lua, "GetSpellAutocast", get_spell_autocast)?;
    LuaApiMut::register_function(lua, "GetSpellLevelLearned", get_spell_level_learned)?;
    Ok(())
}
