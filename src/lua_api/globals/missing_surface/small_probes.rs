//! Small opaque-id and focus-stack probes.
//!
//! Migrates 2 entries off the namespace stub tables:
//!
//! - `C_Timer.NewTimerID()` — returns a fresh monotonically-increasing
//!   opaque number using the same per-environment `next_timer_id()` counter
//!   shared by `C_Timer.NewTimer` / `NewTicker`. Replaces the `stub_nil` stub.
//!
//! - `C_System.GetFrameStack()` — returns an array of frames currently
//!   under the mouse cursor. When `SimState.hovered_frame` is set the
//!   array is `{hovered_frame}`; otherwise an empty array is returned.
//!   Replaces the `stub_nil` stub.
//!
//! `C_AddOnProfiler.CheckForPerformanceMessage` is already fully
//! implemented in `utility_system_spell/c_addon_profiler.rs`; this
//! module does not touch it.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_table, frame_ref};
use crate::lua_api::next_timer_id;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register_small_probes_surface(state: &mut LuaState) -> LuaResult<()> {
    let c_timer = ensure_namespace(state, "C_Timer")?;
    table_set_rust_fn_static(state, c_timer, "NewTimerID", c_timer_new_timer_id)?;

    let c_system = ensure_namespace(state, "C_System")?;
    table_set_rust_fn_static(state, c_system, "GetFrameStack", c_system_get_frame_stack)?;

    Ok(())
}

/// `C_Timer.NewTimerID()` — hand out a fresh opaque timer id.
fn c_timer_new_timer_id(state: &mut LuaState) -> LuaResult<u32> {
    let id = next_timer_id(state)?;
    state.push(Val::Num(id as f64));
    Ok(1)
}

/// `C_System.GetFrameStack()` — array of frames under the mouse cursor.
///
/// Returns `{hovered_frame}` when `SimState.hovered_frame` is set,
/// or an empty array when no frame is hovered.
fn c_system_get_frame_stack(state: &mut LuaState) -> LuaResult<u32> {
    let hovered_id = borrow_state(state)?.hovered_frame;
    let array = create_table(state);
    if let Some(id) = hovered_id {
        let frame = frame_ref(state, id)?;
        set_table_array(state, array, 1, frame);
    }
    state.push(array);
    Ok(1)
}
