//! Model and ModelScene widget methods (stubs + partial impl).

mod model_scene;
mod model_scene_actors;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod model_unit;
#[cfg(feature = "retail-12-0-7")]
mod model_unit_guid;

use super::shared::{opt_bool, val_to_f64};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, frame_id_from_stack};
use crate::lua_bridge::{IntoStack, stack_val, table_set_rust_fn};
use model_scene::*;
use model_scene_actors::*;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn set_model(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = super::shared::opt_string(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let model = frame.model_state_mut();
        model.model_path = path;
        model.model_file_id = None;
    }
    Ok(0)
}

pub(super) fn get_model(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = {
        let sim = borrow_state(state)?;
        sim.widgets
            .get(id)
            .and_then(|f| f.model_state().model_path.clone())
            .unwrap_or_default()
    };
    let val = create_string(state, &path);
    val.into_stack(state)
}

pub(super) fn set_model_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scale = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.update_model_state(scale != 1.0, |model| model.model_transform.scale = scale);
    }
    Ok(0)
}

pub(super) fn get_model_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_transform.scale as f64)
        .unwrap_or(1.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = val_to_f64(stack_val(state, 2)) as f32;
    let y = val_to_f64(stack_val(state, 3)) as f32;
    let z = val_to_f64(stack_val(state, 4)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        let position = (x, y, z);
        f.update_model_state(position != (0.0, 0.0, 0.0), |model| {
            model.model_transform.position = position;
        });
    }
    Ok(0)
}

pub(super) fn get_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (x, y, z) = sim
        .widgets
        .get(id)
        .map(|f| {
            let p = f.model_state().model_transform.position;
            (p.0 as f64, p.1 as f64, p.2 as f64)
        })
        .unwrap_or((0.0, 0.0, 0.0));
    drop(sim);
    (x, y, z).into_stack(state)
}

pub(super) fn set_facing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let rad = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.update_model_state(rad != 0.0, |model| model.model_transform.facing = rad);
    }
    Ok(0)
}

pub(super) fn get_facing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_transform.facing as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_rotation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let rad = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.rotation = rad;
    }
    Ok(0)
}

pub(super) fn set_animation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let anim_id = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_appearance.animation_id = Some(anim_id);
    }
    Ok(0)
}

pub(super) fn set_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let display_id = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let model = frame.model_state_mut();
        model.model_path = None;
        model.model_file_id = None;
        model.model_appearance.display_info = Some(display_id);
        model.model_appearance.creature_id = None;
    }
    Ok(0)
}

/// Records `displayID` on the actor handle, mirroring
/// `Actor:SetModelByCreatureDisplayID` from
/// `vendor/wow-ui-source/Interface/AddOns/Blizzard_SharedXML/ModelSceneActorMixin.lua`.
/// The optional `useCachedModelIfAvailable` flag is stored verbatim — the
/// simulator's 3D path is intentionally stubbed, so the flag never drives
/// a real renderer; tests still see the value the addon passed.
pub(super) fn set_model_by_creature_display_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let display_id = val_to_f64(stack_val(state, 2)) as i32;
    let use_cached = opt_bool(state, 3).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let model = frame.model_state_mut();
        model.model_path = None;
        model.model_file_id = None;
        model.model_appearance.display_info = Some(display_id);
        model.model_appearance.creature_id = None;
        model.model_appearance.use_cached_model = use_cached;
    }
    Ok(0)
}

pub(super) fn set_creature(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let creature_id = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let model = frame.model_state_mut();
        model.model_path = None;
        model.model_file_id = None;
        model.model_appearance.display_info = None;
        model.model_appearance.creature_id = Some(creature_id);
    }
    Ok(0)
}

pub(super) fn clear_model(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(model) = sim
        .widgets
        .get_mut_visual(id)
        .and_then(|frame| frame.existing_model_state_mut())
    {
        model.model_path = None;
        model.model_file_id = None;
        model.model_appearance.display_info = None;
        model.model_appearance.creature_id = None;
        model.model_appearance.animation_id = None;
        model.model_appearance.sequence_id = None;
        model.model_appearance.sequence_time_ms = None;
    }
    Ok(0)
}

pub(super) fn get_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.model_state().model_appearance.display_info)
        .unwrap_or(0);
    (value as f64).into_stack(state)
}

pub(super) fn get_model_file_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .and_then(|f| f.model_state().model_file_id)
        .unwrap_or(0);
    drop(sim);
    (v as f64).into_stack(state)
}

pub(super) fn set_model_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let alpha = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.update_model_state(alpha != 1.0, |model| model.model_rendering.alpha = alpha);
    }
    Ok(0)
}

pub(super) fn set_do_blend(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.update_model_state(enabled, |model| {
            model.player_model_state.do_blend = enabled;
        });
    }
    Ok(0)
}

pub(super) fn apply_spell_visual_kit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let anim_kit = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().player_model_state.active_anim_kit = Some(anim_kit);
    }
    Ok(0)
}

pub(super) fn get_do_blend(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().player_model_state.do_blend)
        .unwrap_or(false);
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(super) fn set_keep_model_on_hide(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let keep = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.update_model_state(keep, |model| {
            model.player_model_state.keep_model_on_hide = keep;
        });
    }
    Ok(0)
}

pub(super) fn get_keep_model_on_hide(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let keep = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().player_model_state.keep_model_on_hide)
        .unwrap_or(false);
    state.push(Val::Bool(keep));
    Ok(1)
}

pub(super) fn set_item(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let item = stringish_arg(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().player_model_state.last_item = item;
    }
    Ok(0)
}

pub(super) fn set_item_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let appearance = stringish_arg(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().player_model_state.last_item_appearance = appearance;
    }
    Ok(0)
}

pub(super) fn play_anim_kit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let anim_kit = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().player_model_state.active_anim_kit = Some(anim_kit);
    }
    Ok(0)
}

pub(super) fn stop_anim_kit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().player_model_state.active_anim_kit = None;
    }
    Ok(0)
}

pub(super) fn can_set_unit(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

/// Model side of the shared `SetUnit` name; dispatched by `widgets::set_unit`.
pub(super) fn set_model_unit(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    {
        model_unit::assign_unit(state)
    }
    #[cfg(not(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )))]
    {
        SKIP_3D_RENDERING(state)
    }
}

pub(super) fn set_model_by_unit(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    {
        return model_unit::assign_unit(state);
    }
    #[cfg(not(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )))]
    {
        let id = frame_id_from_stack(state, 1)?;
        let unit = stringish_arg(state, 2);
        let mut sim = borrow_state_mut(state)?;
        if let Some(f) = sim.widgets.get_mut_visual(id) {
            f.model_state_mut().player_model_state.last_unit = unit;
        }
        drop(sim);
        state.push(Val::Bool(true));
        Ok(1)
    }
}

fn stringish_arg(state: &LuaState, index: i32) -> Option<String> {
    match stack_val(state, index) {
        Val::Str(str_ref) => {
            let lua_str = state.gc.string_arena.get(str_ref)?;
            String::from_utf8(lua_str.data().to_vec()).ok()
        }
        Val::Num(n) if n.fract() == 0.0 => Some((n as i64).to_string()),
        Val::Num(n) => Some(n.to_string()),
        _ => None,
    }
}

pub(super) fn get_model_alpha(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_rendering.alpha as f64)
        .unwrap_or(1.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_shadow_effect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let effect = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_rendering.shadow_effect = effect;
    }
    Ok(0)
}

pub(super) fn get_shadow_effect(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_rendering.shadow_effect as f64)
        .unwrap_or(0.0);
    v.into_stack(state)
}

pub(super) fn set_particles_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_rendering.particles_enabled = enabled;
    }
    Ok(0)
}

pub(super) fn set_use_gbuffer(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_rendering.use_gbuffer = enabled;
    }
    Ok(0)
}

pub(super) fn set_sequence(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let seq = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let appearance = &mut frame.model_state_mut().model_appearance;
        appearance.sequence_id = Some(seq);
        appearance.sequence_time_ms = None;
    }
    Ok(0)
}

pub(super) fn set_sequence_time(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let seq = val_to_f64(stack_val(state, 2)) as i32;
    let time = val_to_f64(stack_val(state, 3)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        let appearance = &mut frame.model_state_mut().model_appearance;
        appearance.sequence_id = Some(seq);
        appearance.sequence_time_ms = Some(time);
    }
    Ok(0)
}

pub(super) fn has_animation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let has_animation = borrow_state(state)?
        .widgets
        .get(id)
        .and_then(|frame| frame.model_state().model_appearance.animation_id)
        .is_some();
    state.push(Val::Bool(has_animation));
    Ok(1)
}

pub(super) fn refresh_unit(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_appearance.refresh_unit_count += 1;
    }
    Ok(0)
}

pub(super) fn refresh_camera(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_appearance.refresh_camera_count += 1;
    }
    Ok(0)
}

pub(super) fn get_camera_distance(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_transform.camera.distance as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_camera_distance(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let dist = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_transform.camera.distance = dist;
    }
    Ok(0)
}

pub(super) fn get_camera_facing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_transform.camera.facing as f64)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_camera_facing(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let rad = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_transform.camera.facing = rad;
    }
    Ok(0)
}

pub(super) fn get_camera_target(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let t = sim
        .widgets
        .get(id)
        .map(|f| f.model_state().model_transform.camera.target)
        .unwrap_or((0.0, 0.0, 0.0));
    drop(sim);
    (t.0 as f64, t.1 as f64, t.2 as f64).into_stack(state)
}

pub(super) fn set_camera_target(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = val_to_f64(stack_val(state, 2)) as f32;
    let y = val_to_f64(stack_val(state, 3)) as f32;
    let z = val_to_f64(stack_val(state, 4)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_transform.camera.target = (x, y, z);
    }
    Ok(0)
}

pub(super) fn get_camera_roll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let v = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_transform.camera.roll as f64)
        .unwrap_or(0.0);
    v.into_stack(state)
}

pub(super) fn set_camera_roll(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let roll = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.model_state_mut().model_transform.camera.roll = roll;
    }
    Ok(0)
}

// Stubs

pub(super) fn stub_variadic(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

pub(super) fn stub_nil(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn transform_camera_space_to_model_space(state: &mut LuaState) -> LuaResult<u32> {
    state.push(stack_val(state, 2));
    Ok(1)
}

pub(super) fn stub_zero(state: &mut LuaState) -> LuaResult<u32> {
    0.0_f64.into_stack(state)
}

pub(super) fn stub_one(state: &mut LuaState) -> LuaResult<u32> {
    1.0_f64.into_stack(state)
}

pub(super) fn stub_false(state: &mut LuaState) -> LuaResult<u32> {
    false.into_stack(state)
}

pub(super) fn stub_true(state: &mut LuaState) -> LuaResult<u32> {
    true.into_stack(state)
}

const SKIP_3D_RENDERING: rilua::vm::closure::RustFn = stub_variadic;

// ---------------------------------------------------------------------------
// register_model
// ---------------------------------------------------------------------------

const MODEL_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Model source + transform
    ("SetModel", set_model),
    ("GetModel", get_model),
    ("SetModelScale", set_model_scale),
    ("GetModelScale", get_model_scale),
    ("SetPosition", set_position),
    ("GetPosition", get_position),
    ("SetFacing", set_facing),
    ("GetFacing", get_facing),
    ("SetRotation", set_rotation),
    // Animation / display info
    ("SetAnimation", set_animation),
    ("ApplySpellVisualKit", apply_spell_visual_kit),
    ("SetDisplayInfo", set_display_info),
    ("GetDisplayInfo", get_display_info),
    (
        "SetModelByCreatureDisplayID",
        set_model_by_creature_display_id,
    ),
    ("SetCreature", set_creature),
    ("ClearModel", clear_model),
    ("GetModelFileID", get_model_file_id),
    #[cfg(feature = "retail-12-0-7")]
    ("GetModelUnitGUID", model_unit_guid::get_model_unit_guid),
    ("SetModelAlpha", set_model_alpha),
    ("GetModelAlpha", get_model_alpha),
    ("SetShadowEffect", set_shadow_effect),
    ("GetShadowEffect", get_shadow_effect),
    ("SetParticlesEnabled", set_particles_enabled),
    ("SetUseGBuffer", set_use_gbuffer),
    ("SetDoBlend", set_do_blend),
    ("GetDoBlend", get_do_blend),
    ("SetKeepModelOnHide", set_keep_model_on_hide),
    ("GetKeepModelOnHide", get_keep_model_on_hide),
    ("SetItem", set_item),
    ("SetItemAppearance", set_item_appearance),
    ("SetModelByUnit", set_model_by_unit),
    ("PlayAnimKit", play_anim_kit),
    ("StopAnimKit", stop_anim_kit),
    ("CanSetUnit", can_set_unit),
    ("HasAnimation", has_animation),
    ("SetSequence", set_sequence),
    ("SetSequenceTime", set_sequence_time),
    // Camera
    ("GetCameraDistance", get_camera_distance),
    ("SetCameraDistance", set_camera_distance),
    ("GetCameraFacing", get_camera_facing),
    ("SetCameraFacing", set_camera_facing),
    ("GetCameraTarget", get_camera_target),
    ("SetCameraTarget", set_camera_target),
    ("GetCameraRoll", get_camera_roll),
    ("SetCameraRoll", set_camera_roll),
    // 3D rendering is intentionally out of scope. These absorb visual-only
    // model/camera calls while preserving the Lua API surface.
    ("SetAutoDress", SKIP_3D_RENDERING),
    ("SetCamDistanceScale", SKIP_3D_RENDERING),
    ("SetCamera", SKIP_3D_RENDERING),
    ("SetPortraitZoom", SKIP_3D_RENDERING),
    ("SetLight", SKIP_3D_RENDERING),
    ("ResetLights", SKIP_3D_RENDERING),
    ("ClearFog", SKIP_3D_RENDERING),
    ("RefreshUnit", refresh_unit),
    ("RefreshCamera", refresh_camera),
    (
        "TransitionToModelSceneID",
        scene_transition_to_model_scene_id,
    ),
    ("SetFromModelSceneID", stub_variadic),
    ("CycleVariation", SKIP_3D_RENDERING),
    ("AdvanceTime", SKIP_3D_RENDERING),
    ("ClearTransform", SKIP_3D_RENDERING),
    ("SetTransform", SKIP_3D_RENDERING),
    ("SetPitch", SKIP_3D_RENDERING),
    ("SetRoll", SKIP_3D_RENDERING),
    ("UseModelCenterToTransform", SKIP_3D_RENDERING),
    ("SetViewTranslation", SKIP_3D_RENDERING),
    ("SetModelDrawLayer", SKIP_3D_RENDERING),
    ("ReplaceIconTexture", SKIP_3D_RENDERING),
    ("SetGlow", SKIP_3D_RENDERING),
    ("SetGradientMask", SKIP_3D_RENDERING),
    #[cfg(feature = "retail-12-0-5")]
    ("SetGradientMaskWithDyes", SKIP_3D_RENDERING),
    #[cfg(feature = "retail-12-0-5")]
    ("SetSheathedCategory", SKIP_3D_RENDERING),
    #[cfg(feature = "retail-12-0-5")]
    ("UseUnitSheatheCategory", SKIP_3D_RENDERING),
    ("SetCustomCamera", SKIP_3D_RENDERING),
    ("MakeCurrentCameraCustom", SKIP_3D_RENDERING),
    // ModelSceneActorMixin lifecycle methods. The simulator does not render
    // 3D models, but Blizzard's ModelScene pools call these on acquired actors.
    ("ApplyFromModelSceneActorInfo", stub_variadic),
    ("OnReleased", stub_variadic),
    // DressUpModel / transmog wardrobe — no real 3D, just absorb the calls
    ("SetUseTransmogSkin", SKIP_3D_RENDERING),
    ("SetUseTransmogChoices", SKIP_3D_RENDERING),
    ("SetObeyHideInTransmogFlag", SKIP_3D_RENDERING),
    ("TryOn", SKIP_3D_RENDERING),
    ("UndressSlot", SKIP_3D_RENDERING),
    ("Undress", SKIP_3D_RENDERING),
    ("UpdateCamera", SKIP_3D_RENDERING),
    ("FreezeAnimation", SKIP_3D_RENDERING),
    // Typed return stubs
    ("GetModelSceneID", stub_zero),
    ("GetCamDistanceScale", stub_one),
    ("HasCustomCamera", stub_false),
    ("HasAttachmentPoints", stub_false),
    ("GetLight", stub_nil),
    ("GetPitch", stub_zero),
    ("GetRoll", stub_zero),
    ("GetWorldScale", stub_one),
    (
        "TransformCameraSpaceToModelSpace",
        transform_camera_space_to_model_space,
    ),
    ("IsUsingModelCenterToTransform", stub_false),
    ("GetUpperEmblemTexture", stub_nil),
    ("GetLowerEmblemTexture", stub_nil),
    // Wardrobe gates appearance enumeration on these — must be true so
    // the items list populates and the geometry-ready code path runs.
    ("IsSlotAllowed", stub_true),
    ("IsGeoReady", stub_true),
    ("HasTrackableSource", stub_false),
    // ModelScene-specific (round-tripped state)
    ("SetCameraPosition", scene_set_camera_position),
    ("GetCameraPosition", scene_get_camera_position),
    (
        "SetCameraOrientationByAxisVectors",
        scene_set_camera_orientation_by_axis_vectors,
    ),
    ("GetCameraForward", scene_get_camera_forward),
    ("GetCameraRight", scene_get_camera_right),
    ("GetCameraUp", scene_get_camera_up),
    ("SetCameraFieldOfView", scene_set_camera_field_of_view),
    ("GetCameraFieldOfView", scene_get_camera_field_of_view),
    ("SetCameraNearClip", scene_set_camera_near_clip),
    ("GetCameraNearClip", scene_get_camera_near_clip),
    ("SetCameraFarClip", scene_set_camera_far_clip),
    ("GetCameraFarClip", scene_get_camera_far_clip),
    ("SetLightType", scene_set_light_type),
    ("GetLightType", scene_get_light_type),
    ("SetLightPosition", scene_set_light_position),
    ("GetLightPosition", scene_get_light_position),
    ("SetLightDirection", scene_set_light_direction),
    ("GetLightDirection", scene_get_light_direction),
    ("SetLightAmbientColor", scene_set_light_ambient_color),
    ("GetLightAmbientColor", scene_get_light_ambient_color),
    ("SetLightDiffuseColor", scene_set_light_diffuse_color),
    ("GetLightDiffuseColor", scene_get_light_diffuse_color),
    ("SetLightVisible", scene_set_light_visible),
    ("IsLightVisible", scene_is_light_visible),
    ("SetFogNear", scene_set_fog_near),
    ("GetFogNear", scene_get_fog_near),
    ("SetFogFar", scene_set_fog_far),
    ("GetFogFar", scene_get_fog_far),
    ("SetFogColor", scene_set_fog_color),
    ("GetFogColor", scene_get_fog_color),
    ("SetPaused", scene_set_paused),
    ("GetPaused", scene_get_paused),
    (
        "SetAllowOverlappedModels",
        scene_set_allow_overlapped_models,
    ),
    ("IsAllowOverlappedModels", scene_is_allow_overlapped_models),
    ("SetViewInsets", scene_set_view_insets),
    ("GetViewInsets", scene_get_view_insets),
    ("SetViewTranslation", scene_set_view_translation),
    ("Project3DPointTo2D", scene_project_3d_point_to_2d),
    ("CreateActor", scene_create_actor),
    ("GetNumActors", scene_get_num_actors),
    ("GetActorAtIndex", scene_get_actor_at_index),
    ("GetActorByTag", scene_get_actor_by_tag),
    ("GetPlayerActor", scene_get_player_actor),
    ("TakeActor", scene_take_actor),
    ("SetResetCallback", scene_set_reset_callback),
    ("ClearScene", scene_clear_scene),
];

pub(super) fn register_model(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in MODEL_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    #[cfg(feature = "retail-12-0-0")]
    {
        // Permanent: collision-bound preference has no effect in this 2D renderer.
        table_set_rust_fn(
            state,
            metatable,
            "SetPreferModelCollisionBounds",
            SKIP_3D_RENDERING,
        )?;
        table_set_rust_fn(
            state,
            metatable,
            "IsPreferringModelCollisionBounds",
            stub_false,
        )?;
    }
    Ok(())
}
