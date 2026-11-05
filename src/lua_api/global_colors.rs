//! Named color constants for Rust producers.
//!
//! Color constants such as `NORMAL_FONT_COLOR`, `ERROR_COLOR` or
//! `ITEM_ARTIFACT_COLOR` are not all present in the raw global table.
//! `runtime_surface_bootstrap.lua` synthesizes the unmodelled `*_COLOR`
//! constants through the `_G` metatable `__index`, so a raw table read misses
//! them while Lua code sees them. Rust producers resolve colors through the
//! same `_G` index chain, which keeps global indexing the single source of
//! truth for named color constants.

use crate::lua_api::methods::table_get;
use crate::lua_api::script_helpers::collect_lua_error;
use rilua::Val;
use rilua::vm::state::LuaState;

/// `_G[name]` with Lua `__index` semantics.
pub(crate) fn global_value(state: &mut LuaState, name: &str) -> Val {
    let key = Val::Str(state.gc.intern_string(name.as_bytes()));
    let globals = Val::Table(state.global);
    match state.gettable(globals, key) {
        Ok(value) => value,
        Err(error) => {
            collect_lua_error(state, &format!("global lookup of {name} failed: {error}"));
            Val::Nil
        }
    }
}

/// RGB channels of a named color constant, or `None` when it resolves to nil
/// or is not a color table.
pub(crate) fn global_color_rgb(state: &mut LuaState, name: &str) -> Option<(f64, f64, f64)> {
    let color = global_value(state, name);
    Some((
        color_component(state, color, "r")?,
        color_component(state, color, "g")?,
        color_component(state, color, "b")?,
    ))
}

/// RGBA channels of a named color constant; a missing alpha reads as 1.
pub(crate) fn global_color_rgba(state: &mut LuaState, name: &str) -> Option<(f64, f64, f64, f64)> {
    let color = global_value(state, name);
    let (r, g, b) = (
        color_component(state, color, "r")?,
        color_component(state, color, "g")?,
        color_component(state, color, "b")?,
    );
    Some((r, g, b, color_component(state, color, "a").unwrap_or(1.0)))
}

fn color_component(state: &mut LuaState, color: Val, channel: &str) -> Option<f64> {
    match table_get(state, color, channel) {
        Val::Num(value) => Some(value),
        _ => None,
    }
}
