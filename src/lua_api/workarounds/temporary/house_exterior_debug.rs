//! Temporary publication-only fixture diagnostics. Neither cached
//! HouseExteriorUIDocumentation.lua nor cached consumers retain these two
//! signatures/payloads. INFERRED: nil means unavailable debug data; not native
//! arity/payload parity. Retire when a native fixture-debug contract and a
//! GUID-indexed/selected fixture diagnostic model are available. Do not make
//! this permanent merely because fixture meshes are outside 2D rendering.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::{LuaApiMut, LuaResult, Val, vm::state::LuaState};

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if cfg!(feature = "retail-12-0-0") {
        let state = lua.state_mut();
        let namespace = ensure_namespace(state, "C_HouseExterior")?;
        for name in ["GetFixtureDebugInfoForGUID", "GetSelectedFixtureDebugInfo"] {
            table_set_rust_fn_static(state, namespace, name, unavailable_debug_info)?;
        }
    }
    Ok(())
}

fn unavailable_debug_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}
