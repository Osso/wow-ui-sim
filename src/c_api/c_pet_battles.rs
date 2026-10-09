//! State-backed `C_PetBattles` identity and pet-type probes.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, IntoStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_pet_battles_surface(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PetBattles")?;
    table_set_rust_fn_static(state, namespace, "GetPetSpeciesID", get_pet_species_id)?;
    table_set_rust_fn_static(state, namespace, "GetPetType", get_pet_type)?;
    Ok(())
}

fn get_pet_type(state: &mut LuaState) -> LuaResult<u32> {
    let owner = i32::from_stack(state, 1)?;
    let pet_index = i32::from_stack(state, 2)?;
    let index = pet_index
        .checked_sub(1)
        .and_then(|i| usize::try_from(i).ok());
    let pet_type = {
        let sim = borrow_state(state)?;
        let pets = match owner {
            1 => Some(&sim.pet_battles.player_pets),
            2 => Some(&sim.pet_battles.enemy_pets),
            _ => None,
        };
        pets.and_then(|pets| index.and_then(|i| pets.get(i)))
            .map(|pet| pet.pet_type)
    };
    pet_type.into_stack(state)
}

fn get_pet_species_id(state: &mut LuaState) -> LuaResult<u32> {
    let owner = i32::from_stack(state, 1)?;
    let pet_index = i32::from_stack(state, 2)?;
    let species_id = {
        let sim = borrow_state(state)?;
        let pets = match owner {
            1 => Some(&sim.pet_battles.player_pets),
            2 => Some(&sim.pet_battles.enemy_pets),
            _ => None,
        };
        pets.and_then(|pets| {
            usize::try_from(pet_index - 1)
                .ok()
                .and_then(|index| pets.get(index))
                .map(|pet| pet.species_id)
        })
    };

    state.push(species_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}
