//! C_PetJournal namespace.

use crate::lua_api::methods::{
    borrow_state, create_string, create_string_static, create_table, table_set, table_set_num,
    val_to_string,
};
use crate::lua_api::state_types::PetData;
use crate::lua_bridge::{FromStack, IntoStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::set_global_val;

const BASE_PET_HEALTH: i32 = 100;
const PET_HEALTH_PER_LEVEL: i32 = 20;
const PET_HEALTH_PER_RARITY: i32 = 10;
const BASE_PET_ATTACK: i32 = 10;
const PET_ATTACK_PER_LEVEL: i32 = 2;
const BASE_PET_SPEED: i32 = 10;
const UNKNOWN_PET_ABILITY_ICON: u32 = 134400;
const PET_ABILITY_LEVELS: [i32; 3] = [1, 2, 4];
const DEFAULT_PET_CARD_MODEL_SCENE_ID: i32 = 596;
const DEFAULT_PET_LOADOUT_MODEL_SCENE_ID: i32 = 596;
// INFERRED: pet experience is not modeled; every owned pet reports 0/100.
const PET_XP: f64 = 0.0;
const PET_MAX_XP: f64 = 100.0;

fn custom_name_val(state: &mut LuaState, pet: &PetData) -> Val {
    match &pet.custom_name {
        Some(name) => create_string(state, name),
        None => Val::Nil,
    }
}

fn pet_get_num_pets(state: &mut LuaState) -> LuaResult<u32> {
    let st = borrow_state(state)?;
    let total = st.world.pets.len() as i32;
    let owned = st.world.pets.iter().filter(|p| p.is_collected).count() as i32;
    drop(st);
    (total, owned).into_stack(state)
}

fn pet_get_num_pet_types(state: &mut LuaState) -> LuaResult<u32> {
    (10i32).into_stack(state)
}

fn pet_get_num_pet_sources(state: &mut LuaState) -> LuaResult<u32> {
    (10i32).into_stack(state)
}

fn pet_get_num_collected_info(state: &mut LuaState) -> LuaResult<u32> {
    let _ = i32::from_stack(state, 1)?;
    let st = borrow_state(state)?;
    let total = st.world.pets.len() as i32;
    let collected = st.world.pets.iter().filter(|pet| pet.is_collected).count() as i32;
    drop(st);
    (collected, total).into_stack(state)
}

fn push_pet_info_by_index(state: &mut LuaState, pet: &PetData) -> u32 {
    let pet_id = create_string(state, &pet.pet_id);
    let name = create_string(state, &pet.name);
    let custom_name = custom_name_val(state, pet);
    state.push(pet_id);
    state.push(Val::Num(pet.species_id as f64));
    state.push(Val::Bool(pet.is_collected));
    state.push(custom_name);
    state.push(Val::Num(pet.level as f64));
    state.push(Val::Bool(pet.is_favorite));
    state.push(Val::Bool(false));
    state.push(name);
    state.push(Val::Num(pet.icon as f64));
    state.push(Val::Num(pet.pet_type as f64));
    10
}

fn push_pet_info_by_pet_id(state: &mut LuaState, pet: &PetData) -> u32 {
    let name = create_string(state, &pet.name);
    let empty = create_string_static(state, "");
    let custom_name = custom_name_val(state, pet);
    state.push(Val::Num(pet.species_id as f64));
    state.push(custom_name);
    state.push(Val::Num(pet.level as f64));
    state.push(Val::Num(PET_XP));
    state.push(Val::Num(PET_MAX_XP));
    state.push(Val::Num(pet.species_id as f64));
    state.push(Val::Bool(pet.is_favorite));
    state.push(name);
    state.push(Val::Num(pet.icon as f64));
    state.push(Val::Num(pet.pet_type as f64));
    state.push(Val::Num(pet.species_id as f64));
    state.push(empty);
    state.push(empty);
    state.push(Val::Bool(false));
    state.push(Val::Bool(pet.quality > 0));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    17
}

fn push_pet_info_by_species_id(state: &mut LuaState, pet: &PetData) -> u32 {
    let name = create_string(state, &pet.name);
    let empty = create_string_static(state, "");
    state.push(name);
    state.push(Val::Num(pet.icon as f64));
    state.push(Val::Num(pet.pet_type as f64));
    state.push(Val::Num(pet.species_id as f64));
    state.push(empty);
    state.push(empty);
    state.push(Val::Bool(false));
    state.push(Val::Bool(pet.quality > 0));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Bool(true));
    state.push(Val::Num(pet.species_id as f64));
    12
}

fn find_pet_by_index(state: &LuaState, index: i32) -> Option<PetData> {
    let st = borrow_state(state).ok()?;
    let pet_index = (index - 1) as usize;
    st.world.pets.get(pet_index).cloned()
}

fn find_pet_by_pet_id(state: &LuaState, pet_id: &str) -> Option<PetData> {
    let st = borrow_state(state).ok()?;
    st.world
        .pets
        .iter()
        .find(|pet| pet.pet_id == pet_id)
        .cloned()
}

fn find_pet_by_species_id(state: &LuaState, species_id: u32) -> Option<PetData> {
    let st = borrow_state(state).ok()?;
    st.world
        .pets
        .iter()
        .find(|pet| pet.species_id == species_id)
        .cloned()
}

fn pet_get_info_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = pet_index_from_stack(state, 1) else {
        return Ok(0);
    };
    let pet = find_pet_by_index(state, index);
    let Some(pet) = pet else {
        return Ok(0);
    };
    Ok(push_pet_info_by_index(state, &pet))
}

fn pet_index_from_stack(state: &mut LuaState, stack_index: i32) -> Option<i32> {
    let value = stack_val(state, stack_index);
    match value {
        Val::Num(index) => Some(index as i32),
        Val::Str(_) => val_to_string(state, value)?.parse::<i32>().ok(),
        _ => None,
    }
}

fn pet_get_info_by_pet_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let pet = match value {
        Val::Str(_) => {
            let Some(pet_id) = val_to_string(state, value) else {
                return Ok(0);
            };
            let species_id = pet_id.parse::<u32>().ok();
            let pet = find_pet_by_pet_id(state, &pet_id);
            pet.or_else(|| {
                species_id.and_then(|species_id| find_pet_by_species_id(state, species_id))
            })
        }
        Val::Num(species_id) => find_pet_by_species_id(state, species_id as u32),
        _ => return Ok(0),
    };
    let Some(pet) = pet else {
        return Ok(0);
    };
    Ok(push_pet_info_by_pet_id(state, &pet))
}

fn pet_get_info_by_species_id(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    let pet = find_pet_by_species_id(state, species_id);
    let Some(pet) = pet else {
        return Ok(0);
    };
    Ok(push_pet_info_by_species_id(state, &pet))
}

/// `PetJournalPetInfo` for one pet. Species queries leave the owned-pet
/// fields (customName, petLevel, xp, maxXP, isFavorite, isWild) nil.
#[cfg(feature = "retail-12-1-0")]
fn push_pet_info_table(state: &mut LuaState, pet: &PetData, owned: bool) -> u32 {
    let info = create_table(state);
    state.push(info);
    let name = create_string(state, &pet.name);
    table_set(state, info, "name", name);
    // INFERRED: no display/creature/source data is modeled; the legacy
    // GetPetInfoByPetID returns use speciesID and empty strings the same way.
    for field in ["sourceText", "description"] {
        let empty = create_string_static(state, "");
        table_set(state, info, field, empty);
    }
    for (field, value) in [
        ("speciesID", f64::from(pet.species_id)),
        ("displayID", f64::from(pet.species_id)),
        ("creatureID", f64::from(pet.species_id)),
        ("icon", f64::from(pet.icon)),
        ("petType", f64::from(pet.pet_type)),
    ] {
        table_set(state, info, field, Val::Num(value));
    }
    for (field, value) in [
        ("canBattle", pet.quality > 0),
        // INFERRED: trade/uniqueness flags are not modeled per species.
        ("tradable", false),
        ("unique", false),
        ("obtainable", true),
        ("canAttachToDecor", pet.can_attach_to_decor),
    ] {
        table_set(state, info, field, Val::Bool(value));
    }
    if let Some(scale) = pet.creature_model_scale {
        table_set(state, info, "creatureModelScale", Val::Num(scale));
    }
    if owned {
        let custom_name = custom_name_val(state, pet);
        table_set(state, info, "customName", custom_name);
        for (field, value) in [
            ("petLevel", f64::from(pet.level)),
            ("xp", PET_XP),
            ("maxXP", PET_MAX_XP),
        ] {
            table_set(state, info, field, Val::Num(value));
        }
        table_set(state, info, "isFavorite", Val::Bool(pet.is_favorite));
        // INFERRED: only owned, non-wild pets are modeled.
        table_set(state, info, "isWild", Val::Bool(false));
    }
    1
}

#[cfg(feature = "retail-12-1-0")]
fn pet_get_info_table_by_species_id(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    let Some(pet) = find_pet_by_species_id(state, species_id) else {
        return Ok(0);
    };
    Ok(push_pet_info_table(state, &pet, false))
}

#[cfg(feature = "retail-12-1-0")]
fn pet_get_info_table_by_pet_id(state: &mut LuaState) -> LuaResult<u32> {
    let pet_id = String::from_stack(state, 1)?;
    let Some(pet) = find_pet_by_pet_id(state, &pet_id) else {
        return Ok(0);
    };
    Ok(push_pet_info_table(state, &pet, true))
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_pet_info_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function(
        "GetPetInfoTableBySpeciesID",
        pet_get_info_table_by_species_id,
    )?
    .set_function("GetPetInfoTableByPetID", pet_get_info_table_by_pet_id)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn register_patch_12_1_pet_info_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    Ok(tb)
}

fn pet_get_model_scene_info_by_species_id(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    if find_pet_by_species_id(state, species_id).is_none() {
        return Ok(0);
    }

    (
        DEFAULT_PET_CARD_MODEL_SCENE_ID,
        DEFAULT_PET_LOADOUT_MODEL_SCENE_ID,
    )
        .into_stack(state)
}

fn find_pet_by_stack_arg(state: &LuaState, index: i32) -> Option<PetData> {
    match stack_val(state, index) {
        Val::Str(value) => {
            let pet_id = val_to_string(state, Val::Str(value))?;
            let species_id = pet_id.parse::<u32>().ok();
            find_pet_by_pet_id(state, &pet_id).or_else(|| {
                species_id.and_then(|species_id| find_pet_by_species_id(state, species_id))
            })
        }
        Val::Num(species_id) => find_pet_by_species_id(state, species_id as u32),
        _ => None,
    }
}

fn pet_get_stats(state: &mut LuaState) -> LuaResult<u32> {
    let Some(pet) = find_pet_by_stack_arg(state, 1) else {
        return Ok(0);
    };

    let level = pet.level.max(1);
    let rarity = pet.quality.max(1);
    let max_health =
        BASE_PET_HEALTH + (level * PET_HEALTH_PER_LEVEL) + (rarity * PET_HEALTH_PER_RARITY);
    let attack = BASE_PET_ATTACK + (level * PET_ATTACK_PER_LEVEL) + rarity;
    let speed = BASE_PET_SPEED + level + rarity;
    state.push(Val::Num(max_health as f64));
    state.push(Val::Num(max_health as f64));
    state.push(Val::Num(attack as f64));
    state.push(Val::Num(speed as f64));
    state.push(Val::Num(rarity as f64));
    Ok(5)
}

fn pet_ability_id(species_id: u32, ability_slot: usize) -> i32 {
    (species_id as i32 * 10) + ability_slot as i32
}

fn species_id_from_ability_id(ability_id: i32) -> Option<u32> {
    let species_id = ability_id / 10;
    (species_id > 0).then_some(species_id as u32)
}

fn pet_get_ability_info(state: &mut LuaState) -> LuaResult<u32> {
    let ability_id = i32::from_stack(state, 1)?;
    let pet =
        species_id_from_ability_id(ability_id).and_then(|id| find_pet_by_species_id(state, id));
    let ability_name = pet
        .as_ref()
        .map(|pet| format!("{} Ability", pet.name))
        .unwrap_or_else(|| "Battle Pet Ability".to_string());
    let ability_icon = pet
        .as_ref()
        .map(|pet| pet.icon)
        .unwrap_or(UNKNOWN_PET_ABILITY_ICON);
    let pet_type = pet.as_ref().map(|pet| pet.pet_type).unwrap_or(1);

    let name = create_string(state, &ability_name);
    state.push(name);
    state.push(Val::Num(ability_icon as f64));
    state.push(Val::Num(pet_type as f64));
    Ok(3)
}

fn push_pet_ability_entry(state: &mut LuaState, species_id: u32, slot: usize, level: i32) -> Val {
    let entry = create_table(state);
    table_set(
        state,
        entry.clone(),
        "abilityID",
        Val::Num(pet_ability_id(species_id, slot) as f64),
    );
    table_set(state, entry.clone(), "level", Val::Num(level as f64));
    entry
}

fn pet_get_ability_list_table(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    let ability_table = create_table(state);

    for (slot, level) in PET_ABILITY_LEVELS.iter().enumerate() {
        let entry = push_pet_ability_entry(state, species_id, slot + 1, *level);
        if let Val::Table(table_ref) = ability_table {
            table_set_num(state, table_ref, (slot + 1) as f64, entry);
        }
    }

    ability_table.into_stack(state)
}

pub fn register_rilua_pet_journal(lua: &mut rilua::Lua) -> LuaResult<()> {
    let tb = TableBuilder::new(lua.state_mut());
    let tb = register_pet_fanfare_stubs(tb)?;
    let tb = register_pet_filter_stubs(tb)?;
    let tb = register_pet_count_stubs(tb)?;
    let tb = register_pet_info_stubs(tb)?;
    let tb = register_pet_ability_stubs(tb)?;
    let tb = register_pet_loadout_stubs(tb)?;
    let tb = register_pet_summon_stubs(tb)?;
    let tb = register_pet_journal_state_stubs(tb)?;
    let tb = register_pet_mutation_stubs(tb)?;
    let tb = register_pet_predicate_stubs(tb)?;
    let t = tb.build();

    set_global_val(lua.state_mut(), "C_PetJournal", t);
    Ok(())
}

fn register_pet_fanfare_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("ClearRecentFanfares", |_state| Ok(0))?
        .set_function("ClearFanfare", |_state| Ok(0))?
        .set_function("PetNeedsFanfare", |state| false.into_stack(state))?
        .set_function("GetNumPetsNeedingFanfare", |state| (0i32).into_stack(state))
}

fn register_pet_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    let tb = register_pet_default_filter_stubs(tb)?;
    let tb = register_pet_type_source_filter_stubs(tb)?;
    register_pet_misc_filter_stubs(tb)
}

fn register_pet_default_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsUsingDefaultFilters", |state| true.into_stack(state))?
        .set_function("SetDefaultFilters", |_state| Ok(0))
}

fn register_pet_type_source_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsPetTypeChecked", |state| false.into_stack(state))?
        .set_function("IsPetSourceChecked", |state| false.into_stack(state))?
        .set_function("SetPetTypeFilter", |_state| Ok(0))?
        .set_function("SetPetSourceChecked", |_state| Ok(0))?
        .set_function("SetAllPetTypesChecked", |_state| Ok(0))?
        .set_function("SetAllPetSourcesChecked", |_state| Ok(0))
}

fn register_pet_misc_filter_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("IsFilterChecked", |state| false.into_stack(state))?
        .set_function("SetFilterChecked", |_state| Ok(0))?
        .set_function("SetSearchFilter", |_state| Ok(0))
}

fn register_pet_count_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetNumPets", pet_get_num_pets)?
        .set_function("GetNumPetTypes", pet_get_num_pet_types)?
        .set_function("GetNumPetSources", pet_get_num_pet_sources)?
        .set_function("GetNumCollectedInfo", pet_get_num_collected_info)
}

fn register_pet_info_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    let tb = tb
        .set_function("GetBattlePetLink", |state| {
            state.push(Val::Nil);
            Ok(1)
        })?
        .set_function("GetPetInfoByIndex", pet_get_info_by_index)?
        .set_function("GetPetInfoByPetID", pet_get_info_by_pet_id)?
        .set_function("GetPetInfoBySpeciesID", pet_get_info_by_species_id)?;
    let tb = register_patch_12_1_pet_info_stubs(tb)?;
    tb.set_function(
        "GetPetModelSceneInfoBySpeciesID",
        pet_get_model_scene_info_by_species_id,
    )
}

fn register_pet_ability_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetPetAbilityInfo", pet_get_ability_info)?
        .set_function("GetPetAbilityList", |state| {
            create_table(state).into_stack(state)
        })?
        .set_function("GetPetAbilityListTable", pet_get_ability_list_table)?
        .set_function("SetAbility", |_state| Ok(0))
}

fn register_pet_loadout_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetPetCooldownByGUID", |state| {
        (0.0f64, 0.0f64, false).into_stack(state)
    })?
    .set_function("GetPetLoadOutInfo", pet_get_loadout_info)?
    .set_function("GetPetStats", pet_get_stats)?
    .set_function("GetPetSortParameter", |state| {
        state.push(Val::Num(0.0));
        Ok(1)
    })?
    .set_function("SetPetSortParameter", |_state| Ok(0))
}

fn pet_get_loadout_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    Ok(5)
}

fn register_pet_summon_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("GetPetSummonInfo", |state| {
        state.push(Val::Nil);
        Ok(1)
    })?
    .set_function("GetSummonedPetGUID", |_state| Ok(0))?
    .set_function("GetSummonBattlePetCooldown", |state| {
        (0.0f64, 0.0f64, false).into_stack(state)
    })?
    .set_function("SummonPetByGUID", |_state| Ok(0))?
    .set_function("SummonRandomPet", |_state| Ok(0))?
    .set_function("SpellTargetBattlePet", |_state| Ok(0))
}

fn register_pet_journal_state_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("ClearHoveredBattlePet", |_state| Ok(0))?
        .set_function("HasFavoritePets", |state| false.into_stack(state))?
        .set_function("IsFindBattleEnabled", |state| false.into_stack(state))?
        .set_function("IsJournalUnlocked", |state| true.into_stack(state))
}

fn register_pet_mutation_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("SetCustomName", |_state| Ok(0))?
        .set_function("SetFavorite", |_state| Ok(0))?
        .set_function("SetHoveredBattlePet", |_state| Ok(0))?
        .set_function("PickupPet", |_state| Ok(0))?
        .set_function("PickupSummonRandomPet", |_state| Ok(0))?
        .set_function("ReleasePetByID", |_state| Ok(0))?
        .set_function("CagePetByID", |_state| Ok(0))
}

fn register_pet_predicate_stubs(tb: TableBuilder) -> LuaResult<TableBuilder> {
    tb.set_function("PetIsFavorite", |state| false.into_stack(state))?
        .set_function("PetIsHurt", |state| false.into_stack(state))?
        .set_function("PetIsLockedForConvert", |state| false.into_stack(state))?
        .set_function("PetIsRevoked", |state| false.into_stack(state))?
        .set_function("PetIsSlotted", |state| false.into_stack(state))?
        .set_function("PetIsSummonable", |state| false.into_stack(state))?
        .set_function("PetIsTradable", |state| false.into_stack(state))
}
