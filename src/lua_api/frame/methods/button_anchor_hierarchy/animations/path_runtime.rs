//! Path and Translation share target resolution and presentation offsets.
//! SMOOTH is simulator policy: uniform Catmull–Rom, not established native parity.
use super::resolve_anim_target_id;
use super::vertex_color::{animation_progress, elapsed_within_order};
use crate::lua_api::SimState;
use crate::lua_api::animation::{AnimGroupState, AnimState, AnimationType, LoopType};

pub(super) fn apply_group_motion_state(sim: &mut SimState, group_id: u64) {
    let Some(group) = sim.animation_groups.get(&group_id) else {
        return;
    };
    if group.paused {
        return;
    }
    if !group.playing && group.done {
        restore_group_motion_offsets(sim, group_id);
        return;
    }
    if !group.playing || group.looping != LoopType::None {
        return;
    }
    let updates: Vec<_> = group
        .animations
        .iter()
        .filter_map(|animation| {
            let displacement = match animation.anim_type {
                AnimationType::Translation => interpolate_translation(animation, group)?,
                _ => interpolate_path(animation)?,
            };
            let target_id =
                resolve_anim_target_id(sim, group.owner_frame_id, animation.child_key.as_deref())?;
            let frame = sim.widgets.get(target_id)?;
            let baseline = [frame.anim_offset_x, frame.anim_offset_y];
            Some((target_id, baseline, displacement))
        })
        .collect();
    apply_motion_offsets(sim, group_id, updates);
}

fn apply_motion_offsets(
    sim: &mut SimState,
    group_id: u64,
    updates: Vec<(u64, [f32; 2], [f32; 2])>,
) {
    for (target_id, baseline, displacement) in updates {
        let Some(group) = sim.animation_groups.get_mut(&group_id) else {
            return;
        };
        let saved = *group
            .saved_motion_offsets
            .entry(target_id)
            .or_insert(baseline);
        let offset = [saved[0] + displacement[0], saved[1] + displacement[1]];
        write_motion_offset(sim, target_id, offset);
    }
}

fn restore_group_motion_offsets(sim: &mut SimState, group_id: u64) {
    let saved = sim
        .animation_groups
        .get_mut(&group_id)
        .map(|group| std::mem::take(&mut group.saved_motion_offsets))
        .unwrap_or_default();
    for (target_id, offset) in saved {
        write_motion_offset(sim, target_id, offset);
    }
}

fn write_motion_offset(sim: &mut SimState, target_id: u64, offset: [f32; 2]) {
    let changed = sim
        .widgets
        .get(target_id)
        .is_some_and(|frame| [frame.anim_offset_x, frame.anim_offset_y] != offset);
    if changed && let Some(frame) = sim.widgets.get_mut_visual(target_id) {
        [frame.anim_offset_x, frame.anim_offset_y] = offset;
        sim.widgets.mark_rect_dirty(target_id);
    }
}

// Model policy: use the existing eased time and layout axes, not native geometry.
fn interpolate_translation(animation: &AnimState, group: &AnimGroupState) -> Option<[f32; 2]> {
    if animation.duration <= 0.0 {
        return None;
    }
    let elapsed = elapsed_within_order(group, animation.order);
    let progress = animation_progress(animation, elapsed);
    Some(
        animation
            .translation_offset
            .map(|offset| (offset * progress) as f32),
    )
}

fn interpolate_path(animation: &AnimState) -> Option<[f32; 2]> {
    let is_supported_path = animation.anim_type == AnimationType::Path
        && matches!(animation.curve.as_str(), "NONE" | "SMOOTH")
        && animation.smoothing == "NONE";
    if !is_supported_path || animation.duration <= 0.0 {
        return None;
    }
    let points = animation.ordered_control_points();
    if points.len() < 2 {
        return None;
    }
    let segment_count = points.len() - 1;
    let progress = (animation.elapsed / animation.duration).clamp(0.0, 1.0);
    let segment_position = progress * segment_count as f64;
    let index = (segment_position.floor() as usize).min(segment_count - 1);
    let fraction = segment_position - index as f64;
    let offsets = [
        points[index.saturating_sub(1)].offset,
        points[index].offset,
        points[index + 1].offset,
        points[(index + 2).min(points.len() - 1)].offset,
    ];
    interpolate_segment(&animation.curve, offsets, fraction)
}

fn interpolate_segment(
    curve: &str,
    [previous, start, end, next]: [[f64; 2]; 4],
    fraction: f64,
) -> Option<[f32; 2]> {
    match curve {
        "NONE" => Some(std::array::from_fn(|axis| {
            (start[axis] + (end[axis] - start[axis]) * fraction) as f32
        })),
        "SMOOTH" => Some(std::array::from_fn(|axis| {
            catmull_rom(
                [previous[axis], start[axis], end[axis], next[axis]],
                fraction,
            ) as f32
        })),
        _ => None,
    }
}

fn catmull_rom([p0, p1, p2, p3]: [f64; 4], t: f64) -> f64 {
    0.5 * (2.0 * p1
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t * t
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t * t * t)
}
