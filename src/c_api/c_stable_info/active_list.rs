//! Supplied modern PetInfo observations, not native acquisition/default/sort policy.
//! Retail StableInfoDocumentation.lua:15–22,193–214; PTR 70493:19–26,215–241.
// C_StableInfo.GetNumActivePets: Retail StableInfoDocumentation.lua:33; Forever StableInfoDocumentation.lua:46.

use crate::c_api::helpers::set_table_array;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const NUMERIC_FIELDS: &[&str] = &[
    "slotID",
    "icon",
    "level",
    "displayID",
    "uiModelSceneID",
    "petNumber",
    "creatureID",
    "specID",
];
const TEXT_FIELDS: &[&str] = &["name", "familyName", "specialization", "type"];
const PTR_NUMERIC_FIELDS: &[&str] = &[
    "loyaltyLevel",
    "happinessLevel",
    "experience",
    "experienceNeeded",
];
const ABILITY_FIELDS: [&str; 2] = ["petAbilities", "specAbilities"];
const UNKNOWN_LIST: &str =
    "native-unknown: C_StableInfo.GetActivePetList has no supplied list or documented default";

/// Only declared, fully supplied fields. No Lua tables or caller-owned arrays are retained.
#[derive(Clone, Debug)]
pub struct PetInfo {
    numeric_fields: Vec<(&'static str, f64)>,
    text_fields: Vec<(&'static str, String)>,
    is_favorite: bool,
    is_exotic: bool,
    abilities: [Vec<f64>; 2],
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        let namespace = super::ensure_namespace(state, "C_StableInfo")?;
        table_set_rust_fn_static(state, namespace, "GetActivePetList", get_active_pet_list)?;
    }
    if ACTIVE == ClientProfile::Retail {
        let namespace = super::ensure_namespace(state, "C_StableInfo")?;
        table_set_rust_fn_static(state, namespace, "GetNumActivePets", get_num_active_pets)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(builder);
    }
    builder
        .set_function("SetActivePetList", supply_list)?
        .set_function("SetActivePet", supply_record)?
        .set_function("RemoveActivePet", remove_record)
}

fn get_active_pet_list(state: &mut LuaState) -> LuaResult<u32> {
    let pets = borrow_state(state)?
        .stable_active_pets
        .clone()
        .ok_or_else(|| runtime_error(UNKNOWN_LIST))?;
    let list = create_table(state);
    for (index, pet) in pets.iter().enumerate() {
        let record = project_record(state, pet);
        set_table_array(state, list, (index + 1) as i64, record);
    }
    state.push(list);
    Ok(1)
}

fn supply_list(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let observation = if matches!(value, Val::Nil) {
        None
    } else {
        let records = read_array(state, value)?;
        Some(
            records
                .into_iter()
                .map(|row| decode_record(state, row))
                .collect::<LuaResult<_>>()?,
        )
    };
    borrow_state_mut(state)?.stable_active_pets = observation;
    Ok(0)
}

fn supply_record(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_ordinal(state)?;
    let row = stack_val(state, 2);
    let record = decode_record(state, row)?;
    let mut sim = borrow_state_mut(state)?;
    let pets = sim
        .stable_active_pets
        .as_mut()
        .ok_or_else(|| runtime_error(UNKNOWN_LIST))?;
    if index > pets.len() {
        return Err(runtime_error(
            "SetActivePet: ordinal must replace or append without a gap",
        ));
    }
    if index == pets.len() {
        pets.push(record);
    } else {
        pets[index] = record;
    }
    Ok(0)
}

fn remove_record(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_ordinal(state)?;
    let mut sim = borrow_state_mut(state)?;
    let pets = sim
        .stable_active_pets
        .as_mut()
        .ok_or_else(|| runtime_error(UNKNOWN_LIST))?;
    if index >= pets.len() {
        return Err(runtime_error(
            "RemoveActivePet: supplied ordinal does not exist",
        ));
    }
    pets.remove(index);
    Ok(0)
}

fn read_ordinal(state: &LuaState) -> LuaResult<usize> {
    match stack_val(state, 1) {
        Val::Num(value) if value >= 1.0 && value <= f64::from(u32::MAX) && value.fract() == 0.0 => {
            Ok(value as usize - 1)
        }
        _ => Err(runtime_error(
            "active-pet observation ordinal must be a positive integer",
        )),
    }
}

pub(super) fn decode_record(state: &mut LuaState, row: Val) -> LuaResult<PetInfo> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(
            "active-pet observation must be a PetInfo table",
        ));
    }
    Ok(PetInfo {
        numeric_fields: read_numeric_fields(state, row)?,
        text_fields: read_text_fields(state, row)?,
        is_favorite: read_boolean(table_get(state, row, "isFavorite"), "isFavorite")?,
        is_exotic: read_boolean(table_get(state, row, "isExotic"), "isExotic")?,
        abilities: [
            read_abilities(state, row, ABILITY_FIELDS[0])?,
            read_abilities(state, row, ABILITY_FIELDS[1])?,
        ],
    })
}

fn read_numeric_fields(state: &mut LuaState, row: Val) -> LuaResult<Vec<(&'static str, f64)>> {
    let keys = NUMERIC_FIELDS.iter().chain(
        PTR_NUMERIC_FIELDS
            .iter()
            .filter(|_| ACTIVE == ClientProfile::Ptr),
    );
    keys.map(|&key| read_number(table_get(state, row, key), key).map(|value| (key, value)))
        .collect()
}

fn read_text_fields(state: &mut LuaState, row: Val) -> LuaResult<Vec<(&'static str, String)>> {
    let keys = TEXT_FIELDS
        .iter()
        .copied()
        .chain((ACTIVE == ClientProfile::Ptr).then_some("loyaltyName"));
    keys.map(|key| read_text(state, row, key).map(|text| (key, text)))
        .collect()
}

fn read_number(value: Val, key: &str) -> LuaResult<f64> {
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "PetInfo.{key} must be a finite number"
        ))),
    }
}

fn read_text(state: &mut LuaState, row: Val, key: &str) -> LuaResult<String> {
    let value = table_get(state, row, key);
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("PetInfo.{key} must be a UTF-8 string")))
}

fn read_boolean(value: Val, key: &str) -> LuaResult<bool> {
    match value {
        Val::Bool(flag) => Ok(flag),
        _ => Err(runtime_error(format!("PetInfo.{key} must be a boolean"))),
    }
}

fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error(
            "active-pet observation array must be a table",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("active-pet observation array is unavailable"))?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|i| table.get_int(i as i64))
        .collect())
}

fn read_abilities(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Vec<f64>> {
    let value = table_get(state, row, key);
    read_array(state, value)?
        .into_iter()
        .map(|value| read_number(value, key))
        .collect()
}

pub(super) fn project_record(state: &mut LuaState, pet: &PetInfo) -> Val {
    let row = create_table(state);
    for &(key, number) in &pet.numeric_fields {
        table_set(state, row, key, Val::Num(number));
    }
    for (key, text) in &pet.text_fields {
        let value = create_string(state, text);
        table_set(state, row, key, value);
    }
    table_set(state, row, "isFavorite", Val::Bool(pet.is_favorite));
    table_set(state, row, "isExotic", Val::Bool(pet.is_exotic));
    for (key, abilities) in ABILITY_FIELDS.into_iter().zip(&pet.abilities) {
        let array = create_table(state);
        for (index, &ability) in abilities.iter().enumerate() {
            set_table_array(state, array, (index + 1) as i64, Val::Num(ability));
        }
        table_set(state, row, key, array);
    }
    row
}

fn get_num_active_pets(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .stable_active_pets
        .as_ref()
        .map_or(0, Vec::len);
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(all(test, feature = "client-retail"))]
#[test]
fn active_pet_count_follows_supplied_list() {
    crate::lua_api::WowLuaEnv::new().unwrap().exec(r#"
        local function pet(slot)
            return {slotID=slot,icon=10,level=80,displayID=20,uiModelSceneID=30,petNumber=40+slot,
                creatureID=50,specID=60,name="Pet "..slot,familyName="Cat",specialization="Ferocity",
                type="Cat",isFavorite=false,isExotic=false,petAbilities={},specAbilities={}}
        end
        A_Admin.SetActivePetList({pet(1),pet(3)})
        assert(C_StableInfo.GetNumActivePets() == 2)
        A_Admin.RemoveActivePet(1)
        assert(C_StableInfo.GetNumActivePets() == 1)
        A_Admin.SetActivePetList({})
        assert(C_StableInfo.GetNumActivePets() == 0)
    "#).unwrap();
}
