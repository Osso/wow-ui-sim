//! INFERRED vignette snapshots supplied by the simulator host (R88g6).

use std::collections::BTreeMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::client_profile::ACTIVE;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, table_get,
    table_set, val_to_string,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default)]
pub struct VignetteState {
    records: BTreeMap<String, VignetteRecord>,
}

#[derive(Clone, Debug)]
struct VignetteRecord {
    fields: BTreeMap<String, Val>,
    map_id: i32,
    x: f64,
    y: f64,
    facing: Option<f64>,
    health: Option<f64>,
    group_size: Option<(f64, f64)>,
}

const FIELDS: &[&str] = &[
    "objectGUID",
    "name",
    "isDead",
    "onWorldMap",
    "zoneInfiniteAOI",
    "onMinimap",
    "isUnique",
    "inFogOfWar",
    "atlasName",
    "hasTooltip",
    "vignetteID",
    "type",
    "rewardQuestID",
    "tooltipWidgetSet",
    "iconWidgetSet",
    "addPaddingAboveTooltipWidgets",
    "mapPin",
    "objectiveType",
];

fn enabled() -> bool {
    ACTIVE.uses_vignette_info_model()
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !enabled() {
        return Ok(());
    }
    let namespace = ensure_namespace(state, "C_VignetteInfo")?;
    for (name, callback) in [
        ("GetVignettes", get_vignettes as _),
        ("GetVignetteInfo", get_vignette_info as _),
        ("GetVignettePosition", get_vignette_position as _),
        ("FindBestUniqueVignette", find_best_unique as _),
        ("GetHealthPercent", get_health as _),
        ("GetRecommendedGroupSize", get_group_size as _),
    ] {
        table_set_rust_fn_static(state, namespace, name, callback)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if enabled() {
        builder.set_function("SetVignettes", set_vignettes)
    } else {
        Ok(builder)
    }
}

fn set_vignettes(state: &mut LuaState) -> LuaResult<u32> {
    let records = read_records(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.vignettes.records = records;
    fire_named_event_state(state, "VIGNETTES_UPDATED", &[]);
    Ok(0)
}

fn read_records(state: &mut LuaState, value: Val) -> LuaResult<BTreeMap<String, VignetteRecord>> {
    let Val::Table(array) = value else {
        return Err(runtime_error("A_Admin.SetVignettes: expected array"));
    };
    let len = state
        .gc
        .tables
        .get(array)
        .map(|table| table.len(&state.gc.string_arena))
        .unwrap_or(0);
    (1..=len)
        .map(|index| read_record(state, array, index))
        .collect()
}

fn read_record(
    state: &mut LuaState,
    array: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
    index: usize,
) -> LuaResult<(String, VignetteRecord)> {
    let value = state
        .gc
        .tables
        .get(array)
        .map(|table| table.get_int(index as i64))
        .unwrap_or(Val::Nil);
    let table = record_table(value)?;
    let guid = string_field(state, table, "vignetteGUID")?;
    Ok((guid, read_record_data(state, table)?))
}

fn record_table(value: Val) -> LuaResult<Val> {
    match value {
        Val::Table(table) => Ok(Val::Table(table)),
        _ => Err(runtime_error("A_Admin.SetVignettes: expected record table")),
    }
}

fn read_record_data(state: &mut LuaState, table: Val) -> LuaResult<VignetteRecord> {
    let map_id = number_field(state, table, "uiMapID")? as i32;
    let x = number_field(state, table, "x")?;
    let y = number_field(state, table, "y")?;
    let facing = optional_number(state, table, "facing");
    let health = optional_number(state, table, "healthPercent");
    let group_size = optional_pair(state, table, "minGroupSize", "maxGroupSize");
    Ok(VignetteRecord {
        fields: read_fields(state, table),
        map_id,
        x,
        y,
        facing,
        health,
        group_size,
    })
}

fn read_fields(state: &mut LuaState, table: Val) -> BTreeMap<String, Val> {
    FIELDS
        .iter()
        .filter_map(|name| {
            let value = table_get(state, table, name);
            (!matches!(value, Val::Nil)).then(|| ((*name).to_owned(), value))
        })
        .collect()
}

fn string_field(state: &mut LuaState, table: Val, name: &str) -> LuaResult<String> {
    let value = table_get(state, table, name);
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("A_Admin.SetVignettes: {name} must be a string")))
}

fn number_field(state: &mut LuaState, table: Val, name: &str) -> LuaResult<f64> {
    match table_get(state, table, name) {
        Val::Num(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "A_Admin.SetVignettes: {name} must be a number"
        ))),
    }
}

fn optional_number(state: &mut LuaState, table: Val, name: &str) -> Option<f64> {
    match table_get(state, table, name) {
        Val::Num(value) => Some(value),
        _ => None,
    }
}

fn optional_pair(state: &mut LuaState, table: Val, min: &str, max: &str) -> Option<(f64, f64)> {
    Some((
        optional_number(state, table, min)?,
        optional_number(state, table, max)?,
    ))
}

fn records(state: &LuaState) -> Vec<(String, VignetteRecord)> {
    borrow_state(state)
        .map(|sim| {
            sim.vignettes
                .records
                .iter()
                .map(|(guid, record)| (guid.clone(), record.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn get_vignettes(state: &mut LuaState) -> LuaResult<u32> {
    let rows = records(state);
    let table = create_table(state);
    for (index, (guid, _)) in rows.into_iter().enumerate() {
        let value = create_string(state, &guid);
        set_table_array(state, table, index as i64 + 1, value);
    }
    state.push(table);
    Ok(1)
}

fn get_vignette_info(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let record = borrow_state(state)?.vignettes.records.get(&guid).cloned();
    let Some(record) = record else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    let guid_value = create_string(state, &guid);
    table_set(state, table, "vignetteGUID", guid_value);
    for (name, value) in record.fields {
        table_set(state, table, &name, value);
    }
    state.push(table);
    Ok(1)
}

fn get_vignette_position(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let map_id = i32::from_stack(state, 2)?;
    let record = borrow_state(state)?.vignettes.records.get(&guid).cloned();
    let Some(record) = record.filter(|record| record.map_id == map_id) else {
        return Ok(0);
    };
    let factory = crate::c_api::global_val(state, "CreateVector2D");
    let vector = call_function_state(state, factory, &[Val::Num(record.x), Val::Num(record.y)])?;
    state.push(vector);
    state.push(record.facing.map_or(Val::Nil, Val::Num));
    Ok(2)
}

fn find_best_unique(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(table) = stack_val(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let ids = state
        .gc
        .tables
        .get(table)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    for (index, id) in ids.iter().enumerate() {
        let Some(guid) = val_to_string(state, *id) else {
            continue;
        };
        if borrow_state(state)?.vignettes.records.contains_key(&guid) {
            state.push(Val::Num((index + 1) as f64));
            return Ok(1);
        }
    }
    state.push(Val::Nil);
    Ok(1)
}

fn get_health(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let health = borrow_state(state)?
        .vignettes
        .records
        .get(&guid)
        .and_then(|record| record.health);
    state.push(health.map_or(Val::Nil, Val::Num));
    Ok(1)
}

fn get_group_size(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let pair = borrow_state(state)?
        .vignettes
        .records
        .get(&guid)
        .and_then(|record| record.group_size);
    if let Some((min, max)) = pair {
        state.push(Val::Num(min));
        state.push(Val::Num(max));
        Ok(2)
    } else {
        Ok(0)
    }
}
