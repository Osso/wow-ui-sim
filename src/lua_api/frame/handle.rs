//! LightUserData-based frame representation helpers.
//!
//! Frames are represented as LightUserData with the frame ID encoded as a pointer.
//! SimState is stored in Lua app_data, accessed via `get_sim_state()`.

use crate::lua_api::SimState;
use mlua::{LightUserData, Lua, Value};
use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

/// Retrieve the shared SimState from Lua app_data.
#[inline]
pub fn get_sim_state(lua: &Lua) -> Rc<RefCell<SimState>> {
    lua.app_data_ref::<Rc<RefCell<SimState>>>()
        .expect("SimState not set in Lua app_data")
        .clone()
}

/// Create a LightUserData Value from a frame ID.
#[inline]
pub fn frame_lud(id: u64) -> Value {
    Value::LightUserData(LightUserData((id as usize) as *mut c_void))
}

/// Extract a frame ID from a Lua Value (LightUserData).
#[inline]
pub fn extract_frame_id(value: &Value) -> Option<u64> {
    match value {
        Value::LightUserData(lud) => Some(lud.0 as u64),
        _ => None,
    }
}

/// Extract a frame ID from a LightUserData directly.
#[inline]
pub fn lud_to_id(lud: LightUserData) -> u64 {
    lud.0 as u64
}
