//! Retail and Forever camera values used by Blizzard Settings Graphics.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetCameraFOVDefaults", get_camera_fov_defaults)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCameraFOV", set_camera_fov)
}

// R88fh VERIFIED: Retail and Forever publish (90, 50, 90).
fn get_camera_fov_defaults(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(90.0));
    state.push(Val::Num(50.0));
    state.push(Val::Num(90.0));
    Ok(3)
}

fn set_camera_fov(state: &mut LuaState) -> LuaResult<u32> {
    let value = f64::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .cvars
        .set_temporary("cameraFov", &value.to_string());
    Ok(0)
}
