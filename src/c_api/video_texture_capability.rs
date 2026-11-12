//! Read-only high-resolution texture capability.
//!
//! Native answer tasklt-a667#100 (R60) verifies registration in Game and Glue
//! for PTR 70668 and Forever 70707, and absence from Retail 70688. The handler
//! reads no arguments and projects a capability byte initialized to zero in
//! both inspected images. Keeping that byte at zero in the simulator is an
//! inferred model, not a measurement of installed texture packs. Ordinary
//! extra-argument handling and the single-boolean return convention are inferred.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const HIGH_RES_TEXTURE_CAPABILITY: u8 = 0;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VideoOptions")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "AreHighResTexturesAvailable",
        are_high_res_textures_available,
    )
}

fn are_high_res_textures_available(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(HIGH_RES_TEXTURE_CAPABILITY != 0));
    Ok(1)
}
