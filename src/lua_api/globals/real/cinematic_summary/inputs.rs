//! INFERRED simulator argument admission; native malformed-argument rules UNKNOWN.
//! Admit numeric types and representable movie IDs; reject other inputs neutrally.
use crate::lua_bridge::stack_val;
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn start_input(state: &LuaState) -> Option<(f64, u32)> {
    let (Val::Num(kind), Val::Num(id)) = (stack_val(state, 1), stack_val(state, 2)) else {
        return None;
    };
    (id >= 0.0 && id <= u32::MAX as f64 && id.fract() == 0.0).then_some((kind, id as u32))
}
