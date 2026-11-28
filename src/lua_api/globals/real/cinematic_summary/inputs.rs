//! INFERRED simulator argument admission; native malformed-argument rules UNKNOWN.
//! Admit numeric types and representable movie IDs; reject other inputs neutrally.
use crate::lua_api::methods::{borrow_state_mut, val_to_string};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn start_input(state: &LuaState) -> Option<(f64, u32)> {
    let (Val::Num(kind), Val::Num(id)) = (stack_val(state, 1), stack_val(state, 2)) else {
        return None;
    };
    (id >= 0.0 && id <= u32::MAX as f64 && id.fract() == 0.0).then_some((kind, id as u32))
}

/// `A_Admin.SetCinematicSummary(movieID, summary|nil)`: supplies one movie row; nil
/// records a row without a summary reference.
pub(crate) fn admin_set_cinematic_summary(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let summary = match stack_val(state, 2) {
        Val::Nil => None,
        value @ Val::Str(_) => val_to_string(state, value),
        _ => {
            return Err(runtime_error(
                "A_Admin.SetCinematicSummary expects a summary string or nil",
            ));
        }
    };
    borrow_state_mut(state)?
        .cinematic_summaries
        .insert(id, summary);
    Ok(0)
}
