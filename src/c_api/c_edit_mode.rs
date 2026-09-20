//! Profile defaults, separate from saved Edit Mode layout selection.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Initial Forever preset observed in the supplied 1.60.1.69913 capture.
/// Input-style changes do not change this modeled default.
const DEFAULT_LAYOUT_MODERN: u32 = 0;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_EditMode")?;
    table_set_rust_fn_static(state, namespace, "GetEditModeDefaultLayout", default_layout)
}

fn default_layout(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(DEFAULT_LAYOUT_MODERN as f64));
    Ok(1)
}
