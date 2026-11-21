//! INFERRED (R88cx): contribution collector queries expose host-supplied map records.

use std::collections::HashMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table_with_capacity, table_get,
    table_set_static, val_to_string,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str =
    "A_Admin.SetContributionCollectors: expected an array of collector tables";

#[derive(Clone, Debug, Default)]
pub struct ContributionCollectorState {
    collectors: Vec<Collector>,
    contributions: HashMap<u32, Contribution>,
}

#[derive(Clone, Debug)]
struct Collector {
    map_id: i32,
    creature_id: u32,
    name: String,
    atlas_name: String,
    texture_kit: String,
    x: f64,
    y: f64,
    contribution_ids: Vec<u32>,
}

#[derive(Clone, Debug)]
struct Contribution {
    name: String,
    state: i32,
    amount: f64,
    next_state_time: Option<f64>,
    appearance: Option<Appearance>,
}

#[derive(Clone, Debug)]
struct Appearance {
    state_name: String,
    color: [f64; 4],
    tooltip_line: String,
    use_time_remaining: bool,
    status_bar_atlas: String,
    border_atlas: String,
    banner_atlas: String,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ContributionCollector")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetContributionCollectorsForMap",
        collectors_for_map,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetManagedContributionsForCreatureID",
        managed_contributions,
    )?;
    table_set_rust_fn_static(state, namespace, "GetName", contribution_name)?;
    table_set_rust_fn_static(state, namespace, "GetState", contribution_state)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetContributionAppearance",
        contribution_appearance,
    )?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetContributionCollectors", set_collectors)
}

fn set_collectors(state: &mut LuaState) -> LuaResult<u32> {
    let records = read_collectors(state, stack_val(state, 1))?;
    let mut contributions = HashMap::new();
    for collector in &records {
        for contribution in &collector.contributions {
            contributions.insert(contribution.0, contribution.1.clone());
        }
    }
    let collectors = records.into_iter().map(|record| record.collector).collect();
    borrow_state_mut(state)?.contribution_collectors = ContributionCollectorState {
        collectors,
        contributions,
    };
    Ok(0)
}

struct CollectorInput {
    collector: Collector,
    contributions: Vec<(u32, Contribution)>,
}

fn read_collectors(state: &mut LuaState, value: Val) -> LuaResult<Vec<CollectorInput>> {
    let Val::Table(array) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(INPUT_ERROR))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let entry = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_collector(state, entry)
        })
        .collect()
}

fn read_collector(state: &mut LuaState, value: Val) -> LuaResult<CollectorInput> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let contribution_values = field(state, table, "contributions");
    let contributions = read_contributions(state, contribution_values)?;
    let collector = Collector {
        map_id: read_integer(state, table, "mapID")? as i32,
        creature_id: read_integer(state, table, "collectorCreatureID")? as u32,
        name: read_string(state, table, "name")?,
        atlas_name: read_string(state, table, "atlasName")?,
        texture_kit: read_string(state, table, "textureKit")?,
        x: read_number(state, table, "x")?,
        y: read_number(state, table, "y")?,
        contribution_ids: contributions.iter().map(|entry| entry.0).collect(),
    };
    Ok(CollectorInput {
        collector,
        contributions,
    })
}

fn read_contributions(state: &mut LuaState, value: Val) -> LuaResult<Vec<(u32, Contribution)>> {
    let Val::Table(array) = value else {
        return match value {
            Val::Nil => Ok(Vec::new()),
            _ => Err(invalid_field("contributions")),
        };
    };
    let length = state
        .gc
        .tables
        .get(array)
        .map(|table| table.len(&state.gc.string_arena))
        .unwrap_or(0);
    (1..=length)
        .map(|index| {
            let value = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_contribution(state, value)
        })
        .collect()
}

fn read_contribution(state: &mut LuaState, value: Val) -> LuaResult<(u32, Contribution)> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let id = read_integer(state, table, "contributionID")? as u32;
    let appearance_value = field(state, table, "appearance");
    let appearance = read_appearance(state, appearance_value)?;
    let contribution = Contribution {
        name: read_string(state, table, "name")?,
        state: read_integer(state, table, "state")? as i32,
        amount: read_number(state, table, "amount")?,
        next_state_time: match field(state, table, "nextStateTime") {
            Val::Num(value) if value.is_finite() => Some(value),
            Val::Nil => None,
            _ => return Err(invalid_field("nextStateTime")),
        },
        appearance,
    };
    Ok((id, contribution))
}

fn read_appearance(state: &mut LuaState, value: Val) -> LuaResult<Option<Appearance>> {
    let Val::Table(table) = value else {
        return match value {
            Val::Nil => Ok(None),
            _ => Err(invalid_field("appearance")),
        };
    };
    let color_value = field(state, table, "stateColor");
    let Val::Table(color) = color_value else {
        return Err(invalid_field("stateColor"));
    };
    let red = read_number(state, color, "r")?;
    let green = read_number(state, color, "g")?;
    let blue = read_number(state, color, "b")?;
    Ok(Some(Appearance {
        state_name: read_string(state, table, "stateName")?,
        color: [red, green, blue, 1.0],
        tooltip_line: read_string(state, table, "tooltipLine")?,
        use_time_remaining: read_bool(state, table, "tooltipUseTimeRemaining")?,
        status_bar_atlas: read_string(state, table, "statusBarAtlas")?,
        border_atlas: read_string(state, table, "borderAtlas")?,
        banner_atlas: read_string(state, table, "bannerAtlas")?,
    }))
}

fn read_bool(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<bool> {
    match field(state, table, key) {
        Val::Bool(value) => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn read_integer(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<i64> {
    match field(state, table, key) {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && value >= 0.0
                && value <= i32::MAX as f64 =>
        {
            Ok(value as i64)
        }
        _ => Err(invalid_field(key)),
    }
}

fn read_number(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<f64> {
    match field(state, table, key) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn read_string(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<String> {
    let value = table_get(state, Val::Table(table), key);
    val_to_string(state, value).ok_or_else(|| invalid_field(key))
}

fn field(state: &mut LuaState, table: GcRef<Table>, key: &str) -> Val {
    table_get(state, Val::Table(table), key)
}

fn invalid_field(key: &str) -> rilua::LuaError {
    runtime_error(format!("A_Admin.SetContributionCollectors: invalid {key}"))
}

fn collectors_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let records = borrow_state(state)?
        .contribution_collectors
        .collectors
        .iter()
        .filter(|record| record.map_id == map_id)
        .cloned()
        .collect::<Vec<_>>();
    let result = create_table_with_capacity(state, records.len());
    for (index, record) in records.iter().enumerate() {
        let position = super::c_map::create_world_position_vector(state, record.x, record.y)?;
        let value = create_table_with_capacity(state, 4);
        table_set_static(
            state,
            value,
            "collectorCreatureID",
            Val::Num(f64::from(record.creature_id)),
        );
        let name = create_string(state, &record.name);
        table_set_static(state, value, "name", name);
        let atlas_name = create_string(state, &record.atlas_name);
        table_set_static(state, value, "atlasName", atlas_name);
        let texture_kit = create_string(state, &record.texture_kit);
        table_set_static(state, value, "textureKit", texture_kit);
        table_set_static(state, value, "position", position);
        set_table_array(state, result, (index + 1) as i64, value);
    }
    state.push(result);
    Ok(1)
}

fn managed_contributions(state: &mut LuaState) -> LuaResult<u32> {
    let creature_id = u32::from_stack(state, 1)?;
    let ids = borrow_state(state)?
        .contribution_collectors
        .collectors
        .iter()
        .find(|record| record.creature_id == creature_id)
        .map(|record| record.contribution_ids.clone())
        .unwrap_or_default();
    for id in &ids {
        state.push(Val::Num(f64::from(*id)));
    }
    Ok(ids.len() as u32)
}

fn contribution_name(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let name = borrow_state(state)?
        .contribution_collectors
        .contributions
        .get(&id)
        .map(|value| value.name.clone());
    let name = name
        .map(|name| create_string(state, &name))
        .unwrap_or(Val::Nil);
    state.push(name);
    Ok(1)
}

fn contribution_state(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let value = borrow_state(state)?
        .contribution_collectors
        .contributions
        .get(&id)
        .cloned();
    if let Some(value) = value {
        state.push(Val::Num(f64::from(value.state)));
        state.push(Val::Num(value.amount));
        state.push(value.next_state_time.map(Val::Num).unwrap_or(Val::Nil));
        Ok(3)
    } else {
        state.push(Val::Nil);
        Ok(1)
    }
}

fn contribution_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let appearance = borrow_state(state)?
        .contribution_collectors
        .contributions
        .get(&id)
        .and_then(|value| value.appearance.clone());
    let result = appearance
        .map(|appearance| build_appearance_table(state, &appearance))
        .unwrap_or(Val::Nil);
    state.push(result);
    Ok(1)
}

fn build_appearance_table(state: &mut LuaState, appearance: &Appearance) -> Val {
    let table = create_table_with_capacity(state, 7);
    set_appearance_name(state, table, appearance);
    set_appearance_color(state, table, appearance);
    set_appearance_details(state, table, appearance);
    table
}

fn set_appearance_name(state: &mut LuaState, table: Val, appearance: &Appearance) {
    let state_name = create_string(state, &appearance.state_name);
    table_set_static(state, table, "stateName", state_name);
}

fn set_appearance_color(state: &mut LuaState, table: Val, appearance: &Appearance) {
    let color = super::c_allied_races::create_color_mixin(
        state,
        appearance.color[0],
        appearance.color[1],
        appearance.color[2],
    );
    table_set_static(state, table, "stateColor", color);
}

fn set_appearance_details(state: &mut LuaState, table: Val, appearance: &Appearance) {
    let tooltip_line = create_string(state, &appearance.tooltip_line);
    table_set_static(state, table, "tooltipLine", tooltip_line);
    table_set_static(
        state,
        table,
        "tooltipUseTimeRemaining",
        Val::Bool(appearance.use_time_remaining),
    );
    set_appearance_atlas(state, table, "statusBarAtlas", &appearance.status_bar_atlas);
    set_appearance_atlas(state, table, "borderAtlas", &appearance.border_atlas);
    set_appearance_atlas(state, table, "bannerAtlas", &appearance.banner_atlas);
}

fn set_appearance_atlas(state: &mut LuaState, table: Val, key: &'static str, name: &str) {
    let name = create_string(state, name);
    table_set_static(state, table, key, name);
}
