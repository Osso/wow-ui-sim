//! rilua implementations for map, minimap, fog-of-war, and unit-position APIs.

use crate::lua_api::frame::methods::core_state;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, frame_id_from_stack, get_or_create_frame_fields, table_get,
    table_set, val_to_string,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const MINIMAP_ZOOM_KEY: &str = "__minimapZoom";
const MINIMAP_ZOOM_LEVELS: i32 = 6;

pub fn register_all(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    register_methods(state, mt, MAP_FRAME_METHODS)?;
    Ok(())
}

struct MethodBinding {
    name: &'static str,
    func: RustFn,
}

macro_rules! method {
    ($name:literal, $func:path) => {
        MethodBinding {
            name: $name,
            func: $func,
        }
    };
}

const MAP_FRAME_METHODS: &[MethodBinding] = &[
    method!("GetUiMapID", get_ui_map_id),
    method!("SetUiMapID", set_ui_map_id),
    method!(
        "GetFogOfWarBackgroundAtlas",
        get_fog_of_war_background_atlas
    ),
    method!("GetFogOfWarMaskAtlas", get_fog_of_war_mask_atlas),
    method!("GetMaskScalar", get_fog_of_war_mask_scalar),
    method!(
        "SetFogOfWarBackgroundAtlas",
        set_fog_of_war_background_atlas
    ),
    method!("SetFogOfWarMaskAtlas", set_fog_of_war_mask_atlas),
    method!("SetMaskScalar", set_fog_of_war_mask_scalar),
    method!("ClearUnits", clear_units),
    method!("AddUnit", add_unit),
    method!("FinalizeUnits", finalize_units),
    method!("SetUnitColor", set_unit_color),
    method!("GetMouseOverUnits", get_mouse_over_units),
    method!("GetPlayerPingScale", get_player_ping_scale),
    method!("SetPlayerPingTexture", set_player_ping_texture),
    method!("SetPlayerPingScale", set_player_ping_scale),
    method!("StartPlayerPing", start_player_ping),
    method!("StopPlayerPing", stop_player_ping),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetBlipTexture", set_blip_texture),
    method!("SetMaskTexture", set_minimap_mask_texture),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetIconTexture", set_minimap_icon_texture),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetPOIArrowTexture", set_poi_arrow_texture),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetCorpsePOIArrowTexture", set_corpse_poi_arrow_texture),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetStaticPOIArrowTexture", set_static_poi_arrow_texture),
    #[cfg(not(feature = "retail-12-0-7"))]
    method!("SetPlayerTexture", set_minimap_player_texture),
    method!("SetQuestBlobInsideTexture", set_quest_blob_inside_texture),
    method!("SetQuestBlobInsideAlpha", set_quest_blob_inside_alpha),
    method!("SetQuestBlobOutsideTexture", set_quest_blob_outside_texture),
    method!("SetQuestBlobOutsideAlpha", set_quest_blob_outside_alpha),
    method!("SetQuestBlobRingTexture", set_quest_blob_ring_texture),
    method!("SetQuestBlobRingAlpha", set_quest_blob_ring_alpha),
    method!("SetQuestBlobRingScalar", set_quest_blob_ring_scalar),
    method!("SetTaskBlobInsideTexture", set_task_blob_inside_texture),
    method!("SetTaskBlobInsideAlpha", set_task_blob_inside_alpha),
    method!("SetTaskBlobOutsideTexture", set_task_blob_outside_texture),
    method!("SetTaskBlobOutsideAlpha", set_task_blob_outside_alpha),
    method!("SetTaskBlobRingTexture", set_task_blob_ring_texture),
    method!("SetTaskBlobRingAlpha", set_task_blob_ring_alpha),
    method!("SetTaskBlobRingScalar", set_task_blob_ring_scalar),
    method!("SetArchBlobInsideTexture", set_arch_blob_inside_texture),
    method!("SetArchBlobInsideAlpha", set_arch_blob_inside_alpha),
    method!("SetArchBlobOutsideTexture", set_arch_blob_outside_texture),
    method!("SetArchBlobOutsideAlpha", set_arch_blob_outside_alpha),
    method!("SetArchBlobRingTexture", set_arch_blob_ring_texture),
    method!("SetArchBlobRingAlpha", set_arch_blob_ring_alpha),
    method!("SetArchBlobRingScalar", set_arch_blob_ring_scalar),
    #[cfg(feature = "retail-12-1-0")]
    method!("SetIconScale", set_icon_scale),
    #[cfg(feature = "retail-12-1-0")]
    method!("GetIconScale", get_icon_scale),
    method!("SetZoom", set_zoom),
    method!("GetZoom", get_zoom),
    method!("GetZoomLevels", get_zoom_levels),
    method!("PingLocation", ping_location),
    method!("GetPingPosition", get_ping_position),
    method!("UpdateBlips", update_blips),
    method!("SetToDefaults", set_to_defaults),
];

fn register_methods(
    state: &mut LuaState,
    mt: GcRef<Table>,
    methods: &[MethodBinding],
) -> LuaResult<()> {
    for method in methods {
        table_set_rust_fn_static(state, mt, method.name, method.func)?;
    }
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn set_icon_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scale = match stack_val(state, 2) {
        Val::Num(value) => value,
        _ => 1.0,
    };
    let fields = get_or_create_frame_fields(state, id);
    table_set(state, fields, "__minimapIconScale", Val::Num(scale));
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_icon_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    match table_get(state, fields, "__minimapIconScale") {
        Val::Num(value) => state.push(Val::Num(value)),
        _ => state.push(Val::Num(1.0)),
    }
    Ok(1)
}

fn get_ui_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let map_id = {
        let sim = borrow_state(state)?;
        if is_fog_of_war_frame(&sim, id) {
            sim.fog_of_war_frames
                .get(&id)
                .and_then(|fog| fog.ui_map_id)
                .unwrap_or(0)
        } else if is_unit_position_frame(&sim, id) {
            sim.unit_position_frames
                .get(&id)
                .and_then(|unit| unit.ui_map_id)
                .unwrap_or(0)
        } else {
            drop(sim);
            return core_state::get_ui_map_id(state);
        }
    };
    state.push(Val::Num(map_id as f64));
    Ok(1)
}

fn set_ui_map_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let map_id = stack_i32(state, 2).unwrap_or(0);
    let mut sim = borrow_state_mut(state)?;
    if is_fog_of_war_frame(&sim, id) {
        store_fog_of_war_ui_map_id(&mut sim, id, map_id);
        return Ok(0);
    }
    if is_unit_position_frame(&sim, id) {
        unit_position_state_mut(&mut sim, id).ui_map_id = Some(map_id);
        return Ok(0);
    }
    drop(sim);
    core_state::set_map_id(state)
}

fn get_fog_of_war_background_atlas(state: &mut LuaState) -> LuaResult<u32> {
    get_fog_of_war_atlas(state, |fog| fog.background_atlas.clone())
}

fn get_fog_of_war_mask_atlas(state: &mut LuaState) -> LuaResult<u32> {
    get_fog_of_war_atlas(state, |fog| fog.mask_atlas.clone())
}

fn get_fog_of_war_atlas<F>(state: &mut LuaState, read: F) -> LuaResult<u32>
where
    F: Fn(&crate::lua_api::state::FogOfWarFrameState) -> Option<String>,
{
    let id = frame_id_from_stack(state, 1)?;
    let atlas = {
        let sim = borrow_state(state)?;
        sim.fog_of_war_frames.get(&id).and_then(read)
    };
    match atlas {
        Some(value) => {
            let value = crate::lua_api::methods::create_string(state, &value);
            state.push(value);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_fog_of_war_mask_scalar(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scalar = {
        let sim = borrow_state(state)?;
        sim.fog_of_war_frames
            .get(&id)
            .and_then(|fog| fog.mask_scalar)
            .unwrap_or(1.0)
    };
    state.push(Val::Num(scalar));
    Ok(1)
}

fn set_fog_of_war_background_atlas(state: &mut LuaState) -> LuaResult<u32> {
    set_fog_of_war_atlas(
        state,
        |fog, atlas| fog.background_atlas = atlas.clone(),
        |frame, atlas| {
            frame.fog_of_war_background_atlas = atlas;
        },
    )
}

fn set_fog_of_war_mask_atlas(state: &mut LuaState) -> LuaResult<u32> {
    set_fog_of_war_atlas(
        state,
        |fog, atlas| fog.mask_atlas = atlas.clone(),
        |frame, atlas| {
            frame.fog_of_war_mask_atlas = atlas;
        },
    )
}

fn set_fog_of_war_atlas<F, G>(
    state: &mut LuaState,
    write_state: F,
    write_frame: G,
) -> LuaResult<u32>
where
    F: Fn(&mut crate::lua_api::state::FogOfWarFrameState, &Option<String>),
    G: Fn(&mut crate::widget::Frame, Option<String>),
{
    let id = frame_id_from_stack(state, 1)?;
    let atlas = val_to_string(state, stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    let fog = sim.fog_of_war_frames.entry(id).or_default();
    write_state(fog, &atlas);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        write_frame(frame, atlas);
    }
    Ok(0)
}

fn set_fog_of_war_mask_scalar(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scalar = stack_num(state, 2);
    let mut sim = borrow_state_mut(state)?;
    let fog = sim.fog_of_war_frames.entry(id).or_default();
    fog.mask_scalar = scalar;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.fog_of_war_mask_scalar = scalar.map(|value| value as f32);
    }
    Ok(0)
}

fn clear_units(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    let unit_state = unit_position_state_mut(&mut sim, id);
    unit_state.units.clear();
    unit_state.unit_colors.clear();
    unit_state.mouse_over_units.clear();
    unit_state.is_finalized = false;
    Ok(0)
}

fn add_unit(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::UnitPositionUnit;

    let id = frame_id_from_stack(state, 1)?;
    let Some(unit) = stack_string(state, 2) else {
        return Ok(0);
    };
    let asset = val_to_string(state, stack_val(state, 3));
    let width = stack_num(state, 4);
    let height = stack_num(state, 5);
    let color = stack_color(state, 6);
    let sublevel = stack_i32(state, 10);
    let show_facing = stack_bool(state, 11);

    let mut sim = borrow_state_mut(state)?;
    let unit_state = unit_position_state_mut(&mut sim, id);
    unit_state.units.push(UnitPositionUnit {
        unit,
        asset,
        width,
        height,
        color,
        sublevel,
        show_facing,
    });
    unit_state.is_finalized = false;
    Ok(0)
}

fn finalize_units(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    unit_position_state_mut(&mut sim, id).is_finalized = true;
    Ok(0)
}

fn set_unit_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(unit) = stack_string(state, 2) else {
        return Ok(0);
    };
    let Some(color) = stack_color(state, 3) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    let unit_state = unit_position_state_mut(&mut sim, id);
    unit_state.unit_colors.insert(unit.clone(), color);
    for pin in &mut unit_state.units {
        if pin.unit == unit {
            pin.color = Some(color);
        }
    }
    Ok(0)
}

fn get_mouse_over_units(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let units = {
        let sim = borrow_state(state)?;
        sim.unit_position_frames
            .get(&id)
            .map(|unit_state| unit_state.mouse_over_units.clone())
            .unwrap_or_default()
    };
    let count = units.len() as u32;
    for unit in units {
        let unit = crate::lua_api::methods::create_string(state, &unit);
        state.push(unit);
    }
    Ok(count)
}

fn get_player_ping_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scale = {
        let sim = borrow_state(state)?;
        sim.unit_position_frames
            .get(&id)
            .map(|unit_state| unit_state.player_ping_scale)
            .unwrap_or(1.0)
    };
    state.push(Val::Num(scale));
    Ok(1)
}

fn set_player_ping_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(texture_type) = stack_i32(state, 2) else {
        return Ok(0);
    };
    let asset = val_to_string(state, stack_val(state, 3));
    let width = stack_num(state, 4).unwrap_or(0.0);
    let height = stack_num(state, 5).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    unit_position_state_mut(&mut sim, id)
        .player_ping_textures
        .insert(
            texture_type,
            crate::lua_api::state::UnitPositionPlayerPingTexture {
                asset,
                width,
                height,
            },
        );
    Ok(0)
}

fn set_player_ping_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scale = stack_num(state, 2).unwrap_or(1.0);
    let mut sim = borrow_state_mut(state)?;
    unit_position_state_mut(&mut sim, id).player_ping_scale = scale;
    Ok(0)
}

fn start_player_ping(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let duration = stack_num(state, 2).unwrap_or(0.0);
    let fade_duration = stack_num(state, 3).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    let unit_state = unit_position_state_mut(&mut sim, id);
    unit_state.player_ping_active = true;
    unit_state.player_ping_duration = Some(duration);
    unit_state.player_ping_fade_duration = Some(fade_duration);
    Ok(0)
}

fn stop_player_ping(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    unit_position_state_mut(&mut sim, id).player_ping_active = false;
    Ok(0)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_blip_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| frame.minimap_blip_texture = texture)
}

fn set_minimap_mask_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| frame.minimap_mask_texture = texture)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_minimap_icon_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| frame.minimap_icon_texture = texture)
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_minimap_player_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| {
        frame.minimap_player_texture = texture
    })
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_poi_arrow_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| {
        frame.minimap_poi_arrow_texture = texture
    })
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_corpse_poi_arrow_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| {
        frame.minimap_corpse_poi_arrow_texture = texture
    })
}

#[cfg(not(feature = "retail-12-0-7"))]
fn set_static_poi_arrow_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_minimap_texture_field(state, |frame, texture| {
        frame.minimap_static_poi_arrow_texture = texture
    })
}

fn set_quest_blob_inside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.quest_blob_inside.texture = texture
    })
}

fn set_quest_blob_inside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.quest_blob_inside.alpha = alpha)
}

fn set_quest_blob_outside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.quest_blob_outside.texture = texture
    })
}

fn set_quest_blob_outside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.quest_blob_outside.alpha = alpha)
}

fn set_quest_blob_ring_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.quest_blob_ring.texture = texture
    })
}

fn set_quest_blob_ring_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.quest_blob_ring.alpha = alpha)
}

fn set_quest_blob_ring_scalar(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_scalar(state, |frame, scalar| frame.quest_blob_ring.scalar = scalar)
}

fn set_task_blob_inside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.task_blob_inside.texture = texture
    })
}

fn set_task_blob_inside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.task_blob_inside.alpha = alpha)
}

fn set_task_blob_outside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.task_blob_outside.texture = texture
    })
}

fn set_task_blob_outside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.task_blob_outside.alpha = alpha)
}

fn set_task_blob_ring_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.task_blob_ring.texture = texture
    })
}

fn set_task_blob_ring_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.task_blob_ring.alpha = alpha)
}

fn set_task_blob_ring_scalar(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_scalar(state, |frame, scalar| frame.task_blob_ring.scalar = scalar)
}

fn set_arch_blob_inside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.arch_blob_inside.texture = texture
    })
}

fn set_arch_blob_inside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.arch_blob_inside.alpha = alpha)
}

fn set_arch_blob_outside_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.arch_blob_outside.texture = texture
    })
}

fn set_arch_blob_outside_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.arch_blob_outside.alpha = alpha)
}

fn set_arch_blob_ring_texture(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_texture(state, |frame, texture| {
        frame.arch_blob_ring.texture = texture
    })
}

fn set_arch_blob_ring_alpha(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_alpha(state, |frame, alpha| frame.arch_blob_ring.alpha = alpha)
}

fn set_arch_blob_ring_scalar(state: &mut LuaState) -> LuaResult<u32> {
    set_blob_scalar(state, |frame, scalar| frame.arch_blob_ring.scalar = scalar)
}

fn set_blob_texture<F>(state: &mut LuaState, write: F) -> LuaResult<u32>
where
    F: Fn(&mut crate::widget::Frame, Option<String>),
{
    set_frame_texture_field(state, write)
}

fn set_blob_alpha<F>(state: &mut LuaState, write: F) -> LuaResult<u32>
where
    F: Fn(&mut crate::widget::Frame, f64),
{
    let id = frame_id_from_stack(state, 1)?;
    let alpha = stack_num(state, 2).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        write(frame, alpha);
    }
    Ok(0)
}

fn set_blob_scalar<F>(state: &mut LuaState, write: F) -> LuaResult<u32>
where
    F: Fn(&mut crate::widget::Frame, f64),
{
    let id = frame_id_from_stack(state, 1)?;
    let scalar = stack_num(state, 2).unwrap_or(0.0);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        write(frame, scalar);
    }
    Ok(0)
}

fn set_minimap_texture_field<F>(state: &mut LuaState, write: F) -> LuaResult<u32>
where
    F: Fn(&mut crate::widget::Frame, Option<String>),
{
    set_frame_texture_field(state, write)
}

fn set_frame_texture_field<F>(state: &mut LuaState, write: F) -> LuaResult<u32>
where
    F: Fn(&mut crate::widget::Frame, Option<String>),
{
    let id = frame_id_from_stack(state, 1)?;
    let texture = val_to_string(state, stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        write(frame, texture);
    }
    Ok(0)
}

fn set_zoom(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let requested = stack_i32(state, 2).unwrap_or(0);
    let clamped = requested.clamp(0, MINIMAP_ZOOM_LEVELS - 1);
    let fields = get_or_create_frame_fields(state, id);
    table_set(state, fields, MINIMAP_ZOOM_KEY, Val::Num(clamped as f64));
    Ok(0)
}

fn get_zoom(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    let zoom = match table_get(state, fields, MINIMAP_ZOOM_KEY) {
        Val::Num(value) => value as i32,
        _ => 0,
    };
    state.push(Val::Num(zoom as f64));
    Ok(1)
}

fn get_zoom_levels(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(MINIMAP_ZOOM_LEVELS as f64));
    Ok(1)
}

fn ping_location(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = stack_num(state, 2).unwrap_or(0.0) as f32;
    let y = stack_num(state, 3).unwrap_or(0.0) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.minimap_ping_position = Some((x, y));
    }
    Ok(0)
}

fn get_ping_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let (x, y) = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|frame| frame.minimap_ping_position)
            .unwrap_or((0.0, 0.0))
    };
    state.push(Val::Num(x as f64));
    state.push(Val::Num(y as f64));
    Ok(2)
}

fn update_blips(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.minimap_blip_update_revision = frame.minimap_blip_update_revision.saturating_add(1);
    }
    Ok(0)
}

fn set_to_defaults(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    {
        // Real WoW's FrameCompositor:SetToDefaults clears anchors and resets size
        // to 0,0 (see BlizzardUI Compositor.lua). The menu element pool relies on
        // this — without it, MeasureFrameExtents reads a stale GetSize() from a
        // previously reused frame and inflates the menu width.
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.remove_all_anchor_dependents_for(id);
        if let Some(frame) = sim.widgets.get_mut_visual(id) {
            frame.clear_all_points();
            frame.set_size(0.0, 0.0);
            frame.width_is_text_auto = false;
            frame.layout_rect = None;
            frame.minimap_blip_texture = None;
            frame.minimap_mask_texture = None;
            frame.minimap_icon_texture = None;
            frame.minimap_player_texture = None;
            frame.minimap_poi_arrow_texture = None;
            frame.minimap_corpse_poi_arrow_texture = None;
            frame.minimap_static_poi_arrow_texture = None;
            frame.minimap_ping_position = None;
            frame.minimap_blip_update_revision = 0;
            frame.quest_blob_inside = Default::default();
            frame.quest_blob_outside = Default::default();
            frame.quest_blob_ring = Default::default();
            frame.task_blob_inside = Default::default();
            frame.task_blob_outside = Default::default();
            frame.task_blob_ring = Default::default();
            frame.arch_blob_inside = Default::default();
            frame.arch_blob_outside = Default::default();
            frame.arch_blob_ring = Default::default();
        }
        sim.widgets.mark_rect_dirty(id);
    }
    let fields = get_or_create_frame_fields(state, id);
    table_set(state, fields, MINIMAP_ZOOM_KEY, Val::Num(0.0));
    Ok(0)
}

fn is_fog_of_war_frame(sim: &crate::lua_api::SimState, id: u64) -> bool {
    sim.widgets
        .get(id)
        .and_then(|frame| frame.object_type_name.as_deref())
        .is_some_and(|name| name.eq_ignore_ascii_case("FogOfWarFrame"))
}

fn is_unit_position_frame(sim: &crate::lua_api::SimState, id: u64) -> bool {
    sim.widgets
        .get(id)
        .and_then(|frame| frame.object_type_name.as_deref())
        .is_some_and(|name| name.eq_ignore_ascii_case("UnitPositionFrame"))
}

fn store_fog_of_war_ui_map_id(sim: &mut crate::lua_api::SimState, id: u64, map_id: i32) {
    let fog_state = sim.fog_of_war_frames.entry(id).or_default();
    fog_state.ui_map_id = Some(map_id);
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.fog_of_war_ui_map_id = Some(map_id);
    }
}

fn unit_position_state_mut(
    sim: &mut crate::lua_api::SimState,
    id: u64,
) -> &mut crate::lua_api::state::UnitPositionFrameState {
    sim.unit_position_frames.entry(id).or_insert_with(|| {
        crate::lua_api::state::UnitPositionFrameState {
            ui_map_id: None,
            units: Vec::new(),
            unit_colors: std::collections::HashMap::new(),
            mouse_over_units: Vec::new(),
            player_ping_scale: 1.0,
            player_ping_textures: std::collections::HashMap::new(),
            player_ping_active: false,
            player_ping_duration: None,
            player_ping_fade_duration: None,
            is_finalized: false,
        }
    })
}

fn stack_string(state: &LuaState, index: i32) -> Option<String> {
    val_to_string(state, stack_val(state, index))
}

fn stack_num(state: &LuaState, index: i32) -> Option<f64> {
    match stack_val(state, index) {
        Val::Num(value) => Some(value),
        _ => None,
    }
}

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    stack_num(state, index).map(|value| value as i32)
}

fn stack_bool(state: &LuaState, index: i32) -> Option<bool> {
    match stack_val(state, index) {
        Val::Bool(value) => Some(value),
        _ => None,
    }
}

fn stack_color(state: &LuaState, start_index: i32) -> Option<(f64, f64, f64, f64)> {
    Some((
        stack_num(state, start_index)?,
        stack_num(state, start_index + 1)?,
        stack_num(state, start_index + 2)?,
        stack_num(state, start_index + 3)?,
    ))
}
