//! Supplied specialization observations, not a native catalog or initial-state policy.
//! Retail StableInfoDocumentation.lua:24–31,216–224; PTR 70493:28–35,243–251.

use crate::c_api::helpers::set_table_array;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Complete declared fields copied from a simulator-supplied observation.
#[derive(Clone, Debug)]
pub struct PetSpecInfo {
    pub spec_id: f64,
    pub spec_index: u32,
    pub specialization_name: String,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        let namespace = super::ensure_namespace(state, "C_StableInfo")?;
        table_set_rust_fn_static(state, namespace, "GetAvailablePetSpecInfos", get_list)?;
    }
    Ok(())
}

fn get_list(state: &mut LuaState) -> LuaResult<u32> {
    let observation = borrow_state(state)?
        .stable_pet_spec_infos
        .clone()
        .unwrap_or_else(crate::lua_api::workarounds::default_pet_spec_catalog);
    let list = create_table(state);
    for (index, record) in observation.iter().enumerate() {
        let row = create_table(state);
        table_set(state, row, "specID", Val::Num(record.spec_id));
        table_set(
            state,
            row,
            "specIndex",
            Val::Num(f64::from(record.spec_index)),
        );
        let name = create_string(state, &record.specialization_name);
        table_set(state, row, "specializationName", name);
        set_table_array(state, list, (index + 1) as i64, row);
    }
    state.push(list);
    Ok(1)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        builder.set_function("SetAvailablePetSpecInfos", supply_list)
    } else {
        Ok(builder)
    }
}

fn supply_list(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let observation = if matches!(value, Val::Nil) {
        None
    } else {
        Some(
            read_array(state, value)?
                .into_iter()
                .map(|record| decode_record(state, record))
                .collect::<LuaResult<Vec<_>>>()?,
        )
    };
    borrow_state_mut(state)?.stable_pet_spec_infos = observation;
    Ok(0)
}

fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error(
            "pet specialization observation must be an array",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("pet specialization observation array is unavailable"))?;
    let length = table.len(&state.gc.string_arena);
    let array_is_bounded = table
        .array_slice()
        .iter()
        .enumerate()
        .all(|(index, value)| matches!(value, Val::Nil) || index < length);
    let hash_is_bounded = table.hash_entries().iter().all(|(key, value)| {
        if matches!(value, Val::Nil) {
            return true;
        }
        matches!(key, Val::Num(index) if *index >= 1.0 && *index <= length as f64 && index.fract() == 0.0)
    });
    if !array_is_bounded || !hash_is_bounded {
        return Err(runtime_error(
            "pet specialization observation must be a dense array",
        ));
    }
    Ok((1..=length)
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn decode_record(state: &mut LuaState, row: Val) -> LuaResult<PetSpecInfo> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(
            "pet specialization observation must be a PetSpecInfo record",
        ));
    }
    let spec_id = read_number(table_get(state, row, "specID"))?;
    let spec_index = read_index(table_get(state, row, "specIndex"))?;
    let name = table_get(state, row, "specializationName");
    let specialization_name = match name {
        Val::Str(_) => val_to_string(state, name),
        _ => None,
    }
    .ok_or_else(|| runtime_error("pet specialization specializationName must be a UTF-8 string"))?;
    Ok(PetSpecInfo {
        spec_id,
        spec_index,
        specialization_name,
    })
}

fn read_number(value: Val) -> LuaResult<f64> {
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(
            "pet specialization specID must be a finite number",
        )),
    }
}

fn read_index(value: Val) -> LuaResult<u32> {
    match value {
        Val::Num(index) if index >= 1.0 && index <= f64::from(u32::MAX) && index.fract() == 0.0 => {
            Ok(index as u32)
        }
        _ => Err(runtime_error(
            "pet specialization specIndex must be a positive u32 integer",
        )),
    }
}
