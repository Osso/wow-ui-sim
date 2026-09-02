//! StatusBar widget methods.

use super::shared::{opt_f32, opt_string, rgba_from_stack, val_to_bool, val_to_f64};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, extract_frame_id, frame_id_from_stack,
    frame_ref, get_or_create_frame_fields, sync_child_to_rilua, table_get_static, table_set_static,
    val_to_string,
};
use crate::lua_bridge::{IntoStack, stack_val, table_set_rust_fn};
use crate::widget::WidgetType;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const TIMER_DURATION_FIELD: &str = "__statusbar_timer_duration";

pub(super) fn statusbar_child_id(sim: &crate::lua_api::SimState, id: u64) -> Option<u64> {
    let bar_id = sim.widgets.get(id)?.statusbar_bar_id?;
    sim.widgets
        .get(bar_id)
        .is_some_and(|f| f.parent_id == Some(id))
        .then_some(bar_id)
}

/// Resolve or create the bar-texture child, returning its id.
/// Returns `None` if `texture_val` is `Val::Nil` and no bar exists yet.
fn resolve_bar_id(state: &mut LuaState, id: u64, texture_val: rilua::Val) -> LuaResult<u64> {
    let existing = {
        let sim = borrow_state(state)?;
        sim.widgets.get(id).and_then(|f| f.statusbar_bar_id)
    };
    if let Some(bar_id) = existing {
        return Ok(bar_id);
    }
    if let Some(texture_id) = extract_frame_id(state, texture_val) {
        return Ok(texture_id);
    }
    create_bar_texture_child(state, id)
}

/// Create a new Texture child named "BarTexture" under `id`.
fn create_bar_texture_child(state: &mut LuaState, id: u64) -> LuaResult<u64> {
    let mut child = crate::widget::Frame::new(WidgetType::Texture, None, Some(id));
    child.parent_key = Some("BarTexture".to_string());
    let child_id = child.id;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.widgets.register(child);
        sim.widgets.add_child(id, child_id);
        if let Some(parent) = sim.widgets.get_mut_visual(id) {
            parent.statusbar_bar_id = Some(child_id);
            parent
                .children_keys
                .insert("BarTexture".to_string(), child_id);
        }
        sim.invalidate_strata_buckets();
    }
    sync_child_to_rilua(state, id, "BarTexture", child_id)?;
    Ok(child_id)
}

/// Write the texture path/file-id onto the bar child and update parent bookkeeping.
fn apply_bar_texture(
    sim: &mut crate::lua_api::SimState,
    id: u64,
    bar_id: u64,
    path: Option<String>,
    file_id: Option<i64>,
) {
    if let Some(parent) = sim.widgets.get_mut_visual(id) {
        parent.statusbar_bar_id = Some(bar_id);
        parent.statusbar_texture_path = path.clone();
        parent
            .children_keys
            .insert("BarTexture".to_string(), bar_id);
    }
    if let Some(bar) = sim.widgets.get_mut_visual(bar_id) {
        bar.parent_id = Some(id);
        bar.parent_key = Some("BarTexture".to_string());
        bar.texture_file_data_id = file_id;
        bar.color_texture = None;
        apply_statusbar_texture_source(bar, path);
    }
}

fn adopt_bar_texture(sim: &mut crate::lua_api::SimState, id: u64, bar_id: u64) {
    let inherited_path = sim
        .widgets
        .get(bar_id)
        .and_then(|bar| bar.atlas.clone().or_else(|| bar.texture.clone()));

    if let Some(parent) = sim.widgets.get_mut_visual(id) {
        parent.statusbar_bar_id = Some(bar_id);
        parent.statusbar_texture_path = inherited_path;
        parent
            .children_keys
            .insert("BarTexture".to_string(), bar_id);
    }

    if let Some(bar) = sim.widgets.get_mut_visual(bar_id) {
        bar.parent_id = Some(id);
        bar.parent_key = Some("BarTexture".to_string());
    }
}

fn apply_statusbar_texture_source(frame: &mut crate::widget::Frame, path: Option<String>) {
    let Some(path) = path else {
        frame.texture = None;
        frame.atlas = None;
        frame.tex_coords = None;
        frame.atlas_tex_coords = None;
        return;
    };

    if let Some(info) = crate::atlas::get_render_atlas_info(&path) {
        let tex_coords = (
            info.info.left_tex_coord,
            info.info.right_tex_coord,
            info.info.top_tex_coord,
            info.info.bottom_tex_coord,
        );
        frame.atlas = Some(path);
        frame.texture = Some(info.info.file.to_string());
        frame.tex_coords = Some(tex_coords);
        frame.atlas_tex_coords = Some(tex_coords);
        return;
    }

    frame.texture = Some(path);
    frame.atlas = None;
    frame.tex_coords = None;
    frame.atlas_tex_coords = None;
}

// ---------------------------------------------------------------------------
// Methods
// ---------------------------------------------------------------------------

pub(super) fn set_status_bar_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = val_to_f64(stack_val(state, 2)) as f32;
    let g = val_to_f64(stack_val(state, 3)) as f32;
    let b = val_to_f64(stack_val(state, 4)) as f32;
    let a = opt_f32(state, 5).unwrap_or(1.0);
    let color = crate::widget::Color::new(r, g, b, a);
    let mut sim = borrow_state_mut(state)?;
    let bar_id = statusbar_child_id(&sim, id);
    if let Some(bar_id) = bar_id {
        if let Some(bar) = sim.widgets.get_mut_visual(bar_id) {
            bar.vertex_color = Some(color);
        }
    }
    Ok(0)
}

pub(super) fn get_status_bar_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let bar_id = statusbar_child_id(&sim, id);
    let (r, g, b, a) = bar_id
        .and_then(|bid| sim.widgets.get(bid))
        .and_then(|f| f.vertex_color)
        .map(|c| (c.r as f64, c.g as f64, c.b as f64, c.a as f64))
        .unwrap_or((1.0, 1.0, 1.0, 1.0));
    drop(sim);
    (r, g, b, a).into_stack(state)
}

pub(super) fn set_color_fill(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(color) = rgba_from_stack(state, 2) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(frame) = sim.widgets.get(id)
        && frame.widget_type == WidgetType::StatusBar
    {
        let _ = frame;
        let bar_id = statusbar_child_id(&sim, id);
        if let Some(bar_id) = bar_id
            && let Some(bar) = sim.widgets.get_mut_visual(bar_id)
        {
            bar.vertex_color = Some(color);
        }
        return Ok(0);
    }
    if let Some(frame) = sim.widgets.get_mut_visual(id) {
        frame.color_texture = Some(color);
        frame.texture = None;
        frame.texture_file_data_id = None;
        frame.atlas = None;
        frame.atlas_tex_coords = None;
    }
    Ok(0)
}

pub(super) fn set_status_bar_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let texture_val = stack_val(state, 2);
    let texture_is_userdata = extract_frame_id(state, texture_val).is_some();
    let texture_has_path = val_to_string(state, texture_val).is_some();
    let preserves_existing_source = texture_is_userdata && !texture_has_path;
    let bar_id = resolve_bar_id(state, id, texture_val)?;
    let path = val_to_string(state, texture_val);
    let file_id = match texture_val {
        Val::Num(value) => Some(value as i64),
        _ => None,
    };
    let mut sim = borrow_state_mut(state)?;
    if preserves_existing_source && file_id.is_none() {
        adopt_bar_texture(&mut sim, id, bar_id);
    } else {
        apply_bar_texture(&mut sim, id, bar_id, path, file_id);
    }
    Ok(0)
}

pub(super) fn get_status_bar_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let bar_id = {
        let sim = borrow_state(state)?;
        statusbar_child_id(&sim, id)
    };
    match bar_id {
        Some(bar_id) => {
            let bar = frame_ref(state, bar_id)?;
            state.push(bar);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn set_fill_style(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let style = opt_string(state, 2).unwrap_or_else(|| "STANDARD".to_string());
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.statusbar_fill_style = style;
    }
    Ok(0)
}

pub(super) fn get_fill_style(state: &mut LuaState) -> LuaResult<u32> {
    let val = create_string(state, "STANDARD");
    val.into_stack(state)
}

pub(super) fn set_reverse_fill(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let reverse = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.statusbar_reverse_fill = reverse;
    }
    Ok(0)
}

pub(super) fn get_reverse_fill(state: &mut LuaState) -> LuaResult<u32> {
    false.into_stack(state)
}

pub(super) fn get_interpolated_value(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.statusbar_interpolated_value)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn is_interpolating(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .and_then(|f| f.statusbar_interpolation_target)
        .is_some();
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_to_target_value(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        if let Some(target) = f.statusbar_interpolation_target.take() {
            f.statusbar_interpolated_value = target;
        }
    }
    Ok(0)
}

pub(super) fn set_timer_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let duration = stack_val(state, 2);
    let fields = get_or_create_frame_fields(state, id);
    table_set_static(state, fields, TIMER_DURATION_FIELD, duration);
    Ok(0)
}

pub(super) fn get_timer_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let fields = get_or_create_frame_fields(state, id);
    let duration = table_get_static(state, fields, TIMER_DURATION_FIELD);
    state.push(duration);
    Ok(1)
}

pub(super) fn set_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desat = val_to_bool(stack_val(state, 2));
    apply_desaturation(state, id, if desat { 1.0 } else { 0.0 });
    Ok(0)
}

pub(super) fn get_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.statusbar_desaturation > 0.0)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_desaturation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let desat = val_to_f64(stack_val(state, 2));
    apply_desaturation(state, id, desat);
    Ok(0)
}

pub(super) fn get_desaturation(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.statusbar_desaturation)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

fn apply_desaturation(state: &mut LuaState, id: u64, desaturation: f64) {
    let clamped = desaturation.clamp(0.0, 1.0);
    let Ok(mut sim) = borrow_state_mut(state) else {
        return;
    };
    let child_id = sim.widgets.get(id).and_then(|f| f.statusbar_bar_id);
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.statusbar_desaturation = clamped;
    }
    if let Some(child_id) = child_id {
        let is_desat = sim
            .widgets
            .get(id)
            .map(|f| f.statusbar_desaturation > 0.0)
            .unwrap_or(false);
        if let Some(child) = sim.widgets.get_mut_visual(child_id) {
            child.desaturated = is_desat;
        }
    }
}

pub(super) fn get_rotates_texture(state: &mut LuaState) -> LuaResult<u32> {
    false.into_stack(state)
}

pub(super) fn set_rotates_texture(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

// Aliases
pub(super) fn is_status_bar_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    get_desaturated(state)
}

pub(super) fn get_status_bar_desaturation(state: &mut LuaState) -> LuaResult<u32> {
    get_desaturation(state)
}

pub(super) fn get_status_bar_desaturated(state: &mut LuaState) -> LuaResult<u32> {
    get_desaturated(state)
}

// ---------------------------------------------------------------------------
// register_statusbar
// ---------------------------------------------------------------------------

#[cfg(feature = "retail-12-1-0")]
fn set_render_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = crate::lua_api::methods::frame_id_from_stack(state, 1)?;
    let fields = crate::lua_api::methods::get_or_create_frame_fields(state, id);
    let value = crate::lua_bridge::stack_val(state, 2);
    let mode = crate::lua_api::methods::val_to_string(state, value)
        .map(|mode| crate::lua_api::methods::create_string(state, &mode))
        .unwrap_or(value);
    crate::lua_api::methods::table_set(state, fields, "__statusBarRenderMode", mode);
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn get_render_mode(state: &mut LuaState) -> LuaResult<u32> {
    let id = crate::lua_api::methods::frame_id_from_stack(state, 1)?;
    let fields = crate::lua_api::methods::get_or_create_frame_fields(state, id);
    let mode = crate::lua_api::methods::table_get(state, fields, "__statusBarRenderMode");
    if matches!(mode, Val::Nil) {
        let default_mode = crate::lua_api::methods::create_string(state, "Standard");
        state.push(default_mode);
    } else {
        state.push(mode);
    }
    Ok(1)
}

const STATUSBAR_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Colors + texture
    ("SetStatusBarColor", set_status_bar_color),
    ("SetColorFill", set_color_fill),
    ("SetStatusBarTexture", set_status_bar_texture),
    ("GetStatusBarTexture", get_status_bar_texture),
    ("GetStatusBarColor", get_status_bar_color),
    // Fill direction / reverse
    ("SetFillStyle", set_fill_style),
    ("GetFillStyle", get_fill_style),
    ("SetReverseFill", set_reverse_fill),
    ("GetReverseFill", get_reverse_fill),
    // Interpolation (smooth value changes)
    ("GetInterpolatedValue", get_interpolated_value),
    ("IsInterpolating", is_interpolating),
    ("SetToTargetValue", set_to_target_value),
    ("SetTimerDuration", set_timer_duration),
    ("GetTimerDuration", get_timer_duration),
    // Render mode
    #[cfg(feature = "retail-12-1-0")]
    ("SetRenderMode", set_render_mode),
    #[cfg(feature = "retail-12-1-0")]
    ("GetRenderMode", get_render_mode),
    // Desaturation
    ("SetStatusBarDesaturated", set_desaturated),
    ("GetStatusBarDesaturated", get_status_bar_desaturated),
    ("SetStatusBarDesaturation", set_desaturation),
    ("GetStatusBarDesaturation", get_status_bar_desaturation),
    ("IsStatusBarDesaturated", is_status_bar_desaturated),
    // Rotation
    ("SetRotatesTexture", set_rotates_texture),
    ("GetRotatesTexture", get_rotates_texture),
];

pub(super) fn register_statusbar(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in STATUSBAR_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}
