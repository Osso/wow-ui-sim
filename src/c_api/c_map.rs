//! `C_Map` probe surface backed by `SimState.maps` +
//! `SimState.player_map_position`.
//!
//! Migrates 7 entries off `NAMESPACE_NIL_STUBS`:
//!
//! - `C_Map.GetMapArtID(uiMapID)` — returns the `art_id` for the
//!   seeded map, or nothing (retail `mayreturnnothing`).
//! - `C_Map.GetMapInfo(uiMapID)` — returns a `UiMapDetails`-shaped
//!   table for seeded maps. For valid-looking positive map IDs that are
//!   not yet seeded, it returns a minimal generic map record instead of
//!   crashing addon startup paths that walk retail UiMap.db2 IDs. Absurd
//!   sentinel IDs still return nothing.
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
//!   — returns the children as an array of `UiMapDetails` tables.
//!   `mapType` filters by the UIMapType enum; `allDescendants`
//!   recursively walks the subtree. Returns nothing when the map is
//!   unknown, an empty array when it exists but has no children
//!   matching the filter.
//! - `C_Map.GetPlayerMapPosition(uiMapID, unitToken)` — returns
//!   `{x, y}` vector2 from `SimState.player_map_position` for any
//!   known map, or `nil` for an unknown map / non-player unit.
//! - `C_Map.GetBestMapForUnit(unitToken)` — returns the seeded player
//!   map id (`2248`) for `"player"`.
//! - `C_Map.GetFallbackWorldMapID()` — returns the seeded player map
//!   id (`2248`).
//! - `C_Map.MapHasArt(uiMapID)` — true for positive map ids.
//! - `C_Map.RequestPreloadMap(uiMapID)` — queues map art + overlay textures.

use super::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_capacity,
    table_get, table_set, table_set_static, val_to_string,
};
use crate::lua_api::state::MapData;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

type LuaTableRef = GcRef<Table>;
type RustLuaFn = rilua::vm::closure::RustFn;

const MAP_ART_LAYER_HASH_FIELDS: usize = 7;

const C_MAP_METHODS: &[(&str, RustLuaFn)] = &[
    (
        "GetMapArtBackgroundAtlas",
        c_map_get_map_art_background_atlas,
    ),
    ("GetMapArtID", c_map_get_map_art_id),
    ("GetMapArtLayerTextures", c_map_get_map_art_layer_textures),
    ("GetMapArtLayers", c_map_get_map_art_layers),
    ("GetMapInfo", c_map_get_map_info),
    ("GetMapInfoAtPosition", c_map_get_map_info_at_position),
    ("GetMapRectOnMap", c_map_get_map_rect_on_map),
    ("GetMapChildrenInfo", c_map_get_map_children_info),
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
    Ok(())
}

fn register_c_map_methods(state: &mut LuaState, table_ref: LuaTableRef) -> LuaResult<()> {
    for (name, rust_fn) in C_MAP_METHODS {
        table_set_rust_fn_static(state, table_ref, name, *rust_fn)?;
    }
    Ok(())
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

fn c_map_get_map_art_id(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
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

fn c_map_get_map_info(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
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
    (1..=MAX_GENERIC_UI_MAP_ID).contains(&ui_map_id)
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
    u32::try_from(ui_map_id)
        .ok()
        .and_then(crate::ui_maps::get_ui_map)
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
) -> Vec<MapData> {
    let Some(root_map) = maps.get(&root) else {
        return Vec::new();
    };

    let mut out: Vec<MapData> = Vec::new();
    let mut visited: HashSet<i32> = HashSet::new();
    let mut frontier: Vec<i32> = root_map.child_map_ids.clone();

    while let Some(child_id) = frontier.pop() {
        if !visited.insert(child_id) {
            continue;
        }
        let Some(child) = maps.get(&child_id) else {
            continue;
        };
        if map_type_filter.is_none_or(|filter| child.map_type == filter) {
            out.push(child.clone());
        }
        if all_descendants {
            frontier.extend(child.child_map_ids.iter().copied());
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

    let children = {
        let sim = borrow_state(state)?;
        if !sim.maps.contains_key(&ui_map_id) {
            return Ok(0);
        }
        collect_children(&sim.maps, ui_map_id, all_descendants, map_type_filter)
    };

    let array = create_table(state);
    for (index, child) in children.into_iter().enumerate() {
        let entry = push_map_details_table(state, &child);
        set_table_array(state, array, index as i64 + 1, entry);
    }
    state.push(array);
    Ok(1)
}

fn c_map_get_world_pos_from_map_pos(state: &mut LuaState) -> LuaResult<u32> {
    let ui_map_id = i32::from_stack(state, 1)?;
    let normalized_pos = stack_val(state, 2);
    if !borrow_state(state)?.maps.contains_key(&ui_map_id) {
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

fn map_world_size(ui_map_id: i32) -> (f64, f64) {
    let Some(map_art) = crate::map_art::get_map_art(ui_map_id as u32) else {
        return (1000.0, 1000.0);
    };
    let Some(layer) = map_art.layers.first() else {
        return (1000.0, 1000.0);
    };
    (layer.layer_width as f64, layer.layer_height as f64)
}

fn normalized_coordinate(state: &mut LuaState, position: Val, key: &str) -> f64 {
    match table_get(state, position, key) {
        Val::Num(value) => value.clamp(0.0, 1.0),
        _ => 0.5,
    }
}

fn create_world_position_vector(state: &mut LuaState, x: f64, y: f64) -> LuaResult<Val> {
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
        if !sim.maps.contains_key(&ui_map_id) || !is_player {
            None
        } else {
            Some(sim.player_map_position)
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
    let unit_token = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    if !matches!(unit_token.as_str(), "" | "player") {
        return Ok(0);
    }
    state.push(Val::Num(DEFAULT_PLAYER_MAP_ID as f64));
    Ok(1)
}

fn c_map_get_current_map_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(DEFAULT_PLAYER_MAP_ID as f64));
    Ok(1)
}

fn c_map_get_fallback_world_map_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(DEFAULT_PLAYER_MAP_ID as f64));
    Ok(1)
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
