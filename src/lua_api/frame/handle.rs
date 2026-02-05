//! FrameHandle userdata for Lua-accessible WoW frames.

use crate::lua_api::SimState;
use std::cell::RefCell;
use std::rc::Rc;

/// Userdata handle to a frame (passed to Lua).
#[derive(Clone)]
pub struct FrameHandle {
    pub id: u64,
    pub state: Rc<RefCell<SimState>>,
}
