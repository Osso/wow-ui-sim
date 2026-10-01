//! `C_AreaPoiInfo` probe surface backed by `SimState.area_pois`.
//!
//! Migrates 2 entries off `NAMESPACE_NIL_STUBS`:
//!
//! - `C_AreaPoiInfo.GetAreaPOIInfo(uiMapID, areaPoiID)` — returns the
//!   `AreaPOIInfo` structure for the seeded POI, or nothing (retail
//!   `mayreturnnothing`). If `uiMapID` is non-nil it must match the
//!   POI's `ui_map_id`.
//! - `C_AreaPoiInfo.GetAreaPOISecondsLeft(areaPoiID)` — returns the
//!   remaining seconds for a time-limited POI, or nothing for
//!   permanent POIs / unknown ids.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_get, table_set, table_set_num,
    table_set_static,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Minimal area-POI metadata keyed by area poi id in
/// `SimState.area_pois`. Drives `C_AreaPoiInfo.GetAreaPOIInfo` and
/// `GetAreaPOISecondsLeft`. Only the subset of retail fields used by
/// Blizzard UI is carried; everything else is returned as nil /
/// default.
#[derive(Debug, Clone)]
pub struct AreaPoiInfo {
    pub area_poi_id: i32,
    pub name: String,
    /// UI-map id the POI is bound to. `None` when the POI is only
    /// looked up by id (the `uiMapID` arg to `GetAreaPOIInfo` is
    /// nilable too, so both sides can be unbound).
    pub ui_map_id: Option<i32>,
    /// Normalized 0..=1 screen position on the map.
    pub position: (f64, f64),
    pub atlas_name: Option<String>,
    pub description: Option<String>,
    pub faction_id: Option<i32>,
    pub icon_widget_set: Option<i32>,
    pub linked_ui_map_id: Option<i32>,
    pub is_current_event: bool,
    pub should_glow: bool,
    /// Explicit suppression input; existing rows use inferred false initialization.
    pub is_suppressible: bool,
    /// Explicit locking input; the API documents a false default.
    pub is_locked: bool,
    /// Seconds remaining until the POI expires. Drives
    /// `GetAreaPOISecondsLeft`. `None` for permanent POIs.
    pub seconds_left: Option<i32>,
}

pub(crate) fn register_area_poi_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_AreaPoiInfo")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAreaPOIInfo",
        c_area_poi_info_get_area_poi_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAreaPOISecondsLeft",
        c_area_poi_info_get_area_poi_seconds_left,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAreaPOIForMap",
        c_area_poi_info_get_area_poi_for_map,
    )?;
    Ok(())
}

fn c_area_poi_info_get_area_poi_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = match stack_val(state, 1) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    };
    let area_poi_id = i32::from_stack(state, 2)?;

    let poi = {
        let sim = borrow_state(state)?;
        let Some(poi) = sim.area_pois.get(&area_poi_id).cloned() else {
            return Ok(0);
        };
        if let (Some(requested), Some(owned)) = (ui_map_id, poi.ui_map_id)
            && requested != owned
        {
            return Ok(0);
        }
        poi
    };

    let table = push_area_poi_info_table(state, &poi);
    state.push(table);
    Ok(1)
}

fn c_area_poi_info_get_area_poi_seconds_left(state: &mut LuaState) -> LuaResult<u32> {
    let area_poi_id = i32::from_stack(state, 1)?;
    let seconds = borrow_state(state)?
        .area_pois
        .get(&area_poi_id)
        .and_then(|p| p.seconds_left);
    let Some(seconds) = seconds else {
        return Ok(0);
    };
    state.push(Val::Num(seconds as f64));
    Ok(1)
}

fn c_area_poi_info_get_area_poi_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let mut area_poi_ids = {
        let sim = borrow_state(state)?;
        sim.area_pois
            .values()
            .filter(|poi| poi.ui_map_id == Some(ui_map_id))
            .map(|poi| poi.area_poi_id)
            .collect::<Vec<_>>()
    };
    area_poi_ids.sort_unstable();

    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return table");
    };
    for (index, area_poi_id) in area_poi_ids.into_iter().enumerate() {
        table_set_num(
            state,
            table_ref,
            (index + 1) as f64,
            Val::Num(area_poi_id as f64),
        );
    }
    state.push(table);
    Ok(1)
}

fn push_area_poi_info_table(state: &mut LuaState, poi: &AreaPoiInfo) -> Val {
    let t = create_table(state);
    let name = create_string(state, &poi.name);
    let position = create_table(state);
    table_set_static(state, position, "x", Val::Num(poi.position.0));
    table_set_static(state, position, "y", Val::Num(poi.position.1));
    let Val::Table(position_ref) = position else {
        unreachable!("create_table must return table");
    };
    table_set_rust_fn_static(state, position_ref, "GetXY", area_poi_position_get_xy)
        .expect("failed to register area poi position getter");

    table_set_static(state, t, "areaPoiID", Val::Num(poi.area_poi_id as f64));
    push_optional_number(state, t, "uiMapID", poi.ui_map_id);
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "position", Val::Table(position_ref));
    table_set_static(state, t, "isCurrentEvent", Val::Bool(poi.is_current_event));
    table_set_static(state, t, "shouldGlow", Val::Bool(poi.should_glow));
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    {
        table_set_static(state, t, "isSuppressible", Val::Bool(poi.is_suppressible));
        table_set_static(state, t, "isLocked", Val::Bool(poi.is_locked));
    }
    table_set_static(state, t, "highlightVignettesOnHover", Val::Bool(false));
    table_set_static(state, t, "highlightWorldQuestsOnHover", Val::Bool(false));
    table_set_static(state, t, "isAlwaysOnFlightmap", Val::Bool(false));
    table_set_static(state, t, "isPrimaryMapForPOI", Val::Bool(true));

    push_optional_string(state, t, "atlasName", poi.atlas_name.as_deref());
    push_optional_string(state, t, "description", poi.description.as_deref());
    push_optional_string(state, t, "uiTextureKit", None);
    push_optional_number(state, t, "factionID", poi.faction_id);
    push_optional_number(state, t, "iconWidgetSet", poi.icon_widget_set);
    push_optional_number(state, t, "linkedUiMapID", poi.linked_ui_map_id);
    t
}

fn push_optional_string(state: &mut LuaState, t: Val, key: &str, value: Option<&str>) {
    match value {
        Some(s) => {
            let v = create_string(state, s);
            table_set(state, t, key, v);
        }
        None => table_set(state, t, key, Val::Nil),
    }
}

fn push_optional_number(state: &mut LuaState, t: Val, key: &str, value: Option<i32>) {
    match value {
        Some(n) => table_set(state, t, key, Val::Num(n as f64)),
        None => table_set(state, t, key, Val::Nil),
    }
}

fn area_poi_position_get_xy(state: &mut LuaState) -> LuaResult<u32> {
    let position = stack_val(state, 1);
    let x = match table_get(state, position, "x") {
        Val::Num(value) => value,
        _ => 0.0,
    };
    let y = match table_get(state, position, "y") {
        Val::Num(value) => value,
        _ => 0.0,
    };
    state.push(Val::Num(x));
    state.push(Val::Num(y));
    Ok(2)
}
