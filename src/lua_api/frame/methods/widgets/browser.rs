//! Browser widget navigation methods.
//!
//! The simulator has no embedded browser engine. Navigation methods preserve
//! WoW's callable, no-result API without opening external content.

use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(super) fn register_browser(state: &mut LuaState, mt: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, mt, "NavigateHome", navigate)?;
    // The 11.2.0 page lists NavigateTo as removed, but current retail
    // Blizzard_PhotoSharingBrowser.lua:66,73 still calls it; keep it published.
    table_set_rust_fn_static(state, mt, "NavigateTo", navigate)?;
    Ok(())
}

fn navigate(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
