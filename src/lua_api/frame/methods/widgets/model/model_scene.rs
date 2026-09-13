use super::super::shared::{opt_bool, val_to_f64};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{IntoStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ModelScene

pub(super) fn scene_set_allow_overlapped_models(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let allow = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame
            .model_state_mut()
            .model_scene_state
            .allow_overlapped_models = allow;
    }
    Ok(0)
}

pub(super) fn scene_set_view_translation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = val_to_f64(stack_val(state, 2)) as f32;
    let y = val_to_f64(stack_val(state, 3)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.view_translation = (x, y);
    }
    Ok(0)
}

pub(super) fn scene_set_camera_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let x = val_to_f64(stack_val(state, 2)) as f32;
    let y = val_to_f64(stack_val(state, 3)) as f32;
    let z = val_to_f64(stack_val(state, 4)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.camera.position = (x, y, z);
    }
    Ok(0)
}

pub(super) fn scene_get_camera_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let pos = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.position)
        .unwrap_or((0.0, 0.0, 0.0));
    (pos.0 as f64, pos.1 as f64, pos.2 as f64).into_stack(state)
}

pub(super) fn scene_set_camera_orientation_by_axis_vectors(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let forward = (
        val_to_f64(stack_val(state, 2)) as f32,
        val_to_f64(stack_val(state, 3)) as f32,
        val_to_f64(stack_val(state, 4)) as f32,
    );
    let right = (
        val_to_f64(stack_val(state, 5)) as f32,
        val_to_f64(stack_val(state, 6)) as f32,
        val_to_f64(stack_val(state, 7)) as f32,
    );
    let up = (
        val_to_f64(stack_val(state, 8)) as f32,
        val_to_f64(stack_val(state, 9)) as f32,
        val_to_f64(stack_val(state, 10)) as f32,
    );
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.camera.forward = forward;
        frame.model_state_mut().model_scene_state.camera.right = right;
        frame.model_state_mut().model_scene_state.camera.up = up;
    }
    Ok(0)
}

pub(super) fn scene_get_camera_forward(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.forward)
        .unwrap_or((0.0, 0.0, 1.0));
    (value.0 as f64, value.1 as f64, value.2 as f64).into_stack(state)
}

pub(super) fn scene_get_camera_right(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.right)
        .unwrap_or((1.0, 0.0, 0.0));
    (value.0 as f64, value.1 as f64, value.2 as f64).into_stack(state)
}

pub(super) fn scene_get_camera_up(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.up)
        .unwrap_or((0.0, 1.0, 0.0));
    (value.0 as f64, value.1 as f64, value.2 as f64).into_stack(state)
}

pub(super) fn scene_set_camera_field_of_view(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame
            .model_state_mut()
            .model_scene_state
            .camera
            .field_of_view = value;
    }
    Ok(0)
}

pub(super) fn scene_get_camera_field_of_view(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.field_of_view as f64)
        .unwrap_or(0.785);
    value.into_stack(state)
}

pub(super) fn scene_set_camera_near_clip(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.camera.near_clip = value;
    }
    Ok(0)
}

pub(super) fn scene_get_camera_near_clip(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.near_clip as f64)
        .unwrap_or(1.0);
    value.into_stack(state)
}

pub(super) fn scene_set_camera_far_clip(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.camera.far_clip = value;
    }
    Ok(0)
}

pub(super) fn scene_get_camera_far_clip(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.camera.far_clip as f64)
        .unwrap_or(100.0);
    value.into_stack(state)
}

pub(super) fn scene_set_light_type(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as i32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.light.light_type = value;
    }
    Ok(0)
}

pub(super) fn scene_get_light_type(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.light_type as f64)
        .unwrap_or(0.0);
    value.into_stack(state)
}

pub(super) fn scene_set_light_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = (
        val_to_f64(stack_val(state, 2)) as f32,
        val_to_f64(stack_val(state, 3)) as f32,
        val_to_f64(stack_val(state, 4)) as f32,
    );
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.light.position = value;
    }
    Ok(0)
}

pub(super) fn scene_get_light_position(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.position)
        .unwrap_or((0.0, 0.0, 0.0));
    (value.0 as f64, value.1 as f64, value.2 as f64).into_stack(state)
}

pub(super) fn scene_set_light_direction(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = (
        val_to_f64(stack_val(state, 2)) as f32,
        val_to_f64(stack_val(state, 3)) as f32,
        val_to_f64(stack_val(state, 4)) as f32,
    );
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.light.direction = value;
    }
    Ok(0)
}

pub(super) fn scene_get_light_direction(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.direction)
        .unwrap_or((0.0, -1.0, 0.0));
    (value.0 as f64, value.1 as f64, value.2 as f64).into_stack(state)
}

fn color_arg(state: &LuaState, start: i32) -> crate::widget::Color {
    crate::widget::Color::rgb(
        val_to_f64(stack_val(state, start)) as f32,
        val_to_f64(stack_val(state, start + 1)) as f32,
        val_to_f64(stack_val(state, start + 2)) as f32,
    )
}

pub(super) fn scene_set_light_ambient_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = color_arg(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame
            .model_state_mut()
            .model_scene_state
            .light
            .ambient_color = value;
    }
    Ok(0)
}

pub(super) fn scene_get_light_ambient_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.ambient_color)
        .unwrap_or(crate::widget::Color::rgb(1.0, 1.0, 1.0));
    (value.r as f64, value.g as f64, value.b as f64).into_stack(state)
}

pub(super) fn scene_set_light_diffuse_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = color_arg(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame
            .model_state_mut()
            .model_scene_state
            .light
            .diffuse_color = value;
    }
    Ok(0)
}

pub(super) fn scene_get_light_diffuse_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.diffuse_color)
        .unwrap_or(crate::widget::Color::rgb(1.0, 1.0, 1.0));
    (value.r as f64, value.g as f64, value.b as f64).into_stack(state)
}

pub(super) fn scene_set_light_visible(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.light.visible = value;
    }
    Ok(0)
}

pub(super) fn scene_is_light_visible(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.light.visible)
        .unwrap_or(true);
    state.push(Val::Bool(value));
    Ok(1)
}

pub(super) fn scene_set_fog_near(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.fog.near = value;
    }
    Ok(0)
}

pub(super) fn scene_get_fog_near(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.fog.near as f64)
        .unwrap_or(0.0);
    value.into_stack(state)
}

pub(super) fn scene_set_fog_far(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = val_to_f64(stack_val(state, 2)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.fog.far = value;
    }
    Ok(0)
}

pub(super) fn scene_get_fog_far(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.fog.far as f64)
        .unwrap_or(0.0);
    value.into_stack(state)
}

pub(super) fn scene_set_fog_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = color_arg(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.model_state_mut().model_scene_state.fog.color = value;
    }
    Ok(0)
}

pub(super) fn scene_get_fog_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let value = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.fog.color)
        .unwrap_or(crate::widget::Color::rgb(0.0, 0.0, 0.0));
    (value.r as f64, value.g as f64, value.b as f64).into_stack(state)
}

pub(super) fn scene_set_paused(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let paused = opt_bool(state, 2).unwrap_or(false);
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        #[cfg(feature = "retail-12-0-0")]
        if frame.widget_type == crate::widget::WidgetType::Cooldown {
            frame.cooldown_paused = paused;
            return Ok(0);
        }
        frame.model_state_mut().model_scene_state.paused = paused;
    }
    Ok(0)
}

pub(super) fn scene_get_paused(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let paused = borrow_state(state)?
        .widgets
        .get(id)
        .map(|frame| frame.model_state().model_scene_state.paused)
        .unwrap_or(false);
    state.push(Val::Bool(paused));
    Ok(1)
}

pub(super) fn scene_project_3d_point_to_2d(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let point = point3_from_stack(state);
    let Some(scene) = read_scene_projection_state(state, id)? else {
        state.push(Val::Nil);
        return Ok(1);
    };

    match project_scene_point(point, scene) {
        Some(projected) => projected.into_stack(state),
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

#[derive(Clone, Copy)]
struct SceneProjectionState {
    width: f32,
    height: f32,
    view_insets: (f32, f32, f32, f32),
    view_translation: (f32, f32),
    camera_position: (f32, f32, f32),
    camera_field_of_view: f32,
    camera_near_clip: f32,
}

fn point3_from_stack(state: &LuaState) -> (f32, f32, f32) {
    (
        val_to_f64(stack_val(state, 2)) as f32,
        val_to_f64(stack_val(state, 3)) as f32,
        val_to_f64(stack_val(state, 4)) as f32,
    )
}

fn read_scene_projection_state(
    state: &mut LuaState,
    id: u64,
) -> LuaResult<Option<SceneProjectionState>> {
    let sim = borrow_state(state)?;
    Ok(sim.widgets.get(id).map(|frame| {
        let camera = frame.model_state().model_scene_state.camera;
        SceneProjectionState {
            width: frame.width,
            height: frame.height,
            view_insets: frame.model_state().model_scene_state.view_insets,
            view_translation: frame.model_state().model_scene_state.view_translation,
            camera_position: camera.position,
            camera_field_of_view: camera.field_of_view,
            camera_near_clip: camera.near_clip,
        }
    }))
}

fn project_scene_point(
    point: (f32, f32, f32),
    scene: SceneProjectionState,
) -> Option<(f64, f64, f64)> {
    let depth = point.2 - scene.camera_position.2;
    if depth <= 0.0 {
        return None;
    }

    let (viewport_width, viewport_height) = projection_viewport_size(scene);
    let focal = (viewport_height * 0.5) / (scene.camera_field_of_view * 0.5).tan();
    let x = projected_x(point, scene, viewport_width, focal, depth);
    let y = projected_y(point, scene, viewport_height, focal, depth);
    let depth_value = 1.0 - (scene.camera_near_clip / depth.max(scene.camera_near_clip));

    Some((x as f64, y as f64, depth_value as f64))
}

fn projection_viewport_size(scene: SceneProjectionState) -> (f32, f32) {
    let viewport_width = (scene.width - scene.view_insets.0 - scene.view_insets.1).max(0.0);
    let viewport_height = (scene.height - scene.view_insets.2 - scene.view_insets.3).max(0.0);
    (viewport_width, viewport_height)
}

fn projected_x(
    point: (f32, f32, f32),
    scene: SceneProjectionState,
    viewport_width: f32,
    focal: f32,
    depth: f32,
) -> f32 {
    scene.view_insets.0
        + viewport_width * 0.5
        + scene.view_translation.0 / 6.0
        + (point.0 - scene.camera_position.0) * focal / depth
}

fn projected_y(
    point: (f32, f32, f32),
    scene: SceneProjectionState,
    viewport_height: f32,
    focal: f32,
    depth: f32,
) -> f32 {
    scene.view_insets.2
        + viewport_height * 0.5
        + scene.view_translation.1 * 6.0
        + (point.1 - scene.camera_position.1) * focal / depth
}
