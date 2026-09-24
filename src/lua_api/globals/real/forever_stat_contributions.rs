//! Coarse Forever primary-stat contributions for Camelot PaperDoll tooltips.
//! These coefficients are simulator assumptions, not native client formulas.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const AGILITY_STAT: i32 = 2;
const INTELLECT_STAT: i32 = 4;
const ATTRIBUTE_POINTS_PER_CRIT_FRACTION: f64 = 10_000.0;
const HEALTH_REGEN_PER_SPIRIT: f64 = 0.2;
const MANA_REGEN_PER_SPIRIT: f64 = 0.1;

fn crit_fraction(stat: i32, value: f64, contributing_stat: i32) -> f64 {
    if stat == contributing_stat {
        value.max(0.0) / ATTRIBUTE_POINTS_PER_CRIT_FRACTION
    } else {
        0.0
    }
}

fn get_crit_chance_from_stat(state: &mut LuaState) -> LuaResult<u32> {
    let stat = i32::from_stack(state, 1)?;
    let value = f64::from_stack(state, 2)?;
    state.push(Val::Num(crit_fraction(stat, value, AGILITY_STAT)));
    Ok(1)
}

fn get_spell_crit_chance_from_stat(state: &mut LuaState) -> LuaResult<u32> {
    let stat = i32::from_stack(state, 1)?;
    let value = f64::from_stack(state, 2)?;
    state.push(Val::Num(crit_fraction(stat, value, INTELLECT_STAT)));
    Ok(1)
}

fn ranged_attack_power_for_stat(class: i32, stat: i32, value: f64) -> f64 {
    if stat != AGILITY_STAT {
        return 0.0;
    }
    let multiplier = match class {
        3 => 2.0,     // Hunter
        1 | 4 => 1.0, // Warrior / Rogue
        _ => 0.0,
    };
    value.max(0.0) * multiplier
}

fn get_ranged_attack_power_for_stat(state: &mut LuaState) -> LuaResult<u32> {
    let stat = i32::from_stack(state, 1)?;
    let value = f64::from_stack(state, 2)?;
    let class = borrow_state(state)?.player.class_index;
    state.push(Val::Num(ranged_attack_power_for_stat(class, stat, value)));
    Ok(1)
}

fn read_player_spirit(state: &LuaState) -> LuaResult<f64> {
    Ok(borrow_state(state)?.player.stats.spirit.max(0.0))
}

fn health_regen_from_spirit(spirit: f64) -> f64 {
    spirit * HEALTH_REGEN_PER_SPIRIT
}

pub(super) fn mana_regen_from_spirit(spirit: f64) -> f64 {
    spirit * MANA_REGEN_PER_SPIRIT
}

fn push_regen(state: &mut LuaState, contribution: f64) -> LuaResult<u32> {
    state.push(Val::Num(contribution));
    state.push(Val::Num(0.0)); // No spirit regeneration in combat is modeled.
    Ok(2)
}

fn get_health_regen_from_spirit(state: &mut LuaState) -> LuaResult<u32> {
    let contribution = health_regen_from_spirit(read_player_spirit(state)?);
    push_regen(state, contribution)
}

fn get_mana_regen_from_spirit(state: &mut LuaState) -> LuaResult<u32> {
    let contribution = mana_regen_from_spirit(read_player_spirit(state)?);
    push_regen(state, contribution)
}

fn get_health_regen(state: &mut LuaState) -> LuaResult<u32> {
    // There is no modeled non-spirit health regeneration.
    get_health_regen_from_spirit(state)
}

const GLOBALS: &[(&str, RustFn)] = &[
    ("GetCritChanceFromStat", get_crit_chance_from_stat),
    (
        "GetSpellCritChanceFromStat",
        get_spell_crit_chance_from_stat,
    ),
    (
        "GetRangedAttackPowerForStat",
        get_ranged_attack_power_for_stat,
    ),
    ("GetHealthRegenFromSpirit", get_health_regen_from_spirit),
    ("GetManaRegenFromSpirit", get_mana_regen_from_spirit),
    ("GetHealthRegen", get_health_regen),
];

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    for &(name, function) in GLOBALS {
        LuaApiMut::register_function(lua, name, function)?;
    }
    Ok(())
}
