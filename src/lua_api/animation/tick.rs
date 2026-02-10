//! Animation tick logic: advance playing groups, apply alpha/translation, fire scripts.

use crate::lua_api::SimState;
use mlua::Lua;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::{AnimGroupHandle, AnimGroupState, AnimationType, LoopType};
use super::group_handle::stop_group;

/// Advance all playing animation groups by `delta` seconds.
/// Applies alpha and translation animations to target frames and fires script callbacks.
pub fn tick_animation_groups(state_rc: &Rc<RefCell<SimState>>, lua: &Lua, delta: f64) -> mlua::Result<()> {
    let playing_ids: Vec<u64> = {
        let state = state_rc.borrow();
        state.animation_groups.iter()
            .filter(|(_, g)| g.playing && !g.paused
                && state.widgets.is_ancestor_visible(g.owner_frame_id))
            .map(|(id, _)| *id)
            .collect()
    };

    for group_id in playing_ids {
        let (finished, scripts_to_fire) = advance_group(state_rc, &group_id, lua, delta);
        fire_animation_scripts(state_rc, lua, group_id, finished, &scripts_to_fire, delta)?;
    }

    Ok(())
}

/// Per-target animation effects collected during a tick.
#[derive(Default)]
struct TargetEffects {
    alpha: Option<f32>,
    offset_x: f32,
    offset_y: f32,
    /// FlipBook: (rows, columns, frames, progress) to compute UV sub-region.
    flipbook: Option<(u32, u32, u32, f64)>,
}

/// Advance a single animation group: update elapsed times, compute effects, handle finish.
fn advance_group(
    state_rc: &Rc<RefCell<SimState>>,
    group_id: &u64,
    lua: &Lua,
    delta: f64,
) -> (bool, Vec<mlua::Function>) {
    let mut state = state_rc.borrow_mut();

    let (group_finished, effects, scripts_to_fire, owner_frame_id) = {
        let Some(group) = state.animation_groups.get_mut(group_id) else {
            return (false, Vec::new());
        };

        group.elapsed += delta * group.speed_multiplier;

        let effects = collect_effects(group);
        let group_finished = group.elapsed >= group.total_duration();
        let owner_id = group.owner_frame_id;

        let scripts_to_fire = if group_finished {
            handle_group_finish(group, lua)
        } else {
            Vec::new()
        };

        (group_finished, effects, scripts_to_fire, owner_id)
    };

    apply_effects(&mut state, owner_frame_id, &effects);

    // On finish, run stop_group to handle setToFinalAlpha/translation cleanup.
    // For non-looping groups, handle_group_finish already set finished=true.
    if group_finished {
        let is_looping = state.animation_groups.get(group_id)
            .is_some_and(|g| g.looping != LoopType::None);
        if !is_looping {
            // stop_group handles alpha restore (if !setToFinalAlpha) and translation clear.
            // But for setToFinalAlpha=true, we want to keep the final alpha we just applied.
            stop_group(&mut state, *group_id);
        }
    }

    (group_finished, scripts_to_fire)
}

/// Collect per-target effects from all active animations in the group.
///
/// Respects start_delay: animations still in their delay period don't contribute.
/// For multiple animations targeting the same child, the last active one wins.
fn collect_effects(group: &mut AnimGroupState) -> HashMap<Option<String>, TargetEffects> {
    let orders = group.order_groups();
    let mut order_start = 0.0;
    let mut effects: HashMap<Option<String>, TargetEffects> = HashMap::new();

    for &order in &orders {
        let order_dur = group.animations.iter()
            .filter(|a| a.order == order)
            .map(|a| a.total_time())
            .fold(0.0_f64, f64::max);

        let time_in_group = (group.elapsed - order_start).clamp(0.0, order_dur);

        // Update elapsed time for each animation in this order group
        for anim in group.animations.iter_mut().filter(|a| a.order == order) {
            anim.elapsed = time_in_group.min(anim.total_time());
        }

        // Collect effects from active animations (past their start_delay)
        for anim in group.animations.iter().filter(|a| a.order == order) {
            if !anim.is_active() {
                continue;
            }
            let progress = compute_progress(anim, group.reverse);
            let entry = effects.entry(anim.child_key.clone()).or_default();
            apply_anim_to_entry(anim, progress, entry);
        }

        order_start += order_dur;
    }

    effects
}

/// Compute smoothed progress for a single animation, accounting for reverse.
fn compute_progress(anim: &super::AnimState, reverse: bool) -> f64 {
    if reverse { 1.0 - anim.smooth_progress() } else { anim.smooth_progress() }
}

/// Apply a single animation's contribution to the target effects entry.
fn apply_anim_to_entry(anim: &super::AnimState, progress: f64, entry: &mut TargetEffects) {
    match anim.anim_type {
        AnimationType::Alpha => {
            let alpha = anim.from_alpha + (anim.to_alpha - anim.from_alpha) * progress;
            entry.alpha = Some(alpha as f32);
        }
        AnimationType::Translation => {
            entry.offset_x = (anim.offset_x * progress) as f32;
            entry.offset_y = (anim.offset_y * progress) as f32;
        }
        AnimationType::FlipBook => {
            entry.flipbook = Some((
                anim.flip_book_rows,
                anim.flip_book_columns,
                anim.flip_book_frames,
                progress,
            ));
        }
        _ => {} // Scale, Rotation, etc. not yet implemented
    }
}

/// Resolve child_key to frame ID and apply effects to widget state.
///
/// Uses `get_mut_silent` to avoid setting `render_dirty` every tick.
/// Animations modify alpha/offset continuously; the render pipeline
/// picks up changes via `quads_dirty` which is set when something
/// structurally changes (show/hide, resize, texture swap).  Animation
/// alpha/translation are already baked into each quad rebuild, so the
/// existing 33ms rebuild throttle is sufficient.
fn apply_effects(
    state: &mut SimState,
    owner_frame_id: u64,
    effects: &HashMap<Option<String>, TargetEffects>,
) {
    for (child_key, fx) in effects {
        let target_id = match child_key {
            Some(key) => state.widgets.get(owner_frame_id)
                .and_then(|owner| owner.children_keys.get(key.as_str()).copied()),
            None => Some(owner_frame_id),
        };
        let Some(id) = target_id else { continue };
        let Some(frame) = state.widgets.get_mut_silent(id) else { continue };
        if let Some(alpha) = fx.alpha {
            frame.alpha = alpha;
        }
        let offset_changed = frame.anim_offset_x != fx.offset_x
            || frame.anim_offset_y != fx.offset_y;
        frame.anim_offset_x = fx.offset_x;
        frame.anim_offset_y = fx.offset_y;
        if let Some((rows, cols, frames, progress)) = fx.flipbook {
            apply_flipbook_uv(frame, rows, cols, frames, progress);
        }
        if offset_changed {
            state.invalidate_layout(id);
        }
    }
}

/// Compute and set UV sub-region for the current flipbook frame.
fn apply_flipbook_uv(
    frame: &mut crate::widget::Frame,
    rows: u32,
    cols: u32,
    frames: u32,
    progress: f64,
) {
    if rows == 0 || cols == 0 || frames == 0 {
        return;
    }
    let idx = ((progress * frames as f64).floor() as u32).min(frames - 1);
    let row = idx / cols;
    let col = idx % cols;

    // Use atlas_tex_coords as the full spritesheet region, fall back to tex_coords
    let (left, right, top, bottom) = frame.atlas_tex_coords
        .or(frame.tex_coords)
        .unwrap_or((0.0, 1.0, 0.0, 1.0));

    let frame_u = (right - left) / cols as f32;
    let frame_v = (bottom - top) / rows as f32;

    let new_left = left + col as f32 * frame_u;
    let new_top = top + row as f32 * frame_v;

    frame.tex_coords = Some((new_left, new_left + frame_u, new_top, new_top + frame_v));
}

/// Apply flipbook UV effects for a group (used when pausing to show current frame).
pub(super) fn apply_flipbook_for_group(state: &mut SimState, group_id: u64) {
    let flipbook_data: Vec<(Option<String>, u32, u32, u32, f64)> = {
        let Some(group) = state.animation_groups.get(&group_id) else { return };
        group.animations.iter()
            .filter(|a| a.anim_type == AnimationType::FlipBook)
            .map(|a| {
                let progress = a.smooth_progress();
                (a.child_key.clone(), a.flip_book_rows, a.flip_book_columns, a.flip_book_frames, progress)
            })
            .collect()
    };

    let owner_id = {
        let Some(group) = state.animation_groups.get(&group_id) else { return };
        group.owner_frame_id
    };

    for (child_key, rows, cols, frames, progress) in flipbook_data {
        let target_id = match &child_key {
            Some(key) => state.widgets.get(owner_id)
                .and_then(|owner| owner.children_keys.get(key.as_str()).copied()),
            None => Some(owner_id),
        };
        if let Some(id) = target_id {
            if let Some(frame) = state.widgets.get_mut(id) {
                apply_flipbook_uv(frame, rows, cols, frames, progress);
            }
        }
    }
}

/// Handle a finished animation group: update state and collect scripts to fire.
fn handle_group_finish(group: &mut AnimGroupState, lua: &Lua) -> Vec<mlua::Function> {
    let mut scripts = Vec::new();
    let total_dur = group.total_duration();

    match group.looping {
        LoopType::None => {
            group.playing = false;
            group.finished = true;
            if let Some(key) = group.scripts.get("OnFinished")
                && let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
        }
        LoopType::Repeat => {
            group.elapsed -= total_dur;
            for anim in &mut group.animations {
                anim.elapsed = 0.0;
            }
            collect_loop_script(&mut scripts, group, lua);
        }
        LoopType::Bounce => {
            group.elapsed -= total_dur;
            group.reverse = !group.reverse;
            for anim in &mut group.animations {
                anim.elapsed = 0.0;
            }
            collect_loop_script(&mut scripts, group, lua);
        }
    }

    scripts
}

/// Collect OnLoop script if present.
fn collect_loop_script(
    scripts: &mut Vec<mlua::Function>,
    group: &AnimGroupState,
    lua: &Lua,
) {
    if let Some(key) = group.scripts.get("OnLoop")
        && let Ok(func) = lua.registry_value::<mlua::Function>(key) {
            scripts.push(func);
        }
}

/// Fire collected animation scripts and OnUpdate callback for a group.
fn fire_animation_scripts(
    state_rc: &Rc<RefCell<SimState>>,
    lua: &Lua,
    group_id: u64,
    finished: bool,
    scripts_to_fire: &[mlua::Function],
    delta: f64,
) -> mlua::Result<()> {
    for func in scripts_to_fire {
        let handle = AnimGroupHandle {
            group_id,
            state: Rc::clone(state_rc),
        };
        let ud = lua.create_userdata(handle)?;
        if let Err(e) = func.call::<()>(ud) {
            eprintln!("Animation script error: {e}");
        }
    }

    let on_update_func = {
        let state = state_rc.borrow();
        state.animation_groups.get(&group_id)
            .and_then(|g| {
                if g.playing || !finished {
                    g.scripts.get("OnUpdate")
                        .and_then(|key| lua.registry_value::<mlua::Function>(key).ok())
                } else {
                    None
                }
            })
    };

    if let Some(func) = on_update_func {
        let handle = AnimGroupHandle {
            group_id,
            state: Rc::clone(state_rc),
        };
        let ud = lua.create_userdata(handle)?;
        if let Err(e) = func.call::<()>((ud, delta)) {
            eprintln!("Animation OnUpdate error: {e}");
        }
    }

    Ok(())
}
