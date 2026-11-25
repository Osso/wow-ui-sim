//! Supplied House Finder scenario rows and asynchronous responses.
//!
//! API declarations are in Retail HousingUIDocumentation.lua:63, 168, 208,
//! 256, 428, and 511. StartTutorial's effect is INFERRED (R88g1; native evidence pending).

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, table_get,
    table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default)]
pub struct HouseFinderState {
    pub neighborhoods: Vec<HouseFinderNeighborhood>,
}

#[derive(Clone, Debug)]
pub struct HouseFinderNeighborhood {
    pub guid: String,
    pub name: String,
    pub neighborhood_type: i64,
    pub owner_type: i64,
    pub suggestion_reason: i64,
    pub ui_map_id: i64,
    pub texture_suffix: String,
    pub faction_matches: bool,
    pub plots: Vec<HouseFinderPlot>,
}

#[derive(Clone, Debug)]
pub struct HouseFinderPlot {
    pub x: f64,
    pub y: f64,
    pub owner_type: i64,
    pub plot_id: Option<i64>,
    pub plot_cost: Option<f64>,
    pub owner_name: Option<String>,
}

#[derive(Clone, Debug)]
pub enum HouseFinderReply {
    Neighborhoods {
        success: Val,
        rows: Vec<HouseFinderNeighborhood>,
    },
    Plots(Vec<HouseFinderPlot>),
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let housing = ensure_namespace(state, "C_Housing")?;
    table_set_rust_fn_static(
        state,
        housing,
        "HouseFinderRequestNeighborhoods",
        request_neighborhoods,
    )?;
    table_set_rust_fn_static(state, housing, "GetUIMapIDForNeighborhood", get_ui_map_id)?;
    table_set_rust_fn_static(
        state,
        housing,
        "GetNeighborhoodTextureSuffix",
        get_texture_suffix,
    )?;
    table_set_rust_fn_static(
        state,
        housing,
        "DoesFactionMatchNeighborhood",
        faction_matches,
    )?;
    table_set_rust_fn_static(
        state,
        housing,
        "RequestHouseFinderNeighborhoodData",
        request_plots,
    )?;
    table_set_rust_fn_static(state, housing, "StartTutorial", start_tutorial)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetHouseFinderNeighborhoods", set_neighborhoods)
}

fn request_neighborhoods(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?
        .housing
        .house_finder
        .neighborhoods
        .clone();
    let success = housing_success_value(state)?;
    borrow_state_mut(state)?
        .pending_house_finder_replies
        .push_back(HouseFinderReply::Neighborhoods { success, rows });
    Ok(0)
}

fn get_ui_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let guid: String = argument(state, 1)?;
    let row = find_neighborhood(state, &guid)?;
    state.push(Val::Num(row.ui_map_id as f64));
    Ok(1)
}

fn get_texture_suffix(state: &mut LuaState) -> LuaResult<u32> {
    let guid: String = argument(state, 1)?;
    let row = find_neighborhood(state, &guid)?;
    let suffix = create_string(state, &row.texture_suffix);
    state.push(suffix);
    Ok(1)
}

fn faction_matches(state: &mut LuaState) -> LuaResult<u32> {
    let guid: String = argument(state, 1)?;
    let matches = find_neighborhood(state, &guid)?.faction_matches;
    state.push(Val::Bool(matches));
    Ok(1)
}

fn request_plots(state: &mut LuaState) -> LuaResult<u32> {
    let guid: String = argument(state, 1)?;
    let _name: String = argument(state, 2)?;
    let rows = borrow_state(state)?
        .housing
        .house_finder
        .neighborhoods
        .clone();
    let plots = rows
        .into_iter()
        .find(|row| row.guid == guid)
        .ok_or_else(|| {
            runtime_error(
                "C_Housing.RequestHouseFinderNeighborhoodData: neighborhood was not supplied",
            )
        })?
        .plots;
    borrow_state_mut(state)?
        .pending_house_finder_replies
        .push_back(HouseFinderReply::Plots(plots));
    Ok(0)
}

fn start_tutorial(state: &mut LuaState) -> LuaResult<u32> {
    let autocomplete = global_value(state, "AUTOCOMPLETE_LIST");
    if matches!(table_get(state, autocomplete, "HOUSE_FINDER"), Val::Nil) {
        load_addon(state, "Blizzard_AutoComplete")?;
    }
    if matches!(global_value(state, "HouseFinderFrame"), Val::Nil) {
        load_addon(state, "Blizzard_HousingHouseFinder")?;
    }
    show_house_finder(state)?;
    hide_housing_dashboard(state)?;
    Ok(0)
}

fn load_addon(state: &mut LuaState, name: &str) -> LuaResult<()> {
    let addons = global_value(state, "C_AddOns");
    let loader = table_get(state, addons, "LoadAddOn");
    let name = create_string(state, name);
    call_function_state(state, loader, &[name])?;
    Ok(())
}

fn show_house_finder(state: &mut LuaState) -> LuaResult<()> {
    let frame = global_value(state, "HouseFinderFrame");
    let show = global_value(state, "ShowUIPanel");
    call_function_state(state, show, &[frame])?;
    if !matches!(
        table_get(state, frame, "hasNeighborhoodList"),
        Val::Bool(true)
    ) {
        let housing = global_value(state, "C_Housing");
        let request = table_get(state, housing, "HouseFinderRequestNeighborhoods");
        call_function_state(state, request, &[])?;
    }
    Ok(())
}

fn hide_housing_dashboard(state: &mut LuaState) -> LuaResult<()> {
    let frame = global_value(state, "HousingDashboardFrame");
    if !matches!(frame, Val::Nil) {
        let hide = global_value(state, "HideUIPanel");
        call_function_state(state, hide, &[frame])?;
    }
    Ok(())
}

fn global_value(state: &mut LuaState, name: &str) -> Val {
    let key = state.gc.intern_string(name.as_bytes());
    state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key, &state.gc.string_arena))
        .unwrap_or(Val::Nil)
}

fn set_neighborhoods(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let rows = decode_neighborhoods(state, value)?;
    borrow_state_mut(state)?.housing.house_finder.neighborhoods = rows;
    Ok(0)
}

fn decode_neighborhoods(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<HouseFinderNeighborhood>> {
    let entries = array_entries(state, value, "A_Admin.SetHouseFinderNeighborhoods")?;
    entries
        .into_iter()
        .map(|row| decode_neighborhood(state, row))
        .collect()
}

fn decode_neighborhood(state: &mut LuaState, row: Val) -> LuaResult<HouseFinderNeighborhood> {
    require_table(
        row,
        "A_Admin.SetHouseFinderNeighborhoods: each row must be a table",
    )?;
    let plots = field(state, row, "plots")?;
    Ok(HouseFinderNeighborhood {
        guid: field(state, row, "neighborhoodGUID")?,
        name: field(state, row, "neighborhoodName")?,
        neighborhood_type: field(state, row, "neighborhoodType")?,
        owner_type: field(state, row, "neighborhoodOwnerType")?,
        suggestion_reason: field(state, row, "suggestionReason")?,
        ui_map_id: field(state, row, "uiMapID")?,
        texture_suffix: field(state, row, "textureSuffix")?,
        faction_matches: field(state, row, "factionMatches")?,
        plots: decode_plots(state, plots)?,
    })
}

fn decode_plots(state: &mut LuaState, value: Val) -> LuaResult<Vec<HouseFinderPlot>> {
    let entries = array_entries(state, value, "A_Admin.SetHouseFinderNeighborhoods plots")?;
    entries
        .into_iter()
        .map(|row| decode_plot(state, row))
        .collect()
}

fn decode_plot(state: &mut LuaState, row: Val) -> LuaResult<HouseFinderPlot> {
    require_table(
        row,
        "A_Admin.SetHouseFinderNeighborhoods: each plot must be a table",
    )?;
    let position = field::<Val>(state, row, "mapPosition")?;
    Ok(HouseFinderPlot {
        x: field(state, position, "x")?,
        y: field(state, position, "y")?,
        owner_type: field(state, row, "ownerType")?,
        plot_id: field(state, row, "plotID")?,
        plot_cost: field(state, row, "plotCost")?,
        owner_name: field(state, row, "ownerName")?,
    })
}

fn array_entries(state: &mut LuaState, value: Val, context: &str) -> LuaResult<Vec<Val>> {
    let Val::Table(table) = value else {
        return Err(runtime_error(format!("{context}: expected an array")));
    };
    let supplied = state
        .gc
        .tables
        .get(table)
        .ok_or_else(|| runtime_error(format!("{context}: invalid array")))?;
    Ok((1..=supplied.len(&state.gc.string_arena))
        .map(|index| supplied.get_int(index as i64))
        .collect())
}

fn require_table(value: Val, message: &str) -> LuaResult<()> {
    if matches!(value, Val::Table(_)) {
        Ok(())
    } else {
        Err(runtime_error(message))
    }
}

fn find_neighborhood<'a>(
    state: &'a mut LuaState,
    guid: &str,
) -> LuaResult<HouseFinderNeighborhood> {
    borrow_state(state)?
        .housing
        .house_finder
        .neighborhoods
        .iter()
        .find(|row| row.guid == guid)
        .cloned()
        .ok_or_else(|| runtime_error("C_Housing: neighborhood was not supplied"))
}

fn housing_success_value(state: &mut LuaState) -> LuaResult<Val> {
    let enum_key = state.gc.intern_string(b"Enum");
    let enums = state
        .gc
        .tables
        .get(state.global)
        .map(|table| table.get_str(enum_key, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let result = table_get(state, enums, "HousingResult");
    let success = table_get(state, result, "Success");
    if matches!(success, Val::Nil) {
        return Err(runtime_error("Enum.HousingResult.Success is unavailable"));
    }
    Ok(success)
}

fn argument<T: FromStack>(state: &mut LuaState, index: i32) -> LuaResult<T> {
    let top = state.top;
    let result = T::from_stack(state, index);
    state.top = top;
    result
}

fn field<T: FromStack>(state: &mut LuaState, row: Val, name: &str) -> LuaResult<T> {
    let value = table_get(state, row, name);
    let top = state.top;
    state.push(value);
    let result = T::from_stack(state, -1);
    state.top = top;
    result
}

pub(crate) fn encode_neighborhoods(
    state: &mut LuaState,
    rows: Vec<HouseFinderNeighborhood>,
) -> Val {
    let Val::Table(table) = create_table(state) else {
        unreachable!()
    };
    for (index, row) in rows.into_iter().enumerate() {
        let value = encode_neighborhood(state, row);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    Val::Table(table)
}

fn encode_neighborhood(state: &mut LuaState, row: HouseFinderNeighborhood) -> Val {
    let value = create_table(state);
    for (key, text) in [
        ("neighborhoodGUID", row.guid),
        ("neighborhoodName", row.name),
    ] {
        let text = create_string(state, &text);
        table_set(state, value, key, text);
    }
    table_set(
        state,
        value,
        "neighborhoodType",
        Val::Num(row.neighborhood_type as f64),
    );
    table_set(
        state,
        value,
        "neighborhoodOwnerType",
        Val::Num(row.owner_type as f64),
    );
    table_set(
        state,
        value,
        "suggestionReason",
        Val::Num(row.suggestion_reason as f64),
    );
    value
}

pub(crate) fn encode_plots(state: &mut LuaState, plots: Vec<HouseFinderPlot>) -> Val {
    let Val::Table(table) = create_table(state) else {
        unreachable!()
    };
    for (index, plot) in plots.into_iter().enumerate() {
        let value = encode_plot(state, plot);
        table_set_num(state, table, (index + 1) as f64, value);
    }
    Val::Table(table)
}

fn encode_plot(state: &mut LuaState, plot: HouseFinderPlot) -> Val {
    let value = create_table(state);
    let position = create_table(state);
    table_set(state, position, "x", Val::Num(plot.x));
    table_set(state, position, "y", Val::Num(plot.y));
    table_set(state, value, "mapPosition", position);
    table_set(state, value, "ownerType", Val::Num(plot.owner_type as f64));
    set_optional(
        state,
        value,
        "plotID",
        plot.plot_id.map(|v| Val::Num(v as f64)),
    );
    set_optional(state, value, "plotCost", plot.plot_cost.map(Val::Num));
    if let Some(name) = plot.owner_name {
        let owner = create_string(state, &name);
        table_set(state, value, "ownerName", owner);
    }
    value
}

fn set_optional(state: &mut LuaState, table: Val, key: &str, value: Option<Val>) {
    if let Some(value) = value {
        table_set(state, table, key, value);
    }
}
