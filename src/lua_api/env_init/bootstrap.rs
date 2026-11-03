//! Lua bootstrap strings executed during environment initialisation.
//!
//! The actual Lua sources live in sibling `.lua` files so this Rust file
//! stays small — see `shared_bootstrap.lua` for the pre-addon stubs and
//! `runtime_surface_bootstrap.lua` for the large post-enum runtime surface.

use rilua::{LuaApiMut, Val};

const SHARED_BOOTSTRAP_LUA: &str = include_str!("shared_bootstrap.lua");
const RUNTIME_SURFACE_BOOTSTRAP_LUA: &str = include_str!("runtime_surface_bootstrap.lua");

pub(crate) fn init_shared_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(
        lua,
        "__wow_internal_projection_call",
        secure_projection_call,
    )?;
    #[cfg(feature = "forbidden-aspects")]
    crate::lua_api::script_object_transfer::register_delegate_projection(lua)?;
    let bootstrap = lua.load_bytes(SHARED_BOOTSTRAP_LUA.as_bytes(), "@shared-bootstrap")?;
    let secure_projection = LuaApiMut::get_global_val(lua, "__wow_internal_projection_call");
    lua.call_function(
        &bootstrap,
        &[
            Val::Bool(cfg!(feature = "forbidden-aspects")),
            Val::Bool(cfg!(feature = "retail-12-0-0")),
            secure_projection,
        ],
    )?;
    LuaApiMut::set_global_val(lua, "__wow_internal_projection_call", Val::Nil)?;
    #[cfg(any(feature = "retail-12-1-5", feature = "client-wowforever"))]
    lua.exec(include_str!("texture_metatable.lua"))?;
    Ok(())
}

fn secure_projection_call(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    if state.base >= state.top {
        return Ok(0);
    }
    let function = state.base;
    rilua::stdlib::taint::call_with_stack_taint(
        state,
        function,
        rilua::vm::callinfo::LUA_MULTRET,
        None,
    )?;
    Ok(state.top.saturating_sub(function) as u32)
}

pub(crate) fn init_runtime_surface_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    let classic_combat = matches!(
        ACTIVE,
        ClientProfile::Mists
            | ClientProfile::Era
            | ClientProfile::Anniversary
            | ClientProfile::WowForever
    );
    let bootstrap = lua.load_bytes(
        RUNTIME_SURFACE_BOOTSTRAP_LUA.as_bytes(),
        "@runtime_surface_bootstrap.lua",
    )?;
    lua.call_function(
        &bootstrap,
        &[
            Val::Bool(classic_combat),
            Val::Bool(ACTIVE == ClientProfile::WowForever),
            Val::Bool(matches!(
                ACTIVE,
                ClientProfile::Era | ClientProfile::Anniversary
            )),
        ],
    )?;
    Ok(())
}
