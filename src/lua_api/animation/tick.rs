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
            .filter(|(_, g)| g.playing && !g.paused)
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
        _ => {} // Scale, Rotation, etc. not yet implemented
    }
}

/// Resolve child_key to frame ID and apply effects to widget state.
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
        let Some(frame) = state.widgets.get_mut(id) else { continue };
        if let Some(alpha) = fx.alpha {
            frame.alpha = alpha;
        }
        frame.anim_offset_x = fx.offset_x;
        frame.anim_offset_y = fx.offset_y;
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
