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
//! - `GetPetSpellBonusDamage()` → explicit `pet.spell_bonus_damage`, 0 when unset.
//!
//! Patch 4.1.0 removed happiness from retail. Its registration and handler
//! remain available to non-retail profiles; existing pet state is preserved.
//! Loyalty is outside that page's removal list and unchanged.

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

#[cfg(not(feature = "client-retail"))]
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
    let bonus = borrow_state(state)?.pet.spell_bonus_damage.unwrap_or(0.0);
    push_stat_number(state, bonus)?;
    Ok(1)
}

#[cfg(feature = "client-retail")]
fn get_pet_melee_haste(state: &mut LuaState) -> LuaResult<u32> {
    let haste = borrow_state(state)?.pet.melee_haste_pct.unwrap_or(0.0);
    push_stat_number(state, haste)?;
    Ok(1)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "client-retail")]
    LuaApiMut::register_function(lua, "GetPetMeleeHaste", get_pet_melee_haste)?;
    LuaApiMut::register_function(lua, "GetPetExperience", get_pet_experience)?;
    #[cfg(not(feature = "client-retail"))]
    LuaApiMut::register_function(lua, "GetPetHappiness", get_pet_happiness)?;
    LuaApiMut::register_function(lua, "GetPetLoyalty", get_pet_loyalty)?;
    LuaApiMut::register_function(lua, "GetPetTimeInCombat", get_pet_time_in_combat)?;
    LuaApiMut::register_function(lua, "GetPetSpellBonusDamage", get_pet_spell_bonus_damage)?;
    Ok(())
}
