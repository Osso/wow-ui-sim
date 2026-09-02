//! Cooldown widget methods.

use super::shared::{animation_group_id_for_frame, opt_string, val_to_bool, val_to_f64};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, call_function_state_multi,
    frame_id_from_stack, frame_ref, sync_child_to_rilua, table_get,
};
use crate::lua_bridge::{IntoStack, stack_val, table_set_rust_fn};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

fn normalize_mod_rate(r: f64) -> f64 {
    if r <= 0.0 { 1.0 } else { r }
}

fn apply_cooldown_state(
    frame: &mut crate::widget::Frame,
    start: f64,
    duration: f64,
    mod_rate: f64,
) {
    frame.cooldown_start = start;
    frame.cooldown_duration = duration;
    frame.cooldown_display_duration_ms = duration.max(0.0) * 1000.0;
    frame.cooldown_mod_rate = normalize_mod_rate(mod_rate);
}

fn clear_cooldown_timing(frame: &mut crate::widget::Frame) {
    frame.cooldown_start = 0.0;
    frame.cooldown_duration = 0.0;
    frame.cooldown_display_duration_ms = 0.0;
    frame.cooldown_mod_rate = 1.0;
}

fn font_object_from_arg(state: &mut LuaState, arg_index: i32) -> Option<Val> {
    match stack_val(state, arg_index) {
        table @ Val::Table(_) => Some(table),
        Val::Str(_) => {
            let name = opt_string(state, arg_index)?;
            Some(table_get(state, Val::Table(state.global), &name))
        }
        _ => None,
    }
}

fn font_field_string(
    state: &mut LuaState,
    table: Val,
    primary: &str,
    fallback: &str,
) -> Option<String> {
    let primary_value = table_get(state, table, primary);
    let value = match primary_value {
        Val::Str(_) => primary_value,
        _ => table_get(state, table, fallback),
    };
    crate::lua_api::methods::val_to_string(state, value)
}

fn font_field_number(
    state: &mut LuaState,
    table: Val,
    primary: &str,
    fallback: &str,
) -> Option<f64> {
    match table_get(state, table, primary) {
        Val::Num(value) => Some(value),
        _ => match table_get(state, table, fallback) {
            Val::Num(value) => Some(value),
            _ => None,
        },
    }
}

fn ensure_countdown_font_string(state: &mut LuaState, parent_id: u64) -> Option<u64> {
    if let Ok(sim) = borrow_state(state)
        && let Some(existing) = sim
            .widgets
            .get(parent_id)
            .and_then(|frame| frame.cooldown_countdown_font_string_id)
    {
        return Some(existing);
    }

    let child_id = {
        let mut sim = borrow_state_mut(state).ok()?;
        let mut child =
            crate::widget::Frame::new(crate::widget::WidgetType::FontString, None, Some(parent_id));
        child.object_type_name = Some("FontString".to_string());
        let child_id = child.id;
        sim.widgets.register(child);
        sim.widgets.add_child(parent_id, child_id);
        if let Some(parent) = sim.widgets.get_mut_visual(parent_id) {
            parent.cooldown_countdown_font_string_id = Some(child_id);
        }
        child_id
    };
    let _ = sync_child_to_rilua(state, parent_id, "Countdown", child_id);
    Some(child_id)
}

pub(super) fn set_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let start = val_to_f64(stack_val(state, 2));
    let duration = val_to_f64(stack_val(state, 3));
    let mod_rate = normalize_mod_rate(val_to_f64(stack_val(state, 4)));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        apply_cooldown_state(f, start, duration, mod_rate);
    }
    Ok(0)
}

pub(super) fn set_cooldown_unix(state: &mut LuaState) -> LuaResult<u32> {
    set_cooldown(state)
}

pub(super) fn set_cooldown_from_expiration_time(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let expiration = val_to_f64(stack_val(state, 2));
    let duration = val_to_f64(stack_val(state, 3));
    let mod_rate = normalize_mod_rate(val_to_f64(stack_val(state, 4)));
    let start = expiration - duration;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        apply_cooldown_state(f, start, duration, mod_rate);
    }
    Ok(0)
}

pub(super) fn set_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let duration = val_to_f64(stack_val(state, 2));
    let mod_rate = normalize_mod_rate(val_to_f64(stack_val(state, 3)));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        let start = f.cooldown_start;
        apply_cooldown_state(f, start, duration, mod_rate);
    }
    Ok(0)
}

pub(super) fn get_cooldown_times(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let (s, d) = sim
        .widgets
        .get(id)
        .map(|f| (f.cooldown_start, f.cooldown_duration))
        .unwrap_or((0.0, 0.0));
    drop(sim);
    (s, d).into_stack(state)
}

pub(super) fn get_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_duration)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn get_cooldown_display_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_display_duration_ms)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn clear(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        clear_cooldown_timing(f);
    }
    Ok(0)
}

pub(super) fn pause(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = animation_group_id_for_frame(&sim, id)
        && let Some(group) = sim.animation_groups.get_mut(&group_id)
    {
        group.paused = true;
        group.playing = false;
        return Ok(0);
    }
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_paused = true;
    }
    Ok(0)
}

pub(super) fn resume(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    if let Some(group_id) = animation_group_id_for_frame(&sim, id)
        && let Some(group) = sim.animation_groups.get_mut(&group_id)
    {
        group.paused = false;
        group.playing = true;
        return Ok(0);
    }
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_paused = false;
    }
    Ok(0)
}

pub(super) fn is_paused(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    if let Some(group_id) = animation_group_id_for_frame(&sim, id) {
        let paused = sim
            .animation_groups
            .get(&group_id)
            .map(|group| group.paused)
            .unwrap_or(false);
        drop(sim);
        return paused.into_stack(state);
    }
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_paused)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_draw_swipe(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let draw = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_draw_swipe = draw;
    }
    Ok(0)
}

pub(super) fn get_draw_swipe(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_draw_swipe)
        .unwrap_or(true);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_draw_edge(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let draw = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_draw_edge = draw;
    }
    Ok(0)
}

pub(super) fn get_draw_edge(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_draw_edge)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_draw_bling(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let draw = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_draw_bling = draw;
    }
    Ok(0)
}

pub(super) fn get_draw_bling(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_draw_bling)
        .unwrap_or(true);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_reverse(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let rev = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_reverse = rev;
    }
    Ok(0)
}

pub(super) fn get_reverse(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_reverse)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_hide_countdown_numbers(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let hide = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_hide_countdown = hide;
    }
    Ok(0)
}

pub(super) fn get_hide_countdown_numbers(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_hide_countdown)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_edge_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let scale = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_edge_scale = scale;
    }
    Ok(0)
}

pub(super) fn get_edge_scale(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_edge_scale)
        .unwrap_or(1.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_minimum_countdown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let ms = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_min_countdown_duration_ms = ms;
    }
    Ok(0)
}

pub(super) fn get_minimum_countdown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_min_countdown_duration_ms)
        .unwrap_or(0.0);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_use_aura_display_time(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_use_aura_display_time = enabled;
    }
    Ok(0)
}

pub(super) fn get_use_aura_display_time(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let sim = borrow_state(state)?;
    let v = sim
        .widgets
        .get(id)
        .map(|f| f.cooldown_use_aura_display_time)
        .unwrap_or(false);
    drop(sim);
    v.into_stack(state)
}

pub(super) fn set_use_circular_edge(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = val_to_bool(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_use_circular_edge = enabled;
    }
    Ok(0)
}

pub(super) fn set_countdown_abbrev_threshold(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let threshold = val_to_f64(stack_val(state, 2));
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_countdown_abbrev_threshold_seconds = threshold;
    }
    Ok(0)
}

pub(super) fn set_swipe_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = opt_string(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_swipe_texture = path;
    }
    Ok(0)
}

pub(super) fn set_bling_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = opt_string(state, 2);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_bling_texture = path;
    }
    Ok(0)
}

pub(super) fn set_countdown_font(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some(font_name) = opt_string(state, 2) else {
        return Ok(0);
    };
    let font_object = font_object_from_arg(state, 2);
    let child_id = ensure_countdown_font_string(state, id);
    if let Some(child_id) = child_id {
        let font_path = font_object
            .clone()
            .and_then(|font| font_field_string(state, font, "__fontPath", "__font"));
        let font_height =
            font_object.and_then(|font| font_field_number(state, font, "__fontHeight", "__height"));
        let mut sim = borrow_state_mut(state)?;
        if let Some(child) = sim.widgets.get_mut_visual(child_id) {
            child.font = Some(font_path.unwrap_or(font_name));
            if let Some(height) = font_height {
                child.font_size = height as f32;
            }
        }
    }
    Ok(0)
}

pub(super) fn get_countdown_font_string(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let child_id = ensure_countdown_font_string(state, id);
    match child_id {
        Some(cid) => {
            let val = frame_ref(state, cid)?;
            val.into_stack(state)
        }
        None => {
            state.push(Val::Nil);
            Ok(1)
        }
    }
}

pub(super) fn set_from_duration_object(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let duration_object = stack_val(state, 2);
    let zero = {
        let is_zero = table_get(state, duration_object, "IsZero");
        matches!(
            call_function_state(state, is_zero, &[duration_object]),
            Ok(Val::Bool(true))
        )
    };
    if zero {
        if let Some(frame) = borrow_state_mut(state)?.widgets.get_mut_visual(id) {
            clear_cooldown_timing(frame);
        }
        return Ok(0);
    }

    let start = table_get(state, duration_object, "GetStartTime");
    let duration = table_get(state, duration_object, "GetTotalDuration");
    let mod_rate = table_get(state, duration_object, "GetModRate");
    let start = match call_function_state(state, start, &[duration_object]) {
        Ok(Val::Num(value)) => value,
        _ => 0.0,
    };
    let duration = match call_function_state(state, duration, &[duration_object]) {
        Ok(Val::Num(value)) => value,
        _ => 0.0,
    };
    let mod_rate = match call_function_state(state, mod_rate, &[duration_object]) {
        Ok(Val::Num(value)) => value,
        _ => 1.0,
    };
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut_visual(id) else {
        return Ok(0);
    };
    apply_cooldown_state(frame, start, duration, mod_rate);
    Ok(0)
}

pub(super) fn set_edge_texture(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let path = opt_string(state, 2);
    let tint = super::shared::rgba_from_stack(state, 3);
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_edge_texture = path;
        if let Some(tint) = tint {
            f.cooldown_edge_color = tint;
        }
    }
    Ok(0)
}

pub(super) fn set_swipe_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = val_to_f64(stack_val(state, 2)) as f32;
    let g = val_to_f64(stack_val(state, 3)) as f32;
    let b = val_to_f64(stack_val(state, 4)) as f32;
    let a = val_to_f64(stack_val(state, 5)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.attributes.insert(
            "__swipe_color".to_string(),
            crate::widget::AttributeValue::String(format!("{},{},{},{}", r, g, b, a)),
        );
    }
    Ok(0)
}

pub(super) fn set_edge_color(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let r = val_to_f64(stack_val(state, 2)) as f32;
    let g = val_to_f64(stack_val(state, 3)) as f32;
    let b = val_to_f64(stack_val(state, 4)) as f32;
    let a = val_to_f64(stack_val(state, 5)) as f32;
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_edge_color = crate::widget::Color::new(r, g, b, a);
    }
    Ok(0)
}

fn vector2_from_stack(state: &mut LuaState, index: i32) -> Option<(f32, f32)> {
    let vector = stack_val(state, index);
    vector2_from_xy_fields(state, vector).or_else(|| vector2_from_get_xy(state, vector))
}

fn vector2_from_xy_fields(state: &mut LuaState, vector: Val) -> Option<(f32, f32)> {
    let Val::Num(x) = table_get(state, vector, "x") else {
        return None;
    };
    let Val::Num(y) = table_get(state, vector, "y") else {
        return None;
    };
    Some((x as f32, y as f32))
}

fn vector2_from_get_xy(state: &mut LuaState, vector: Val) -> Option<(f32, f32)> {
    let get_xy = table_get(state, vector, "GetXY");
    let values = call_function_state_multi(state, get_xy, &[vector]).ok()?;
    let [Val::Num(x), Val::Num(y), ..] = values.as_slice() else {
        return None;
    };
    Some((*x as f32, *y as f32))
}

pub(super) fn set_tex_coord_range(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let Some((low_x, low_y)) = vector2_from_stack(state, 2) else {
        return Ok(0);
    };
    let Some((high_x, high_y)) = vector2_from_stack(state, 3) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(f) = sim.widgets.get_mut_visual(id) {
        f.cooldown_tex_coord_range = Some((low_x, low_y, high_x, high_y));
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// register_cooldown
// ---------------------------------------------------------------------------

const COOLDOWN_METHODS: &[(&'static str, rilua::vm::closure::RustFn)] = &[
    // Duration / expiration configuration
    ("SetCooldown", set_cooldown),
    ("SetCooldownUNIX", set_cooldown_unix),
    (
        "SetCooldownFromExpirationTime",
        set_cooldown_from_expiration_time,
    ),
    ("SetCooldownDuration", set_cooldown_duration),
    ("SetCooldownFromDurationObject", set_from_duration_object),
    ("GetCooldownTimes", get_cooldown_times),
    ("GetCooldownDuration", get_cooldown_duration),
    ("GetCooldownDisplayDuration", get_cooldown_display_duration),
    // Playback controls
    ("Clear", clear),
    ("Pause", pause),
    ("Resume", resume),
    ("IsPaused", is_paused),
    // Drawing flags
    ("SetDrawSwipe", set_draw_swipe),
    ("GetDrawSwipe", get_draw_swipe),
    ("SetDrawEdge", set_draw_edge),
    ("GetDrawEdge", get_draw_edge),
    ("SetDrawBling", set_draw_bling),
    ("GetDrawBling", get_draw_bling),
    ("SetReverse", set_reverse),
    ("GetReverse", get_reverse),
    // Countdown number formatting
    ("SetHideCountdownNumbers", set_hide_countdown_numbers),
    ("GetHideCountdownNumbers", get_hide_countdown_numbers),
    (
        "SetMinimumCountdownDuration",
        set_minimum_countdown_duration,
    ),
    (
        "GetMinimumCountdownDuration",
        get_minimum_countdown_duration,
    ),
    (
        "SetCountdownAbbrevThreshold",
        set_countdown_abbrev_threshold,
    ),
    // Edge scaling
    ("SetEdgeScale", set_edge_scale),
    ("GetEdgeScale", get_edge_scale),
    // Aura-display-time mode + circular edge
    ("SetUseAuraDisplayTime", set_use_aura_display_time),
    ("GetUseAuraDisplayTime", get_use_aura_display_time),
    ("SetUseCircularEdge", set_use_circular_edge),
    // Textures / fonts / colors
    ("SetSwipeTexture", set_swipe_texture),
    ("SetEdgeTexture", set_edge_texture),
    ("SetBlingTexture", set_bling_texture),
    ("SetCountdownFont", set_countdown_font),
    ("GetCountdownFontString", get_countdown_font_string),
    ("SetSwipeColor", set_swipe_color),
    ("SetEdgeColor", set_edge_color),
    ("SetTexCoordRange", set_tex_coord_range),
];

pub(super) fn register_cooldown(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    for (name, func) in COOLDOWN_METHODS {
        table_set_rust_fn(state, metatable, name, *func)?;
    }
    Ok(())
}
