//! Owner-serviced UI lifecycle requests, shared with the legacy ReloadUI global.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_ref};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UI")?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    {
        // UIDocumentation: byte-identical Retail/Forever blocks. Original consumers:
        // Blizzard_PingUI and Blizzard_RadialWheel (InputUtil.GetCursorPosition).
        table_set_rust_fn_static(state, namespace, "GetUIParent", |state| {
            push_named_frame(state, "UIParent")
        })?;
        table_set_rust_fn_static(state, namespace, "GetWorldFrame", |state| {
            push_named_frame(state, "WorldFrame")
        })?;
    }
    table_set_rust_fn_static(state, namespace, "Reload", request_reload)
}

/// The engine-owned frame registered under `name`, independent of the global.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn push_named_frame(state: &mut LuaState, name: &str) -> LuaResult<u32> {
    let id = borrow_state(state)?
        .widgets
        .get_id_by_name(name)
        .ok_or_else(|| runtime_error(format!("C_UI: {name} does not exist")))?;
    let frame = frame_ref(state, id)?;
    state.push(frame);
    Ok(1)
}

pub(crate) fn request_reload(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.ui_reload_requested = true;
    Ok(0)
}
