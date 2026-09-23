//! MovieFrame subtitle preference, independent of unsupported playback.

use crate::lua_api::methods::{borrow_state_mut, frame_id_from_stack};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

fn enable_subtitles(state: &mut LuaState) -> LuaResult<u32> {
    let id = frame_id_from_stack(state, 1)?;
    let enabled = match stack_val(state, 2) {
        Val::Bool(value) => value,
        value => {
            return Err(runtime_error(format!(
                "EnableSubtitles expected boolean at argument 2, got {}",
                value.type_name()
            )));
        }
    };
    let mut sim = borrow_state_mut(state)?;
    let frame = sim
        .widgets
        .get_mut(id)
        .ok_or_else(|| runtime_error("EnableSubtitles requires a MovieFrame"))?;
    if frame.object_type_name.as_deref() != Some("MovieFrame") {
        return Err(runtime_error("EnableSubtitles requires a MovieFrame"));
    }
    frame.movie_subtitles_enabled = enabled;
    Ok(0)
}

pub(super) fn register_movie(state: &mut LuaState, metatable: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, metatable, "EnableSubtitles", enable_subtitles)
}
