//! Resolve animation-group ownership from group or animation frame IDs.

pub(in super::super) fn resolve_animation_group_id(
    sim: &crate::lua_api::SimState,
    frame_id: u64,
) -> Option<u64> {
    sim.anim_frame_to_group.get(&frame_id).copied().or_else(|| {
        sim.anim_frame_to_anim
            .get(&frame_id)
            .map(|(group_id, _)| *group_id)
    })
}
