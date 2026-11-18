//! Retail 12.1.5 legacy PerformanceBar globals and protocol input.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolCode {
    Unsupported,
    One,
    Two,
}

impl ProtocolCode {
    pub fn value(self) -> u8 {
        match self {
            Self::Unsupported => 0,
            Self::One => 1,
            Self::Two => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PerformanceProtocolTypes {
    pub home: ProtocolCode,
    pub world: ProtocolCode,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        state.global,
        "GetAvailableBandwidth",
        available_bandwidth,
    )?;
    table_set_rust_fn_static(state, state.global, "GetProtocolTypes", protocol_types)
}

/// VERIFIED R88ap.7: the normal path publishes one number with value zero.
fn available_bandwidth(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

/// VERIFIED R88ap.8: publish two numeric endpoint codes in order; absent input is (0, 0).
fn protocol_types(state: &mut LuaState) -> LuaResult<u32> {
    let input = borrow_state(state)?.performance_bar_protocol_types;
    let (home, world) = input.map_or((0, 0), |types| (types.home.value(), types.world.value()));
    state.push(Val::Num(f64::from(home)));
    state.push(Val::Num(f64::from(world)));
    Ok(2)
}
