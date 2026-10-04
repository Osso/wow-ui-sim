use crate::lua_api::methods::borrow_state_mut;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(super) fn imply_mouse_enabled_for_mouse_handler(
    state: &LuaState,
    frame_id: u64,
    handler_name: &str,
) -> LuaResult<()> {
    if !script_implies_enable_mouse(handler_name) {
        return Ok(());
    }
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut(frame_id) else {
        return Ok(());
    };
    if frame.mouse_enabled && frame.mouse_motion_enabled {
        return Ok(());
    }

    frame.set_mouse_enabled(true);
    sim.queue_hit_grid_eligibility_change(frame_id);
    Ok(())
}

fn script_implies_enable_mouse(handler_name: &str) -> bool {
    matches!(
        handler_name,
        "OnEnter" | "OnLeave" | "OnMouseDown" | "OnMouseUp"
    )
}
