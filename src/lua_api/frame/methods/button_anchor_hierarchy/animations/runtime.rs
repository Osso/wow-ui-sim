use super::{animation_group_frame_id, current_group_total_duration, resolve_anim_target_id};

fn finish_group_now(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
) -> u64 {
    group.elapsed = total_duration.max(0.0);
    group.playing = false;
    group.paused = false;
    group.done = true;
    group.pending_finish = false;
    sync_animation_elapsed(group);
    animation_group_frame_id(group)
}

pub(crate) fn advance_animation_groups(
    env: &crate::lua_api::env::WowLuaEnv,
    elapsed: f64,
) -> crate::Result<()> {
    let mut finished_scripts = Vec::new();
    let mut finished_animation_scripts = Vec::new();
    let mut loop_scripts = Vec::new();
    let mut sim = env.state().borrow_mut();
    let group_ids: Vec<u64> = sim.active_animation_groups.iter().copied().collect();
    for group_id in group_ids {
        let Some(result) = advance_animation_group(
            &mut sim,
            group_id,
            elapsed,
            &mut finished_scripts,
            &mut finished_animation_scripts,
        ) else {
            continue;
        };
        apply_animation_group_outcome(&mut sim, &result);
        sync_action_bar_busy_for_group(&mut sim, group_id);
        super::refresh_active_animation_group(&mut sim, group_id);
        for _ in 0..result.loop_count {
            loop_scripts.push(result.frame_id);
        }
    }
    drop(sim);

    fire_animation_group_scripts(env, "OnLoop", loop_scripts)?;
    fire_animation_group_scripts(env, "OnFinished", finished_animation_scripts)?;
    fire_animation_group_scripts(env, "OnFinished", finished_scripts)?;
    Ok(())
}

pub(crate) fn stop_animation_groups_for_hidden_subtree(
    sim: &mut crate::lua_api::state::SimState,
    root_id: u64,
) {
    let group_ids: Vec<u64> = sim
        .active_animation_groups
        .iter()
        .filter_map(|&group_id| {
            let group = sim.animation_groups.get(&group_id)?;
            let owner_is_hidden = frame_is_in_subtree(&sim.widgets, group.owner_frame_id, root_id);
            (animation_group_needs_stop(group) && owner_is_hidden).then_some(group_id)
        })
        .collect();

    for group_id in group_ids {
        if let Some(group) = sim.animation_groups.get_mut(&group_id) {
            stop_group(group);
        }
        apply_group_flipbook_state(sim, group_id);
        sync_action_bar_busy_for_group(sim, group_id);
        super::refresh_active_animation_group(sim, group_id);
    }
}

fn animation_group_needs_stop(group: &crate::lua_api::animation::AnimGroupState) -> bool {
    group.playing || group.paused || group.pending_finish
}

fn frame_is_in_subtree(
    widgets: &crate::widget::WidgetRegistry,
    frame_id: u64,
    root_id: u64,
) -> bool {
    let mut current_id = Some(frame_id);
    while let Some(frame_id) = current_id {
        if frame_id == root_id {
            return true;
        }
        current_id = widgets.get(frame_id).and_then(|frame| frame.parent_id);
    }
    false
}

fn stop_group(group: &mut crate::lua_api::animation::AnimGroupState) {
    group.playing = false;
    group.paused = false;
    group.done = true;
    group.pending_finish = false;
    group.elapsed = 0.0;
    for animation in &mut group.animations {
        animation.elapsed = 0.0;
    }
}

struct AnimationGroupAdvance {
    owner_id: u64,
    alpha_updates: Vec<AlphaUpdate>,
    flipbook_updates: Vec<FlipbookUpdate>,
    loop_count: u32,
    frame_id: u64,
}

struct AnimationGroupTargets {
    owner_id: u64,
    alpha_target_ids_by_animation: Vec<Option<u64>>,
    unique_alpha_target_ids: Vec<u64>,
}

struct AlphaUpdate {
    target_id: u64,
    pending_alpha: Option<f64>,
    restore_saved_alpha: Option<f32>,
}

struct FlipbookUpdate {
    child_key: Option<String>,
    frame_index: u32,
    rows: u32,
    columns: u32,
    frames: u32,
}

fn advance_animation_group(
    sim: &mut crate::lua_api::state::SimState,
    group_id: u64,
    elapsed: f64,
    finished_scripts: &mut Vec<u64>,
    finished_animation_scripts: &mut Vec<u64>,
) -> Option<AnimationGroupAdvance> {
    let targets = active_group_targets(sim, group_id)?;
    let saved_alphas = saved_alphas_for_targets(sim, &targets.unique_alpha_target_ids);
    let mut loop_count = 0u32;
    let (group_finished, alpha_updates, flipbook_updates, frame_id) = {
        let group = sim.animation_groups.get_mut(&group_id)?;
        advance_group_state(
            group,
            elapsed,
            &mut loop_count,
            &targets.unique_alpha_target_ids,
            &targets.alpha_target_ids_by_animation,
            &saved_alphas,
            finished_scripts,
        )
    };

    if group_finished {
        finished_animation_scripts.extend(animation_frame_ids_for_group(sim, group_id));
    }

    Some(AnimationGroupAdvance {
        owner_id: targets.owner_id,
        alpha_updates,
        flipbook_updates,
        loop_count,
        frame_id,
    })
}

fn active_group_targets(
    sim: &crate::lua_api::state::SimState,
    group_id: u64,
) -> Option<AnimationGroupTargets> {
    let group = sim.animation_groups.get(&group_id)?;
    // Most registered animation groups are idle; avoid any expensive
    // target-resolution work unless the group is actively ticking.
    if !group.playing || group.paused {
        return None;
    }

    let alpha_target_ids_by_animation = resolve_group_alpha_targets(sim, group);
    let unique_alpha_target_ids = unique_alpha_targets(&alpha_target_ids_by_animation);
    Some(AnimationGroupTargets {
        owner_id: group.owner_frame_id,
        alpha_target_ids_by_animation,
        unique_alpha_target_ids,
    })
}

fn saved_alphas_for_targets(
    sim: &crate::lua_api::state::SimState,
    target_ids: &[u64],
) -> std::collections::HashMap<u64, f32> {
    target_ids
        .iter()
        .copied()
        .map(|target_id| {
            let alpha = sim
                .widgets
                .get(target_id)
                .map(|frame| frame.alpha)
                .unwrap_or(1.0);
            (target_id, alpha)
        })
        .collect()
}

fn advance_group_state(
    group: &mut crate::lua_api::animation::AnimGroupState,
    elapsed: f64,
    loop_count: &mut u32,
    unique_alpha_target_ids: &[u64],
    alpha_target_ids_by_animation: &[Option<u64>],
    saved_alphas: &std::collections::HashMap<u64, f32>,
    finished_scripts: &mut Vec<u64>,
) -> (bool, Vec<AlphaUpdate>, Vec<FlipbookUpdate>, u64) {
    remember_saved_alphas(group, saved_alphas);
    let total_duration = current_group_total_duration(group);
    let group_finished =
        advance_group_playback(group, elapsed, total_duration, loop_count, finished_scripts);

    sync_animation_elapsed(group);
    let alpha_updates = collect_group_alpha_updates(
        group,
        unique_alpha_target_ids,
        alpha_target_ids_by_animation,
    );
    let flipbook_updates = collect_group_flipbook_updates(group);
    let frame_id = animation_group_frame_id(group);
    (group_finished, alpha_updates, flipbook_updates, frame_id)
}

fn remember_saved_alphas(
    group: &mut crate::lua_api::animation::AnimGroupState,
    saved_alphas: &std::collections::HashMap<u64, f32>,
) {
    for (&target_id, &saved_alpha) in saved_alphas {
        group.saved_alphas.entry(target_id).or_insert(saved_alpha);
    }
}

fn collect_group_alpha_updates(
    group: &crate::lua_api::animation::AnimGroupState,
    unique_alpha_target_ids: &[u64],
    alpha_target_ids_by_animation: &[Option<u64>],
) -> Vec<AlphaUpdate> {
    unique_alpha_target_ids
        .iter()
        .copied()
        .map(|target_id| group_alpha_update(group, target_id, alpha_target_ids_by_animation))
        .collect()
}

fn group_alpha_update(
    group: &crate::lua_api::animation::AnimGroupState,
    target_id: u64,
    alpha_target_ids_by_animation: &[Option<u64>],
) -> AlphaUpdate {
    let pending_alpha = group_current_alpha_for_target(
        group,
        group.elapsed,
        target_id,
        alpha_target_ids_by_animation,
    );
    let restore_saved_alpha = if !group.playing && group.done && !group.set_to_final_alpha {
        group.saved_alphas.get(&target_id).copied()
    } else {
        None
    };
    AlphaUpdate {
        target_id,
        pending_alpha,
        restore_saved_alpha,
    }
}

fn advance_group_playback(
    group: &mut crate::lua_api::animation::AnimGroupState,
    elapsed: f64,
    total_duration: f64,
    loop_count: &mut u32,
    finished_scripts: &mut Vec<u64>,
) -> bool {
    if group.pending_finish {
        finished_scripts.push(finish_group_now(group, total_duration));
        return true;
    }

    let was_done = group.done;
    advance_group_elapsed(group, elapsed, total_duration, loop_count, finished_scripts);
    !was_done && group.done && !group.playing
}

fn animation_frame_ids_for_group(sim: &crate::lua_api::state::SimState, group_id: u64) -> Vec<u64> {
    let mut frame_ids: Vec<(usize, u64)> = sim
        .anim_frame_to_anim
        .iter()
        .filter_map(|(&frame_id, &(mapped_group_id, animation_index))| {
            (mapped_group_id == group_id).then_some((animation_index, frame_id))
        })
        .collect();
    frame_ids.sort_unstable_by_key(|(animation_index, _)| *animation_index);
    frame_ids
        .into_iter()
        .map(|(_, frame_id)| frame_id)
        .collect()
}

fn advance_group_elapsed(
    group: &mut crate::lua_api::animation::AnimGroupState,
    elapsed: f64,
    total_duration: f64,
    loop_count: &mut u32,
    finished_scripts: &mut Vec<u64>,
) {
    let advance = elapsed * group.speed_multiplier.max(0.0);
    if group.reverse {
        group.elapsed -= advance;
    } else {
        group.elapsed += advance;
    }

    if total_duration <= 0.0 {
        finished_scripts.push(finish_group_now(group, total_duration));
        return;
    }

    match group.looping {
        crate::lua_api::animation::LoopType::None => {
            finish_unlooped_group_at_boundary(group, total_duration, finished_scripts)
        }
        crate::lua_api::animation::LoopType::Repeat => {
            wrap_repeating_group_elapsed(group, total_duration, loop_count);
        }
        crate::lua_api::animation::LoopType::Bounce => {
            bounce_group_elapsed_at_boundaries(group, total_duration, loop_count);
        }
    }
}

fn finish_unlooped_group_at_boundary(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
    finished_scripts: &mut Vec<u64>,
) {
    let finish_elapsed = if group.reverse && group.elapsed <= 0.0 {
        Some(0.0)
    } else if !group.reverse && group.elapsed >= total_duration {
        Some(total_duration)
    } else {
        None
    };

    if let Some(elapsed) = finish_elapsed {
        finished_scripts.push(finish_group_now(group, elapsed));
    }
}

fn wrap_repeating_group_elapsed(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
    loop_count: &mut u32,
) {
    if group.reverse {
        wrap_reverse_repeating_group(group, total_duration, loop_count);
    } else {
        wrap_forward_repeating_group(group, total_duration, loop_count);
    }
}

fn wrap_reverse_repeating_group(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
    loop_count: &mut u32,
) {
    while group.elapsed < 0.0 {
        group.elapsed += total_duration;
        *loop_count += 1;
    }
}

fn wrap_forward_repeating_group(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
    loop_count: &mut u32,
) {
    while group.elapsed >= total_duration {
        group.elapsed -= total_duration;
        *loop_count += 1;
    }
}

fn bounce_group_elapsed_at_boundaries(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
    loop_count: &mut u32,
) {
    while is_group_elapsed_outside_bounds(group.elapsed, total_duration) {
        reflect_bouncing_group_elapsed(group, total_duration);
        group.reverse = !group.reverse;
        *loop_count += 1;
    }
}

fn is_group_elapsed_outside_bounds(elapsed: f64, total_duration: f64) -> bool {
    elapsed >= total_duration || elapsed < 0.0
}

fn reflect_bouncing_group_elapsed(
    group: &mut crate::lua_api::animation::AnimGroupState,
    total_duration: f64,
) {
    if group.elapsed >= total_duration {
        group.elapsed = (group.elapsed - total_duration).max(0.0);
    } else {
        group.elapsed = (-group.elapsed).min(total_duration);
    }
}

fn apply_animation_group_outcome(
    sim: &mut crate::lua_api::state::SimState,
    result: &AnimationGroupAdvance,
) {
    // Alpha writes stay silent: propagate_effective_alpha below marks a frame
    // dirty only when its effective alpha changes, so a pulse under a
    // transparent parent does not force a redraw every tick.
    let mut changed_alpha_targets = Vec::new();
    for alpha_update in &result.alpha_updates {
        let mut changed = false;
        if let Some(alpha) = alpha_update.pending_alpha
            && let Some(frame) = sim.widgets.get_mut(alpha_update.target_id)
            && (frame.alpha as f64 - alpha).abs() > f32::EPSILON as f64
        {
            frame.alpha = alpha as f32;
            changed = true;
        }

        if let Some(saved_alpha) = alpha_update.restore_saved_alpha
            && let Some(frame) = sim.widgets.get_mut(alpha_update.target_id)
            && (frame.alpha - saved_alpha).abs() > f32::EPSILON
        {
            frame.alpha = saved_alpha;
            changed = true;
        }

        if changed {
            changed_alpha_targets.push(alpha_update.target_id);
        }
    }

    apply_group_flipbook_updates(sim, result.owner_id, &result.flipbook_updates);

    for target_id in changed_alpha_targets {
        let parent_effective_alpha = sim
            .widgets
            .get(target_id)
            .and_then(|frame| frame.parent_id)
            .and_then(|parent_id| sim.widgets.get(parent_id))
            .map(|parent| parent.effective_alpha)
            .unwrap_or(1.0_f32);
        sim.widgets
            .propagate_effective_alpha(target_id, parent_effective_alpha);
    }
}

fn fire_animation_group_scripts(
    env: &crate::lua_api::env::WowLuaEnv,
    handler_name: &str,
    frame_ids: Vec<u64>,
) -> crate::Result<()> {
    for frame_id in frame_ids {
        env.fire_script_handler(frame_id, handler_name, Vec::new())?;
    }
    Ok(())
}

fn resolve_group_alpha_targets(
    sim: &crate::lua_api::state::SimState,
    group: &crate::lua_api::animation::AnimGroupState,
) -> Vec<Option<u64>> {
    group
        .animations
        .iter()
        .map(|animation| {
            if !matches!(
                animation.anim_type,
                crate::lua_api::animation::AnimationType::Alpha
            ) {
                return None;
            }
            resolve_anim_target_id(sim, group.owner_frame_id, animation.child_key.as_deref())
        })
        .collect()
}

fn unique_alpha_targets(alpha_targets_by_animation: &[Option<u64>]) -> Vec<u64> {
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();
    for target_id in alpha_targets_by_animation.iter().flatten().copied() {
        if seen.insert(target_id) {
            unique.push(target_id);
        }
    }
    unique
}

fn group_current_alpha_for_target(
    group: &crate::lua_api::animation::AnimGroupState,
    elapsed: f64,
    target_id: u64,
    alpha_targets_by_animation: &[Option<u64>],
) -> Option<f64> {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (index, animation) in group.animations.iter().enumerate() {
        groups.entry(animation.order).or_default().push(index);
    }
    let mut remaining = elapsed;
    for (_order, animation_indices) in groups {
        let order_duration = animation_indices
            .iter()
            .map(|&index| group.animations[index].total_time())
            .fold(0.0, f64::max);
        let within_order = remaining.min(order_duration);
        let mut current = None;
        for &index in &animation_indices {
            if alpha_targets_by_animation.get(index).copied().flatten() != Some(target_id) {
                continue;
            }
            if let Some(alpha) = current_animation_alpha(&group.animations[index], within_order) {
                current = Some(alpha);
            }
        }
        if remaining <= order_duration {
            return current;
        }
        remaining -= order_duration;
    }
    None
}

fn current_animation_alpha(
    animation: &crate::lua_api::animation::AnimState,
    within_order: f64,
) -> Option<f64> {
    if !matches!(
        animation.anim_type,
        crate::lua_api::animation::AnimationType::Alpha
    ) {
        return None;
    }

    let start = animation.start_delay.max(0.0);
    let end = start + animation.duration.max(0.0);
    let alpha = if within_order <= start {
        animation.from_alpha
    } else if within_order >= end || animation.duration <= 0.0 {
        animation.to_alpha
    } else {
        let progress = ((within_order - start) / animation.duration).clamp(0.0, 1.0);
        animation.from_alpha + (animation.to_alpha - animation.from_alpha) * progress
    };
    Some(alpha)
}

pub(super) fn apply_group_flipbook_state(sim: &mut crate::lua_api::state::SimState, group_id: u64) {
    let Some(group) = sim.animation_groups.get(&group_id) else {
        return;
    };
    let owner_id = group.owner_frame_id;
    let updates = collect_group_flipbook_updates(group);
    apply_group_flipbook_updates(sim, owner_id, &updates);
}

fn collect_group_flipbook_updates(
    group: &crate::lua_api::animation::AnimGroupState,
) -> Vec<FlipbookUpdate> {
    group
        .animations
        .iter()
        .filter_map(|animation| {
            flipbook_frame_index(animation).map(|frame_index| FlipbookUpdate {
                child_key: animation.child_key.clone(),
                frame_index,
                rows: animation.flipbook_rows,
                columns: animation.flipbook_columns,
                frames: animation.flipbook_frames,
            })
        })
        .collect()
}

pub(super) fn sync_action_bar_busy_for_group(
    sim: &mut crate::lua_api::state::SimState,
    group_id: u64,
) {
    let Some(group) = sim.animation_groups.get(&group_id) else {
        return;
    };
    if is_override_action_bar_slideout(sim, group) {
        sim.action_bar_state.busy = group.playing;
    }
}

fn is_override_action_bar_slideout(
    sim: &crate::lua_api::state::SimState,
    group: &crate::lua_api::animation::AnimGroupState,
) -> bool {
    let owner_is_override_bar = sim
        .widgets
        .get(group.owner_frame_id)
        .and_then(|owner| owner.name.as_deref())
        == Some("OverrideActionBar");
    if !owner_is_override_bar {
        return false;
    }

    let Some(group_frame_id) = group.frame_id else {
        return false;
    };
    let owner_child_key_matches = sim
        .widgets
        .get(group.owner_frame_id)
        .and_then(|owner| owner.children_keys.get("slideOut"))
        .copied()
        == Some(group_frame_id);
    let group_parent_key_matches = sim
        .widgets
        .get(group_frame_id)
        .and_then(|frame| frame.parent_key.as_deref())
        == Some("slideOut");
    owner_child_key_matches || group_parent_key_matches
}

fn apply_group_flipbook_updates(
    sim: &mut crate::lua_api::state::SimState,
    owner_id: u64,
    updates: &[FlipbookUpdate],
) {
    for update in updates {
        let Some(target_id) = resolve_anim_target_id(sim, owner_id, update.child_key.as_deref())
        else {
            continue;
        };
        let Some(frame) = sim.widgets.get_mut_visual(target_id) else {
            continue;
        };
        if let Some(tex_coords) = flipbook_tex_coords(
            frame.atlas_tex_coords.or(frame.tex_coords),
            update.rows,
            update.columns,
            update.frames,
            update.frame_index,
        ) {
            frame.tex_coords = Some(tex_coords);
            frame.tex_coords_quad = None;
        }
    }
}

fn flipbook_frame_index(animation: &crate::lua_api::animation::AnimState) -> Option<u32> {
    if !matches!(
        animation.anim_type,
        crate::lua_api::animation::AnimationType::FlipBook
    ) {
        return None;
    }
    let frames = animation.flipbook_frames;
    let columns = animation.flipbook_columns;
    let rows = animation.flipbook_rows;
    if frames == 0 || columns == 0 || rows == 0 {
        return None;
    }
    let duration = animation.duration.max(0.0);
    let frame_index = if duration <= 0.0 {
        0
    } else {
        let progress = (animation.elapsed / duration).clamp(0.0, 1.0);
        let candidate = (progress * frames as f64).floor() as u32;
        candidate.min(frames.saturating_sub(1))
    };
    Some(frame_index)
}

fn flipbook_tex_coords(
    atlas_tex_coords: Option<(f32, f32, f32, f32)>,
    rows: u32,
    columns: u32,
    frames: u32,
    frame_index: u32,
) -> Option<(f32, f32, f32, f32)> {
    let (al, ar, at, ab) = atlas_tex_coords?;
    if rows == 0 || columns == 0 || frames == 0 {
        return None;
    }
    let frame_index = frame_index.min(frames.saturating_sub(1));
    let row = frame_index / columns;
    if row >= rows {
        return None;
    }
    let col = frame_index % columns;
    let width = (ar - al) / columns as f32;
    let height = (ab - at) / rows as f32;
    let left = al + col as f32 * width;
    let right = left + width;
    let top = at + row as f32 * height;
    let bottom = top + height;
    Some((left, right, top, bottom))
}

fn sync_animation_elapsed(group: &mut crate::lua_api::animation::AnimGroupState) {
    use std::collections::BTreeMap;

    let mut groups: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (index, animation) in group.animations.iter().enumerate() {
        groups.entry(animation.order).or_default().push(index);
    }

    let mut remaining = group.elapsed;
    for (_order, indices) in groups {
        let order_duration = indices
            .iter()
            .map(|&index| group.animations[index].total_time())
            .fold(0.0, f64::max);
        let within_order = remaining.min(order_duration);
        for index in indices {
            let animation = &mut group.animations[index];
            let start = animation.start_delay.max(0.0);
            let end = start + animation.duration.max(0.0);
            animation.elapsed = if within_order <= start {
                0.0
            } else if within_order >= end || animation.duration <= 0.0 {
                animation.duration.max(0.0)
            } else {
                (within_order - start).clamp(0.0, animation.duration.max(0.0))
            };
        }
        if remaining <= order_duration {
            break;
        }
        remaining -= order_duration;
    }
}
