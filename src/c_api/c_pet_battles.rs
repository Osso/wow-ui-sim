//! State-backed `C_PetBattles` identity and pet-type probes.

use crate::c_api::ensure_namespace;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::{FromStack, IntoStack, table_set_rust_fn_static};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_pet_battles_surface(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PetBattles")?;
    table_set_rust_fn_static(state, namespace, "GetPetSpeciesID", get_pet_species_id)?;
    table_set_rust_fn_static(state, namespace, "GetPetType", get_pet_type)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::Mists
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, namespace, "GetIcon", get_icon)?;
        table_set_rust_fn_static(state, namespace, "GetName", get_name)?;
    }
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists {
        table_set_rust_fn_static(state, namespace, "GetDisplayID", get_display_id)?;
    }
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, namespace, "CanPetSwapIn", can_pet_swap_in)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "CanActivePetSwapOut",
            can_active_pet_swap_out,
        )?;
        table_set_rust_fn_static(state, namespace, "IsSkipAvailable", is_skip_available)?;
    }
    Ok(())
}

fn can_pet_swap_in(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let index = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok());
    let can_swap = if let Some(index) = index.filter(|index| *index < 3) {
        borrow_state(state)?.pet_battles.can_swap_in[index]
    } else {
        false
    };
    state.push(Val::Bool(can_swap));
    Ok(1)
}

// INFERRED (R88cl): the UI combines this turn permission with per-slot legality.
fn can_active_pet_swap_out(state: &mut LuaState) -> LuaResult<u32> {
    let can_swap_out = borrow_state(state)?.pet_battles.can_active_pet_swap_out;
    state.push(Val::Bool(can_swap_out));
    Ok(1)
}

fn is_skip_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?.pet_battles.skip_available;
    state.push(Val::Bool(available));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        builder.set_function("SetPetBattleActionLegality", set_action_legality)
    } else {
        Ok(builder)
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_action_legality(state: &mut LuaState) -> LuaResult<u32> {
    let legality = [
        bool_arg(state, 1)?,
        bool_arg(state, 2)?,
        bool_arg(state, 3)?,
    ];
    let can_swap_out = bool_arg(state, 4)?;
    let skip_available = bool_arg(state, 5)?;
    let mut sim = borrow_state_mut(state)?;
    sim.pet_battles.can_swap_in = legality;
    sim.pet_battles.can_active_pet_swap_out = can_swap_out;
    sim.pet_battles.skip_available = skip_available;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn bool_arg(state: &LuaState, index: i32) -> LuaResult<bool> {
    match stack_val(state, index) {
        Val::Bool(value) => Ok(value),
        _ => Err(rilua::runtime_error(
            "pet battle legality requires booleans",
        )),
    }
}

fn get_display_id(state: &mut LuaState) -> LuaResult<u32> {
    let owner = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let index = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok());
    let display_id = {
        let sim = borrow_state(state)?;
        let pets = match owner {
            1 => Some(&sim.pet_battles.player_pets),
            2 => Some(&sim.pet_battles.enemy_pets),
            _ => None,
        };
        pets.and_then(|pets| index.and_then(|index| pets.get(index)))
            .and_then(|pet| pet.display_id)
    };
    // One nil represents unknown simulator input, not verified native return arity.
    state.push(display_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn get_name(state: &mut LuaState) -> LuaResult<u32> {
    let owner = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let Some((name, species_name)) = lookup_names(state, owner, slot)? else {
        // Documented MayReturnNothing representation, not native invalid-input parity.
        return Ok(0);
    };
    let name = create_string(state, &name);
    let species_name = create_string(state, &species_name);
    state.push(name);
    state.push(species_name);
    Ok(2)
}

fn lookup_names(state: &LuaState, owner: i32, slot: i32) -> LuaResult<Option<(String, String)>> {
    let index = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok());
    let sim = borrow_state(state)?;
    let pets = match owner {
        1 => Some(&sim.pet_battles.player_pets),
        2 => Some(&sim.pet_battles.enemy_pets),
        _ => None,
    };
    let Some(pet) = pets.and_then(|pets| index.and_then(|index| pets.get(index))) else {
        return Ok(None);
    };
    // Battle names are already effective display names, independent of journal aliases.
    // Missing species metadata is a simulator input gap; never synthesize a species name.
    let species = u32::try_from(pet.species_id).ok();
    Ok(sim
        .world
        .pets
        .iter()
        .find(|entry| Some(entry.species_id) == species)
        .map(|entry| (pet.name.clone(), entry.name.clone())))
}

fn get_icon(state: &mut LuaState) -> LuaResult<u32> {
    let owner = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let index = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok());
    let icon = {
        let sim = borrow_state(state)?;
        let pets = match owner {
            1 => Some(&sim.pet_battles.player_pets),
            2 => Some(&sim.pet_battles.enemy_pets),
            _ => None,
        };
        let species = pets
            .and_then(|pets| index.and_then(|index| pets.get(index)))
            .and_then(|pet| u32::try_from(pet.species_id).ok());
        species.and_then(|species| {
            sim.world
                .pets
                .iter()
                .find(|pet| pet.species_id == species)
                .map(|pet| pet.icon)
        })
    };
    icon.into_stack(state)
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
