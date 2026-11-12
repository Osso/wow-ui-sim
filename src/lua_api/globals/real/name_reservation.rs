//! Forever Glue name-reservation status supplied by the feature-status input.

use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const GLOBAL_NAME: &str = "IsNameReservationOnly";

/// Apply the one-byte name-reservation value from Forever's Glue feature status.
pub fn set_feature_status(env: &WowLuaEnv, status_byte: u8) {
    env.state.borrow_mut().name_reservation_feature_status = status_byte;
}

pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    let mut lua = env.lua.borrow_mut();
    let registration = if is_glue {
        LuaApiMut::register_function(&mut *lua, GLOBAL_NAME, is_name_reservation_only)
    } else {
        LuaApiMut::set_global_val(&mut *lua, GLOBAL_NAME, Val::Nil)
    };
    registration.expect("register Forever Glue IsNameReservationOnly");
}

fn is_name_reservation_only(state: &mut LuaState) -> LuaResult<u32> {
    let status_byte = borrow_state(state)?.name_reservation_feature_status;
    state.push(Val::Bool(status_byte != 0));
    Ok(1)
}
