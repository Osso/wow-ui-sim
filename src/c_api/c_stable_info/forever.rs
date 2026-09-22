//! Bounded Camelot stable reads. Scenario defaults are simulator guesses, not native observations.

use std::collections::BTreeMap;

use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Pets keyed by the public one-based slot: current pet = 1, stabled pets = 2..=3.
/// Count includes every stored pet, including the current slot (inferred policy).
#[derive(Debug, Clone)]
pub struct StableReadState {
    pub owned_slots: u32,
    pub next_slot_cost: u64,
    pub pets: BTreeMap<u32, PetInfo>,
}

impl Default for StableReadState {
    fn default() -> Self {
        // Explicit empty, fully-unlocked scenario; capacity alone does not imply ownership.
        Self {
            owned_slots: 2,
            next_slot_cost: 0,
            pets: BTreeMap::new(),
        }
    }
}

/// Cached Forever StableInfoDocumentation PetInfo fields; slotID comes from the map key.
#[derive(Debug, Clone)]
pub struct PetInfo {
    pub icon: u32,
    pub name: String,
    pub level: u32,
    pub family_name: String,
    pub specialization: String,
    pub pet_type: String,
    pub pet_abilities: Vec<u32>,
    pub spec_abilities: Vec<u32>,
    pub display_id: u32,
    pub is_favorite: bool,
    pub is_exotic: bool,
    pub ui_model_scene_id: u32,
    pub pet_number: u32,
    pub creature_id: u32,
    pub spec_id: u32,
    pub loyalty_level: u32,
    pub loyalty_name: String,
    pub happiness_level: u32,
    pub experience: u32,
    pub experience_needed: u32,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_StableInfo")?;
    table_set_rust_fn_static(state, namespace, "GetNumStableSlots", get_num_stable_slots)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNextStableSlotCost",
        get_next_slot_cost,
    )?;
    table_set_rust_fn_static(state, namespace, "GetNumStablePets", get_num_stable_pets)?;
    table_set_rust_fn_static(state, namespace, "GetStablePetInfo", get_stable_pet_info)
}

fn get_num_stable_slots(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.stable_reads.owned_slots;
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_next_slot_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = borrow_state(state)?.stable_reads.next_slot_cost;
    state.push(Val::Num(cost as f64));
    Ok(1)
}

fn get_num_stable_pets(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.stable_reads.pets.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_stable_pet_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let pet = {
        let model = borrow_state(state)?;
        u32::try_from(index)
            .ok()
            .filter(|index| *index > 0)
            .and_then(|index| model.stable_reads.pets.get(&index).cloned())
    };
    let result = match pet {
        Some(pet) => pet_info_table(state, index as u32, &pet),
        None => Val::Nil,
    };
    state.push(result);
    Ok(1)
}

fn pet_info_table(state: &mut LuaState, slot: u32, pet: &PetInfo) -> Val {
    let table = create_table(state);
    for (name, value) in [
        ("slotID", slot),
        ("icon", pet.icon),
        ("level", pet.level),
        ("displayID", pet.display_id),
        ("uiModelSceneID", pet.ui_model_scene_id),
        ("petNumber", pet.pet_number),
        ("creatureID", pet.creature_id),
        ("specID", pet.spec_id),
        ("loyaltyLevel", pet.loyalty_level),
        ("happinessLevel", pet.happiness_level),
        ("experience", pet.experience),
        ("experienceNeeded", pet.experience_needed),
    ] {
        table_set(state, table, name, Val::Num(value as f64));
    }
    for (name, value) in [
        ("name", pet.name.as_str()),
        ("familyName", pet.family_name.as_str()),
        ("specialization", pet.specialization.as_str()),
        ("type", pet.pet_type.as_str()),
        ("loyaltyName", pet.loyalty_name.as_str()),
    ] {
        let string = Val::Str(state.gc.intern_string(value.as_bytes()));
        table_set(state, table, name, string);
    }
    table_set(state, table, "isFavorite", Val::Bool(pet.is_favorite));
    table_set(state, table, "isExotic", Val::Bool(pet.is_exotic));
    for (name, values) in [
        ("petAbilities", &pet.pet_abilities),
        ("specAbilities", &pet.spec_abilities),
    ] {
        let array = create_table(state);
        for (index, value) in values.iter().enumerate() {
            set_table_array(state, array, (index + 1) as i64, Val::Num(*value as f64));
        }
        table_set(state, table, name, array);
    }
    table
}
