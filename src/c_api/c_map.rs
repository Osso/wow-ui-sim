//! `C_Map` probe surface backed by `SimState.maps`, Retail UiMap data,
//! and `SimState.player_map_position`.
//!
//! Migrates 7 entries off `NAMESPACE_NIL_STUBS`:
//!
//! - `C_Map.GetMapArtID(uiMapID)` — returns the `art_id` for the
//!   seeded map, or nothing (retail `mayreturnnothing`).
//! - `C_Map.GetMapInfo(uiMapID)` — returns a `UiMapDetails`-shaped
//!   table for seeded maps and unseeded Retail UiMap records. Unknown
//!   Retail IDs return nothing; other profiles retain generic details for
//!   positive unseeded IDs.
//! - `C_Map.GetMapInfoAtPosition(uiMapID, normalizedX, normalizedY)`
//!   — resolves a normalized point on a parent map back to the leaf
//!   zone whose `child_rects` entry contains the point. Returns
//!   nothing when the parent is unknown, the point falls outside
//!   every rect, or the matched child id is missing from
//!   `SimState.maps`.
//! - `C_Map.GetMapRectOnMap(uiMapID, topMapID)` — projects the rect
//!   of `uiMapID` onto `topMapID`'s coordinate space by composing
//!   each `rect_on_parent` along the parent chain. Returns nothing
//!   when `topMapID` is not an ancestor or any link in the chain
//!   lacks a seeded `rect_on_parent`.
//! - `C_Map.GetMapChildrenInfo(uiMapID, mapType?, allDescendants?)`
//!   — returns seeded children and, on Retail, children from UiMap parent
//!   links as `UiMapDetails` tables. `mapType` filters by the UIMapType
//!   enum; `allDescendants` recursively walks the subtree. Unknown maps
//!   return nothing; known maps with no matching children return an empty array.
//! - `C_Map.GetMapGroupID` and `GetMapGroupMembersInfo` on Retail and
//!   Forever read explicitly supplied map-group membership (INFERRED, R88cy pending).
//! - `C_Map.GetPlayerMapPosition(uiMapID, unitToken)` — returns
//!   `{x, y}` vector2 from `SimState.player_map_position` for any
//!   known map, or matching explicit input for an active party member.
//!   Returns `nil` when no position is available.
//! - `C_Map.GetBestMapForUnit` and `C_Map.GetFallbackWorldMapID` read
//!   the simulator quest-log selection on Retail and Forever, defaulting
//!   to the initial player map. Native equivalence is not established.
//! - `C_Map.MapHasArt(uiMapID)` — true for positive map ids.
//! - `C_Map.RequestPreloadMap(uiMapID)` — queues map art + overlay textures.

use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::globals::unit_api::parse_party_index;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_capacity,
    table_get, table_set, table_set_static, val_to_string,
};
use crate::lua_api::state::MapData;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::val_to_i32;
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
#[cfg(any(
    all(feature = "profile-retail", feature = "retail-12-1-5"),
    feature = "client-retail",
    feature = "client-wowforever"
))]
use rilua::runtime_error;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

mod quest_log_selection;
mod user_waypoint;

pub(crate) use quest_log_selection::uses_rust_quest_log_selection;

type LuaTableRef = GcRef<Table>;
type RustLuaFn = rilua::vm::closure::RustFn;

/// Explicit normalized position associated with one UI map.
#[derive(Clone, Copy, Debug)]
pub struct UnitMapPosition {
    pub ui_map_id: i32,
    pub position: (f64, f64),
}

/// Session-only waypoint state; account/character ownership and persistence are UNKNOWN.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UserWaypoint {
    pub(crate) ui_map_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: Option<f64>,
}

impl UserWaypoint {
    pub(crate) fn is_valid(&self) -> bool {
        self.ui_map_id != 0 && is_normalized_coordinate(self.x) && is_normalized_coordinate(self.y)
    }
}

fn is_normalized_coordinate(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

const MAP_ART_LAYER_HASH_FIELDS: usize = 7;

const C_MAP_METHODS: &[(&str, RustLuaFn)] = &[
    (
        "GetMapArtBackgroundAtlas",
        c_map_get_map_art_background_atlas,
    ),
    ("GetMapArtID", c_map_get_map_art_id),
    ("GetMapArtLayerTextures", c_map_get_map_art_layer_textures),
    ("GetMapArtLayers", c_map_get_map_art_layers),
    #[cfg(feature = "retail-12-1-0")]
    ("GetMapDisplayInfo", c_map_get_map_display_info),
    #[cfg(feature = "profile-retail")]
    ("GetAreaInfo", c_map_get_area_info),
    ("GetMapInfo", c_map_get_map_info),
    ("GetMapInfoAtPosition", c_map_get_map_info_at_position),
    ("GetMapRectOnMap", c_map_get_map_rect_on_map),
    ("GetMapChildrenInfo", c_map_get_map_children_info),
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    ("GetMapGroupID", c_map_get_map_group_id),
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    ("GetMapGroupMembersInfo", c_map_get_map_group_members_info),
    ("GetWorldPosFromMapPos", c_map_get_world_pos_from_map_pos),
    ("GetPlayerMapPosition", c_map_get_player_map_position),
    ("GetBestMapForUnit", c_map_get_best_map_for_unit),
    ("GetCurrentMapID", c_map_get_current_map_id),
    ("GetFallbackWorldMapID", c_map_get_fallback_world_map_id),
    ("MapHasArt", c_map_map_has_art),
    ("RequestPreloadMap", c_map_request_preload_map),
];

pub(crate) fn register_c_map_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Map")?;
    register_c_map_methods(state, table_ref)?;
    quest_log_selection::register(state, table_ref)?;
    super::c_map_bounties::register(state)?;
    super::map_world_coordinates::register(state)
}

fn register_c_map_methods(state: &mut LuaState, table_ref: LuaTableRef) -> LuaResult<()> {
    for (name, rust_fn) in C_MAP_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *rust_fn)?;
    }
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        table_set_rust_fn_static(
            state,
            table_ref,
            "CloseWorldMapInteraction",
            close_world_map_interaction,
        )?;
    }
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Mists
            | crate::client_profile::ClientProfile::Era
            | crate::client_profile::ClientProfile::Anniversary
    ) {
        table_set_rust_fn_static(
            state,
            table_ref,
            "GetMapDisplayInfo",
            c_map_get_map_display_info,
        )?;
    }
    user_waypoint::register(state, table_ref)?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetMapGroupMember", set_map_group_member)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_map_group_member(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = i32::from_stack(state, 1)?;
    let group_id = i32::from_stack(state, 2)?;
    let relative_height_index = i32::from_stack(state, 3)?;
    let Some(map) = borrow_state_mut(state)?.maps.get_mut(&map_id) else {
        return Err(runtime_error(format!(
            "A_Admin.SetMapGroupMember: unknown map {map_id}"
        )));
    };
    map.map_group_id = Some(group_id);
    map.map_group_relative_height_index = relative_height_index;
    Ok(0)
}

fn close_world_map_interaction(state: &mut LuaState) -> LuaResult<u32> {
    const WORLD_MAP_INTERACTION: i32 = 29;
    let was_active = borrow_state_mut(state)?
        .active_player_interactions
        .remove(&WORLD_MAP_INTERACTION);
    if was_active {
        crate::lua_api::script_helpers::fire_named_event_state(
            state,
            "PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
            &[Val::Num(f64::from(WORLD_MAP_INTERACTION))],
        );
    }
    Ok(0)
}

const DEFAULT_PLAYER_MAP_ID: i32 = 2248;
const DEFAULT_MAP_ART_BACKGROUND_ATLAS: &str = "AdventureMap_TileBg";
const MAP_DETAILS_HASH_FIELDS: usize = 5;
const MAX_GENERIC_UI_MAP_ID: i32 = 10_000;

fn c_map_get_map_art_background_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    if crate::map_art::get_map_art(ui_map_id as u32).is_none() {
        return Ok(0);
    }
    let atlas = create_string(state, DEFAULT_MAP_ART_BACKGROUND_ATLAS);
    state.push(atlas);
    Ok(1)
}

#[cfg(feature = "profile-retail")]
fn c_map_get_area_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(area_id) = read_optional_area_id(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let name = u32::try_from(area_id)
        .ok()
        .and_then(sim_data_profile::area_names_retail::get_area_name);
    let value = name.map_or(Val::Nil, |name| create_string(state, name));
    state.push(value);
    Ok(1)
}

#[cfg(feature = "profile-retail")]
fn read_optional_area_id(state: &LuaState) -> LuaResult<Option<i32>> {
    #[cfg(feature = "retail-12-1-0")]
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    #[cfg(not(feature = "retail-12-1-0"))]
    let value = stack_val(state, 1);
    if matches!(value, Val::Nil) {
        return Ok(None);
    }
    #[cfg(feature = "retail-12-1-0")]
    {
        val_to_i32(value, 1).map(Some)
    }
    #[cfg(not(feature = "retail-12-1-0"))]
    {
        i32::from_stack(state, 1).map(Some)
    }
}

fn read_scalar_map_id(state: &LuaState) -> LuaResult<i32> {
    #[cfg(feature = "retail-12-1-0")]
    {
        let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
        val_to_i32(value, 1)
    }
    #[cfg(not(feature = "retail-12-1-0"))]
    {
        i32::from_stack(state, 1)
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn c_map_get_map_group_id(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = read_scalar_map_id(state)?;
    let group_id = borrow_state(state)?
        .maps
        .get(&map_id)
        .and_then(|map| map.map_group_id);
    if let Some(group_id) = group_id {
        state.push(Val::Num(f64::from(group_id)));
        return Ok(1);
    }
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn c_map_get_map_group_members_info(state: &mut LuaState) -> LuaResult<u32> {
    let group_id = i32::from_stack(state, 1)?;
    let maps = borrow_state(state)?.maps.clone();
    let members = map_group_members(&maps, group_id);
    if members.is_empty() {
        return Ok(0);
    }
    let result = create_table_with_capacity(state, members.len());
    for (index, map) in members.iter().enumerate() {
        let info = push_map_group_member_info(state, map);
        set_table_array(state, result, index as i64 + 1, info);
    }
    state.push(result);
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn map_group_members(
    maps: &std::collections::HashMap<i32, MapData>,
    group_id: i32,
) -> Vec<&MapData> {
    let mut members: Vec<_> = maps
        .values()
        .filter(|map| map.map_group_id == Some(group_id))
        .collect();
    // INFERRED (R88cy pending): display lower relative-height indexes first.
    members.sort_by_key(|map| (map.map_group_relative_height_index, map.ui_map_id));
    members
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn push_map_group_member_info(state: &mut LuaState, map: &MapData) -> Val {
    let info = create_table_with_capacity(state, 3);
    let name = create_string(state, &map.name);
    table_set_static(state, info, "mapID", Val::Num(f64::from(map.ui_map_id)));
    table_set_static(
        state,
        info,
        "relativeHeightIndex",
        Val::Num(f64::from(map.map_group_relative_height_index)),
    );
    table_set_static(state, info, "name", name);
    info
}

fn c_map_get_map_art_id(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = read_scalar_map_id(state)?;
    let art_id = borrow_state(state)?.maps.get(&ui_map_id).map(|m| m.art_id);
    let Some(art_id) = art_id else {
        return Ok(0);
    };
    state.push(Val::Num(art_id as f64));
    Ok(1)
}

fn c_map_get_map_art_layers(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = Option::<i32>::from_stack(state, 1)?.unwrap_or(DEFAULT_PLAYER_MAP_ID);
    let Some(map_art) = crate::map_art::get_map_art(ui_map_id as u32) else {
        return Ok(0);
    };

    let layers = create_table(state);
    for (index, layer) in map_art.layers.iter().enumerate() {
        let layer_info = create_map_art_layer_table(state, layer);
        set_table_array(state, layers, index as i64 + 1, layer_info);
    }

    state.push(layers);
    Ok(1)
}

fn create_map_art_layer_table(state: &mut LuaState, layer: &crate::map_art::MapArtLayer) -> Val {
    let layer_info = create_table_with_capacity(state, MAP_ART_LAYER_HASH_FIELDS);
    table_set_num_field(state, layer_info, "layerWidth", layer.layer_width as f64);
    table_set_num_field(state, layer_info, "layerHeight", layer.layer_height as f64);
    table_set_num_field(state, layer_info, "tileWidth", layer.tile_width as f64);
    table_set_num_field(state, layer_info, "tileHeight", layer.tile_height as f64);
    table_set_num_field(state, layer_info, "minScale", layer.min_scale as f64);
    table_set_num_field(state, layer_info, "maxScale", layer.max_scale as f64);
    table_set_num_field(
        state,
        layer_info,
        "additionalZoomSteps",
        layer.additional_zoom_steps as f64,
    );
    layer_info
}

fn table_set_num_field(state: &mut LuaState, table: Val, key: &str, value: f64) {
    table_set(state, table, key, Val::Num(value));
}

fn c_map_get_map_art_layer_textures(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let layer_index = i32::from_stack(state, 2)?;
    if layer_index < 1 {
        return Ok(0);
    }

    let Some(map_art) = crate::map_art::get_map_art(ui_map_id as u32) else {
        return Ok(0);
    };
    let Some(textures) = map_art.tiles.get((layer_index - 1) as usize) else {
        return Ok(0);
    };

    let texture_ids = create_table(state);
    for (index, file_data_id) in textures.iter().copied().enumerate() {
        set_table_array(
            state,
            texture_ids,
            index as i64 + 1,
            Val::Num(file_data_id as f64),
        );
    }
    state.push(texture_ids);
    Ok(1)
}

fn read_map_display_id(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(
            "map display selector #1 requires a number",
        ));
    };
    let is_exact_integer = number.is_finite() && number.fract() == 0.0;
    let is_in_range = (i32::MIN as f64..=i32::MAX as f64).contains(&number);
    if !is_exact_integer || !is_in_range {
        return Err(rilua::runtime_error(
            "map display selector #1 requires an integer representable as i32",
        ));
    }
    Ok(number as i32)
}

fn c_map_get_map_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = read_map_display_id(state)?;
    let hide_icons = borrow_state(state)?
        .map_display_hide_icons
        .get(&ui_map_id)
        .copied();
    let Some(hide_icons) = hide_icons else {
        return Ok(0);
    };
    state.push(Val::Bool(hide_icons));
    Ok(1)
}

fn c_map_get_map_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = read_scalar_map_id(state)?;
    let map = borrow_state(state)?.maps.get(&ui_map_id).cloned();
    let details = match map {
        Some(map) => push_map_details_table(state, &map),
        None => match ui_map_table_entry(ui_map_id) {
            Some(info) => push_ui_map_table_details(state, ui_map_id, info),
            None if should_return_generic_map_info(ui_map_id) => {
                push_generic_map_details_table(state, ui_map_id)
            }
            None => return Ok(0),
        },
    };
    state.push(details);
    Ok(1)
}

fn should_return_generic_map_info(ui_map_id: i32) -> bool {
    crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail
        && (1..=MAX_GENERIC_UI_MAP_ID).contains(&ui_map_id)
}

fn c_map_get_map_info_at_position(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let normalized_x = f64::from_stack(state, 2)?;
    let normalized_y = f64::from_stack(state, 3)?;
    let resolved = {
        let sim = borrow_state(state)?;
        let parent = sim.maps.get(&ui_map_id);
        let child_id = parent
            .and_then(|map| find_child_at_point(&map.child_rects, normalized_x, normalized_y));
        child_id.and_then(|id| sim.maps.get(&id).cloned())
    };
    let Some(child) = resolved else {
        return Ok(0);
    };
    let details = push_map_details_table(state, &child);
    state.push(details);
    Ok(1)
}

fn find_child_at_point(
    rects: &[crate::lua_api::state::MapChildRect],
    x: f64,
    y: f64,
) -> Option<i32> {
    rects
        .iter()
        .find(|rect| x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom)
        .map(|rect| rect.map_id)
}

const MAP_RECT_MAX_DEPTH: usize = 16;

fn c_map_get_map_rect_on_map(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let top_map_id = i32::from_stack(state, 2)?;
    let rect = {
        let sim = borrow_state(state)?;
        compose_rect_up_chain(&sim.maps, ui_map_id, top_map_id)
    };
    let Some(rect) = rect else {
        return Ok(0);
    };
    state.push(Val::Num(rect.left));
    state.push(Val::Num(rect.right));
    state.push(Val::Num(rect.top));
    state.push(Val::Num(rect.bottom));
    Ok(4)
}

/// Walk parent links from `ui_map_id` up to `top_map_id`, composing
/// each `rect_on_parent` so the returned rect describes `ui_map_id`'s
/// footprint inside `top_map_id`'s coordinate space. Identity rect is
/// returned when `ui_map_id == top_map_id`. Returns `None` when the
/// chain misses `top_map_id`, exceeds `MAP_RECT_MAX_DEPTH`, or hits a
/// link without a seeded `rect_on_parent`.
fn compose_rect_up_chain(
    maps: &std::collections::HashMap<i32, crate::lua_api::state::MapData>,
    ui_map_id: i32,
    top_map_id: i32,
) -> Option<crate::lua_api::state::MapRect> {
    let mut rect = crate::lua_api::state::MapRect {
        left: 0.0,
        right: 1.0,
        top: 0.0,
        bottom: 1.0,
    };
    let mut current = ui_map_id;
    for _ in 0..MAP_RECT_MAX_DEPTH {
        if current == top_map_id {
            return Some(rect);
        }
        let map = maps.get(&current)?;
        let parent_rect = map.rect_on_parent?;
        rect = compose_rect_in_parent(parent_rect, rect);
        current = map.parent_map_id;
    }
    None
}

/// Project an inner rect (in `current`'s normalized space) into the
/// parent's normalized space, given the placement of `current` inside
/// the parent (`parent_rect`).
fn compose_rect_in_parent(
    parent_rect: crate::lua_api::state::MapRect,
    inner: crate::lua_api::state::MapRect,
) -> crate::lua_api::state::MapRect {
    let span_x = parent_rect.right - parent_rect.left;
    let span_y = parent_rect.bottom - parent_rect.top;
    crate::lua_api::state::MapRect {
        left: parent_rect.left + inner.left * span_x,
        right: parent_rect.left + inner.right * span_x,
        top: parent_rect.top + inner.top * span_y,
        bottom: parent_rect.top + inner.bottom * span_y,
    }
}

fn push_map_details_table(state: &mut LuaState, map: &MapData) -> Val {
    let t = create_table_with_capacity(state, MAP_DETAILS_HASH_FIELDS);
    let name = create_string(state, &map.name);
    table_set_static(state, t, "mapID", Val::Num(map.ui_map_id as f64));
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "mapType", Val::Num(map.map_type as f64));
    table_set_static(state, t, "parentMapID", Val::Num(map.parent_map_id as f64));
    table_set_static(state, t, "flags", Val::Num(map.flags as f64));
    t
}

/// The client's UiMap row for a map the hand seeds do not carry. Seeded maps
/// win because they also carry rects and children; this table only knows
/// name, type, parent and flags, which is what `GetMapInfo` reports.
fn ui_map_table_entry(ui_map_id: i32) -> Option<&'static crate::ui_maps::UiMapInfo> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return None;
    }
    u32::try_from(ui_map_id)
        .ok()
        .and_then(crate::ui_maps::get_ui_map)
}

enum MapInfoSource {
    Seeded(MapData),
    UiMap {
        ui_map_id: i32,
        info: &'static crate::ui_maps::UiMapInfo,
    },
}

impl MapInfoSource {
    fn map_type(&self) -> i32 {
        match self {
            Self::Seeded(map) => map.map_type,
            Self::UiMap { info, .. } => info.map_type as i32,
        }
    }

    fn push_details(self, state: &mut LuaState) -> Val {
        match self {
            Self::Seeded(map) => push_map_details_table(state, &map),
            Self::UiMap { ui_map_id, info } => push_ui_map_table_details(state, ui_map_id, info),
        }
    }
}

fn map_info_source(
    maps: &std::collections::HashMap<i32, MapData>,
    ui_map_id: i32,
) -> Option<MapInfoSource> {
    if let Some(map) = maps.get(&ui_map_id) {
        return Some(MapInfoSource::Seeded(map.clone()));
    }
    let info = ui_map_table_entry(ui_map_id)?;
    Some(MapInfoSource::UiMap { ui_map_id, info })
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn map_type_for_ui_map_id(
    maps: &std::collections::HashMap<i32, MapData>,
    ui_map_id: i32,
) -> Option<i32> {
    map_info_source(maps, ui_map_id).map(|info| info.map_type())
}

type MapChildrenByParent = std::collections::HashMap<i32, Vec<i32>>;

fn insert_map_child(children: &mut MapChildrenByParent, parent_id: i32, child_id: i32) {
    let child_ids = children.entry(parent_id).or_default();
    if !child_ids.contains(&child_id) {
        child_ids.push(child_id);
    }
}

fn append_explicit_map_children(
    maps: &std::collections::HashMap<i32, MapData>,
    children: &mut MapChildrenByParent,
) {
    for map in maps.values() {
        children
            .entry(map.ui_map_id)
            .or_default()
            .extend(map.child_map_ids.iter().copied());
    }
}

fn append_seeded_map_parent_links(
    maps: &std::collections::HashMap<i32, MapData>,
    children: &mut MapChildrenByParent,
) {
    let mut parent_links = maps
        .values()
        .map(|map| (map.parent_map_id, map.ui_map_id))
        .collect::<Vec<_>>();
    parent_links.sort_unstable();
    for (parent_id, child_id) in parent_links {
        insert_map_child(children, parent_id, child_id);
    }
}

fn append_ui_map_catalog_children(
    maps: &std::collections::HashMap<i32, MapData>,
    children: &mut MapChildrenByParent,
) {
    let mut parent_links = crate::ui_maps::UI_MAP_DB
        .entries()
        .filter_map(|(ui_map_id, info)| {
            let ui_map_id = i32::try_from(*ui_map_id).ok()?;
            if maps.contains_key(&ui_map_id) {
                return None;
            }
            let parent_id = i32::try_from(info.parent_map_id).ok()?;
            Some((parent_id, ui_map_id))
        })
        .collect::<Vec<_>>();
    parent_links.sort_unstable();
    for (parent_id, child_id) in parent_links {
        insert_map_child(children, parent_id, child_id);
    }
}

fn map_children_by_parent(maps: &std::collections::HashMap<i32, MapData>) -> MapChildrenByParent {
    let mut children = MapChildrenByParent::new();
    append_explicit_map_children(maps, &mut children);
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        append_seeded_map_parent_links(maps, &mut children);
        append_ui_map_catalog_children(maps, &mut children);
    }
    children
}

fn push_ui_map_table_details(
    state: &mut LuaState,
    ui_map_id: i32,
    info: &crate::ui_maps::UiMapInfo,
) -> Val {
    let t = create_table_with_capacity(state, MAP_DETAILS_HASH_FIELDS);
    let name = create_string(state, info.name);
    table_set_static(state, t, "mapID", Val::Num(ui_map_id as f64));
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "mapType", Val::Num(info.map_type as f64));
    table_set_static(state, t, "parentMapID", Val::Num(info.parent_map_id as f64));
    table_set_static(state, t, "flags", Val::Num(info.flags as f64));
    t
}

fn push_generic_map_details_table(state: &mut LuaState, ui_map_id: i32) -> Val {
    let t = create_table_with_capacity(state, MAP_DETAILS_HASH_FIELDS);
    let name = create_string(state, &format!("Map {ui_map_id}"));
    table_set_static(state, t, "mapID", Val::Num(ui_map_id as f64));
    table_set_static(state, t, "name", name);
    table_set_static(state, t, "mapType", Val::Num(3.0));
    table_set_static(state, t, "parentMapID", Val::Num(0.0));
    table_set_static(state, t, "flags", Val::Num(0.0));
    t
}

fn collect_children(
    maps: &std::collections::HashMap<i32, MapData>,
    root: i32,
    all_descendants: bool,
    map_type_filter: Option<i32>,
) -> Option<Vec<MapInfoSource>> {
    map_info_source(maps, root)?;
    let children_by_parent = map_children_by_parent(maps);
    Some(collect_children_from_index(
        maps,
        &children_by_parent,
        root,
        all_descendants,
        map_type_filter,
    ))
}

fn append_map_children(
    frontier: &mut Vec<i32>,
    children_by_parent: &MapChildrenByParent,
    parent_id: i32,
) {
    if let Some(children) = children_by_parent.get(&parent_id) {
        frontier.extend(children.iter().copied());
    }
}

fn collect_children_from_index(
    maps: &std::collections::HashMap<i32, MapData>,
    children_by_parent: &MapChildrenByParent,
    root: i32,
    all_descendants: bool,
    map_type_filter: Option<i32>,
) -> Vec<MapInfoSource> {
    let mut out = Vec::new();
    let mut visited = HashSet::new();
    let mut frontier = children_by_parent.get(&root).cloned().unwrap_or_default();

    while let Some(child_id) = frontier.pop() {
        if !visited.insert(child_id) {
            continue;
        }
        let Some(child) = map_info_source(maps, child_id) else {
            continue;
        };
        if map_type_filter.is_none_or(|filter| child.map_type() == filter) {
            out.push(child);
        }
        if all_descendants {
            append_map_children(&mut frontier, children_by_parent, child_id);
        }
    }

    out
}

fn c_map_get_map_children_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let map_type_filter = match stack_val(state, 2) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    };
    let all_descendants = matches!(stack_val(state, 3), Val::Bool(true));
    let children = collect_children(
        &borrow_state(state)?.maps,
        ui_map_id,
        all_descendants,
        map_type_filter,
    );
    let Some(children) = children else {
        return Ok(0);
    };

    let array = create_table(state);
    for (index, child) in children.into_iter().enumerate() {
        let entry = child.push_details(state);
        set_table_array(state, array, index as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
const GET_WORLD_POS_USAGE: &str =
    "Usage: local continentID, worldPosition = C_Map.GetWorldPosFromMapPos(uiMapID, mapPosition)";

fn c_map_get_world_pos_from_map_pos(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    let ui_map_id = read_retail_map_id(state, 1)?;
    #[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
    let ui_map_id = i32::from_stack(state, 1)?;

    let normalized_pos = stack_val(state, 2);
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    let map_position = read_world_map_position(state, normalized_pos)?;
    let (map_is_known, explicit_rect) = {
        let sim = borrow_state(state)?;
        (
            sim.maps.contains_key(&ui_map_id),
            sim.map_world_rects.get(&ui_map_id).copied(),
        )
    };

    // Explicit simulator rectangles take precedence over generated Retail assignments.
    if map_is_known {
        if let Some(rect) = explicit_rect {
            #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
            let (normalized_x, normalized_y) = map_position;
            #[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
            let Some((normalized_x, normalized_y)) = normalized_coordinates(state, normalized_pos)
            else {
                return Ok(0);
            };
            let Some((x, y)) = rect.unproject(normalized_x, normalized_y) else {
                return Ok(0);
            };
            let world_pos = create_world_position_vector(state, x, y)?;
            state.push(Val::Num(f64::from(rect.continent_id)));
            state.push(world_pos);
            return Ok(2);
        }
    }

    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    if ui_map_table_entry(ui_map_id).is_none() {
        return Ok(0);
    }

    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    {
        let Some((continent_id, x, y)) =
            retail_map_assignment_world_position(ui_map_id, map_position)
        else {
            return Ok(0);
        };
        let world_pos = create_world_position_vector(state, x, y)?;
        state.push(Val::Num(f64::from(continent_id)));
        state.push(world_pos);
        return Ok(2);
    }

    #[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
    {
        if !map_is_known {
            return Ok(0);
        }
        let (width, height) = map_world_size(ui_map_id);
        let x = normalized_coordinate(state, normalized_pos, "x") * width;
        let y = normalized_coordinate(state, normalized_pos, "y") * height;
        let world_pos = create_world_position_vector(state, x, y)?;
        state.push(Val::Num(0.0));
        state.push(world_pos);
        Ok(2)
    }
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn read_retail_map_id(state: &LuaState, index: i32) -> LuaResult<i32> {
    let value = f64::from_stack(state, index)
        .map_err(|_| runtime_error(format!("{GET_WORLD_POS_USAGE}: invalid uiMapID")))?;
    if !value.is_finite() || value < f64::from(i32::MIN) || value > f64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "{GET_WORLD_POS_USAGE}: invalid uiMapID"
        )));
    }
    Ok(value.trunc() as i32)
}

#[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
fn normalized_coordinates(state: &mut LuaState, position: Val) -> Option<(f64, f64)> {
    let Val::Num(x) = table_get(state, position, "x") else {
        return None;
    };
    let Val::Num(y) = table_get(state, position, "y") else {
        return None;
    };
    (x.is_finite() && y.is_finite()).then_some((x, y))
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn read_world_map_position(state: &mut LuaState, position: Val) -> LuaResult<(f64, f64)> {
    let Val::Table(_) = position else {
        return Err(runtime_error(GET_WORLD_POS_USAGE));
    };
    let Val::Num(x) = table_get(state, position, "x") else {
        return Err(runtime_error(GET_WORLD_POS_USAGE));
    };
    let Val::Num(y) = table_get(state, position, "y") else {
        return Err(runtime_error(GET_WORLD_POS_USAGE));
    };
    Ok((x, y))
}

/// VERIFIED: choose the containing eligible row nearest its UI-rectangle centre,
/// or the nearest centre when no row contains the point. CSV tie order is INFERRED.
#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn select_retail_map_assignment(
    assignments: &[sim_data_profile::ui_map_assignments::UiMapAssignment],
    x: f64,
    y: f64,
) -> Option<(&sim_data_profile::ui_map_assignments::UiMapAssignment, i32)> {
    let mut containing = None;
    let mut nearest = None;
    for assignment in assignments {
        let Some(map_number) = assignment_map_number(
            assignment,
            sim_data_profile::area_table_retail::get_continent_id,
        ) else {
            continue;
        };
        let Some((inside, score)) = assignment_geometry_score(assignment, x, y) else {
            continue;
        };
        let best = if inside {
            &mut containing
        } else {
            &mut nearest
        };
        retain_closest_assignment(best, assignment, map_number, score);
    }
    containing
        .or(nearest)
        .map(|(assignment, map_number, _)| (assignment, map_number))
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn assignment_map_number(
    assignment: &sim_data_profile::ui_map_assignments::UiMapAssignment,
    area_continent: impl FnOnce(u32) -> Option<u16>,
) -> Option<i32> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return None;
    }
    if assignment.map_id >= 0 {
        return Some(assignment.map_id);
    }
    // VERIFIED: an unresolved AreaID disqualifies this row; INFERRED: map the u16 to ContinentID.
    area_continent(assignment.area_id).map(i32::from)
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn assignment_geometry_score(
    assignment: &sim_data_profile::ui_map_assignments::UiMapAssignment,
    x: f64,
    y: f64,
) -> Option<(bool, f64)> {
    let min_x = f64::from(assignment.ui_min[0]);
    let min_y = f64::from(assignment.ui_min[1]);
    let max_x = f64::from(assignment.ui_max[0]);
    let max_y = f64::from(assignment.ui_max[1]);
    let span_x = max_x - min_x;
    let span_y = max_y - min_y;
    let epsilon = f64::from(f32::EPSILON);
    if span_x < epsilon || span_y < epsilon {
        return None;
    }
    let center_x = min_x + span_x * 0.5;
    let center_y = min_y + span_y * 0.5;
    let delta_x = x - center_x;
    let delta_y = y - center_y;
    let inside = x >= min_x && x <= max_x && y >= min_y && y <= max_y;
    Some((inside, delta_x * delta_x + delta_y * delta_y))
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn retain_closest_assignment<'a>(
    best: &mut Option<(
        &'a sim_data_profile::ui_map_assignments::UiMapAssignment,
        i32,
        f64,
    )>,
    assignment: &'a sim_data_profile::ui_map_assignments::UiMapAssignment,
    map_number: i32,
    score: f64,
) {
    if best
        .as_ref()
        .is_none_or(|(_, _, best_score)| score < *best_score)
    {
        *best = Some((assignment, map_number, score));
    }
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn retail_map_assignment_world_position(
    ui_map_id: i32,
    (x, y): (f64, f64),
) -> Option<(i32, f64, f64)> {
    let ui_map_id = u32::try_from(ui_map_id).ok()?;
    let assignments = sim_data_profile::ui_map_assignments::get_ui_map_assignments(ui_map_id)?;
    let (assignment, map_number) = select_retail_map_assignment(assignments, x, y)?;
    let (world_x, world_y) = project_retail_assignment(assignment, x, y);
    Some((map_number, world_x, world_y))
}

#[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
fn project_retail_assignment(
    assignment: &sim_data_profile::ui_map_assignments::UiMapAssignment,
    x: f64,
    y: f64,
) -> (f64, f64) {
    let ui_width = f64::from(assignment.ui_max[0]) - f64::from(assignment.ui_min[0]);
    let ui_height = f64::from(assignment.ui_max[1]) - f64::from(assignment.ui_min[1]);
    let local_x = (x - f64::from(assignment.ui_min[0])) / ui_width;
    let local_y = (y - f64::from(assignment.ui_min[1])) / ui_height;
    // INFERRED (R88i.41): world X blends Region_3 toward Region_0 by map Y.
    let world_x = f64::from(assignment.region_max[0])
        + (f64::from(assignment.region_min[0]) - f64::from(assignment.region_max[0])) * local_y;
    let world_y = f64::from(assignment.region_max[1])
        + (f64::from(assignment.region_min[1]) - f64::from(assignment.region_max[1])) * local_x;
    (world_x, world_y)
}

#[cfg(all(
    test,
    feature = "optional-tests",
    feature = "profile-retail",
    feature = "retail-12-1-5"
))]
mod retail_assignment_tests {
    use super::{assignment_map_number, select_retail_map_assignment};
    use sim_data_profile::ui_map_assignments::UiMapAssignment;

    fn negative_assignment(area_id: u32) -> UiMapAssignment {
        UiMapAssignment {
            map_id: -1,
            area_id,
            ui_min: [0.0, 0.0],
            ui_max: [1.0, 1.0],
            region_min: [0.0, 0.0],
            region_max: [1.0, 1.0],
        }
    }

    #[test]
    fn negative_assignment_accepts_zero_continent_number() {
        let assignment = negative_assignment(7);
        assert_eq!(assignment_map_number(&assignment, |_| Some(0)), Some(0));
    }

    #[test]
    fn unresolved_negative_assignment_skips_to_later_usable_row() {
        let assignments = [negative_assignment(0), negative_assignment(14620)];
        let (selected, map_number) = select_retail_map_assignment(&assignments, 0.5, 0.5).unwrap();
        assert_eq!(selected.area_id, 14620);
        assert!(u16::try_from(map_number).is_ok());
    }
}

#[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
fn map_world_size(ui_map_id: i32) -> (f64, f64) {
    let Some(map_art) = crate::map_art::get_map_art(ui_map_id as u32) else {
        return (1000.0, 1000.0);
    };
    let Some(layer) = map_art.layers.first() else {
        return (1000.0, 1000.0);
    };
    (layer.layer_width as f64, layer.layer_height as f64)
}

#[cfg(not(all(feature = "profile-retail", feature = "retail-12-1-5")))]
fn normalized_coordinate(state: &mut LuaState, position: Val, key: &str) -> f64 {
    match table_get(state, position, key) {
        Val::Num(value) => value.clamp(0.0, 1.0),
        _ => 0.5,
    }
}

pub(super) fn create_world_position_vector(state: &mut LuaState, x: f64, y: f64) -> LuaResult<Val> {
    let vector = create_table(state);
    table_set_static(state, vector, "x", Val::Num(x));
    table_set_static(state, vector, "y", Val::Num(y));
    let Val::Table(vector_ref) = vector else {
        unreachable!("create_table must return table");
    };
    table_set_rust_fn_static(state, vector_ref, "GetXY", world_position_get_xy)?;
    Ok(Val::Table(vector_ref))
}

fn world_position_get_xy(state: &mut LuaState) -> LuaResult<u32> {
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

fn c_map_get_player_map_position(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let unit_token = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let is_player = matches!(unit_token.as_str(), "player" | "");

    let position = {
        let sim = borrow_state(state)?;
        if !sim.maps.contains_key(&ui_map_id) {
            None
        } else if is_player {
            Some(sim.player_map_position)
        } else if sim.party_group_active {
            parse_party_index(&unit_token)
                .and_then(|index| sim.party_members.get(index))
                .and_then(|member| member.map_position)
                .filter(|input| input.ui_map_id == ui_map_id)
                .map(|input| input.position)
        } else {
            None
        }
    };

    let Some(position) = position else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let position = create_world_position_vector(state, position.0, position.1)?;
    state.push(position);
    Ok(1)
}

fn c_map_get_best_map_for_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = stack_val(state, 1);
    let unit_token = val_to_string(state, unit).unwrap_or_default();
    if uses_rust_quest_log_selection() {
        if unit != Val::Nil && unit_token != "player" {
            state.push(Val::Nil);
            return Ok(1);
        }
    } else if !matches!(unit_token.as_str(), "" | "player") {
        return Ok(0);
    }
    c_map_get_current_map_id(state)
}

pub(crate) fn best_map_for_player(state: &LuaState) -> LuaResult<f64> {
    Ok(borrow_state(state)?
        .quest_log_selected_map_id
        .unwrap_or(f64::from(DEFAULT_PLAYER_MAP_ID)))
}

fn c_map_get_current_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let current = best_map_for_player(state)?;
    state.push(Val::Num(current));
    Ok(1)
}

fn c_map_get_fallback_world_map_id(state: &mut LuaState) -> LuaResult<u32> {
    c_map_get_current_map_id(state)
}

fn c_map_map_has_art(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(ui_map_id > 0));
    Ok(1)
}

fn c_map_request_preload_map(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let queued_paths = collect_preload_paths_for_map(ui_map_id);
    borrow_state_mut(state)?.enqueue_texture_preloads(queued_paths);
    Ok(0)
}

fn collect_preload_paths_for_map(ui_map_id: i32) -> Vec<String> {
    let mut paths = Vec::new();
    if let Some(art_info) = crate::map_art::get_map_art(ui_map_id as u32) {
        for file_data_id in art_info
            .tiles
            .iter()
            .flat_map(|tiles| tiles.iter().copied())
        {
            if let Some(path) = file_data_id_to_wow_path(file_data_id) {
                paths.push(path);
            }
        }
    }
    if let Some(overlays) = crate::map_exploration::get_overlays_for_map(ui_map_id as u32) {
        for file_data_id in overlays
            .iter()
            .flat_map(|overlay| overlay.file_data_ids.iter().copied())
        {
            if let Some(path) = file_data_id_to_wow_path(file_data_id) {
                paths.push(path);
            }
        }
    }
    paths
}

fn file_data_id_to_wow_path(file_data_id: u32) -> Option<String> {
    let path = crate::manifest_interface_data::get_texture_path(file_data_id)?;
    Some(format!("Interface\\{}", path.replace('/', "\\")))
}
