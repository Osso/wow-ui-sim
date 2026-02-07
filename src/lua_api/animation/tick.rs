//! Animation tick logic: advance playing groups, apply alpha, fire scripts.

use crate::lua_api::SimState;
use mlua::Lua;
use std::cell::RefCell;
use std::rc::Rc;

use super::{AnimGroupHandle, AnimGroupState, AnimationType, LoopType};

/// Advance all playing animation groups by `delta` seconds.
/// Applies alpha animations to frame opacity and fires script callbacks.
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

/// Advance a single animation group: update elapsed times, compute alpha, handle looping.
fn advance_group(
    state_rc: &Rc<RefCell<SimState>>,
    group_id: &u64,
    lua: &Lua,
    delta: f64,
) -> (bool, Vec<mlua::Function>) {
    let mut state = state_rc.borrow_mut();

    let (group_finished, alpha_to_apply, scripts_to_fire) = {
        let Some(group) = state.animation_groups.get_mut(group_id) else {
            return (false, Vec::new());
        };

        let effective_delta = delta * group.speed_multiplier;
        group.elapsed += effective_delta;

        let alpha_to_apply = advance_order_groups(group);
        let total_dur = group.total_duration();
        let group_finished = group.elapsed >= total_dur;

        let scripts_to_fire = if group_finished {
            handle_group_finish(group, total_dur, lua)
        } else {
            Vec::new()
        };

        (group_finished, alpha_to_apply, scripts_to_fire)
    };

    if let Some((frame_id, alpha)) = alpha_to_apply
        && let Some(frame) = state.widgets.get_mut(frame_id) {
            frame.alpha = alpha;
        }

    (group_finished, scripts_to_fire)
}

/// Advance animations within each order group, returning computed alpha if any.
fn advance_order_groups(group: &mut AnimGroupState) -> Option<(u64, f32)> {
    let owner_frame_id = group.owner_frame_id;
    let orders = group.order_groups();
    let mut order_start = 0.0;
    let mut alpha_to_apply: Option<(u64, f32)> = None;

    for &order in &orders {
        let order_dur = group.animations.iter()
            .filter(|a| a.order == order)
            .map(|a| a.total_time())
            .fold(0.0_f64, f64::max);

        let time_in_group = (group.elapsed - order_start).clamp(0.0, order_dur);

        for anim in group.animations.iter_mut().filter(|a| a.order == order) {
            anim.elapsed = time_in_group.min(anim.total_time());
        }

        for anim in group.animations.iter().filter(|a| a.order == order) {
            if anim.anim_type == AnimationType::Alpha {
                let progress = if group.reverse {
                    1.0 - anim.smooth_progress()
                } else {
                    anim.smooth_progress()
                };
                let alpha = anim.from_alpha + (anim.to_alpha - anim.from_alpha) * progress;
                alpha_to_apply = Some((owner_frame_id, alpha as f32));
            }
        }

        order_start += order_dur;
    }

    alpha_to_apply
}

/// Handle a finished animation group: update state and collect scripts to fire.
fn handle_group_finish(
    group: &mut AnimGroupState,
    total_dur: f64,
    lua: &Lua,
) -> Vec<mlua::Function> {
    let mut scripts = Vec::new();

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
            if let Some(key) = group.scripts.get("OnLoop")
                && let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
        }
        LoopType::Bounce => {
            group.elapsed -= total_dur;
            group.reverse = !group.reverse;
            for anim in &mut group.animations {
                anim.elapsed = 0.0;
            }
            if let Some(key) = group.scripts.get("OnLoop")
                && let Ok(func) = lua.registry_value::<mlua::Function>(key) {
                    scripts.push(func);
                }
        }
    }

    scripts
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
