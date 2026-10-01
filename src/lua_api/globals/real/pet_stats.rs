//! Legacy hunter / warlock pet-stat probe globals backed by `SimState`.
//!
//! Migrates 4 entries off `GLOBAL_ZERO_STUBS`:
//!
//! - `GetPetExperience()`    → `(pet.xp, pet.xp_max)`
//! - `GetPetHappiness()`     → `(pet.happiness, pet.damage_percent,
//!   pet.loyalty_rate)`
//! - `GetPetLoyalty()`       → `pet.loyalty_label` (string, or nil
//!   when empty)
//! - `GetPetTimeInCombat()`  → `pet.time_in_combat`
//! - `GetPetSpellBonusDamage()` → 0 unless a fuller pet caster stat model is
//!   added.
//!
//! The happiness / loyalty APIs were removed from retail in Cataclysm
//! and addons that still probe them expect zeros or nil. The default
//! `PetState::default()` matches that — tests can seed the struct to
//! exercise classic code paths.

use crate::c_api::c_secrets::push_stat_number;
use crate::lua_api::methods::{borrow_state, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn get_pet_experience(state: &mut LuaState) -> LuaResult<u32> {
    let pet = borrow_state(state)?.pet.clone();
    state.push(Val::Num(pet.xp as f64));
    state.push(Val::Num(pet.xp_max as f64));
    Ok(2)
}

fn get_pet_happiness(state: &mut LuaState) -> LuaResult<u32> {
    let pet = borrow_state(state)?.pet.clone();
    state.push(Val::Num(pet.happiness as f64));
    state.push(Val::Num(pet.damage_percent as f64));
    state.push(Val::Num(pet.loyalty_rate as f64));
    Ok(3)
}

fn get_pet_loyalty(state: &mut LuaState) -> LuaResult<u32> {
    let label = borrow_state(state)?.pet.loyalty_label.clone();
    if label.is_empty() {
        state.push(Val::Nil);
    } else {
        let val = create_string(state, &label);
        state.push(val);
    }
    Ok(1)
}

fn get_pet_time_in_combat(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = borrow_state(state)?.pet.time_in_combat as f64;
    state.push(Val::Num(seconds));
    Ok(1)
}

fn get_pet_spell_bonus_damage(state: &mut LuaState) -> LuaResult<u32> {
    push_stat_number(state, 0.0)?;
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetPetExperience", get_pet_experience)?;
    LuaApiMut::register_function(lua, "GetPetHappiness", get_pet_happiness)?;
    LuaApiMut::register_function(lua, "GetPetLoyalty", get_pet_loyalty)?;
    LuaApiMut::register_function(lua, "GetPetTimeInCombat", get_pet_time_in_combat)?;
    LuaApiMut::register_function(lua, "GetPetSpellBonusDamage", get_pet_spell_bonus_damage)?;
    Ok(())
}
