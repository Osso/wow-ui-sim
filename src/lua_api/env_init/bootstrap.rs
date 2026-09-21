//! Lua bootstrap strings executed during environment initialisation.
//!
//! The actual Lua sources live in sibling `.lua` files so this Rust file
//! stays small — see `shared_bootstrap.lua` for the pre-addon stubs and
//! `runtime_surface_bootstrap.lua` for the large post-enum runtime surface.

use rilua::{LuaApiMut, Val};

const SHARED_BOOTSTRAP_LUA: &str = include_str!("shared_bootstrap.lua");
const RUNTIME_SURFACE_BOOTSTRAP_LUA: &str = include_str!("runtime_surface_bootstrap.lua");

pub(crate) fn init_shared_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(feature = "forbidden-aspects")]
    crate::lua_api::script_object_transfer::register_delegate_projection(lua)?;
    let bootstrap = lua.load_bytes(SHARED_BOOTSTRAP_LUA.as_bytes(), "@shared-bootstrap")?;
    lua.call_function(
        &bootstrap,
        &[Val::Bool(cfg!(feature = "forbidden-aspects"))],
    )?;
    #[cfg(feature = "client-wowforever")]
    lua.exec(include_str!("texture_metatable.lua"))?;
    Ok(())
}

pub(crate) fn init_runtime_surface_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(RUNTIME_SURFACE_BOOTSTRAP_LUA)?;
    Ok(())
}
