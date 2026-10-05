//! INFERRED host grants for the four declared input kinds; no event delivery bypass.
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_LimitedInput")?;
    table_set_rust_fn_static(state, ns, "LimitedInputAllowed", allowed)
}
fn allowed(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let kind = match value {
        Val::Num(n) if (0.0..=3.0).contains(&n) && n.fract() == 0.0 => n as usize,
        _ => {
            return Err(rilua::runtime_error(
                "LimitedInputType must be MouseMove (0), MouseDown (1), MouseUp (2), or MouseWheel (3)",
            ));
        }
    };
    let allowed = borrow_state(state)?.limited_input_allowed[kind];
    state.push(Val::Bool(allowed));
    Ok(1)
}
