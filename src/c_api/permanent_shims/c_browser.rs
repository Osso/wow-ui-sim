//! Permanent `C_Browser` surface.
//!
//! The simulator embeds no web browser, so there is never a fullscreen browser
//! to close. `CloseFullscreenBrowser` is accepted as a no-op so the fullscreen
//! browser spinner's Escape handler can run; the spinner hides itself.

use crate::c_api::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_browser_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Browser")?;
    table_set_rust_fn_static(
        state,
        ns,
        "CloseFullscreenBrowser",
        close_fullscreen_browser,
    )
}

fn close_fullscreen_browser(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
