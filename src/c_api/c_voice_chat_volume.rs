//! Voice ducking scale backed by the existing voice-chat model.
//! Valid settings-slider inputs are normalized to [0, 1]. Invalid-input rejection
//! preserves the model invariant; native out-of-range behavior is not claimed.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetMasterVolumeScale",
        set_master_volume_scale,
    )
}

fn set_master_volume_scale(state: &mut LuaState) -> LuaResult<u32> {
    let scale = match stack_val(state, 1) {
        Val::Num(scale) if (0.0..=1.0).contains(&scale) => scale,
        _ => {
            return Err(runtime_error(
                "C_VoiceChat.SetMasterVolumeScale: scale must be a number in [0, 1]",
            ));
        }
    };
    borrow_state_mut(state)?.voice_chat.master_volume_scale = scale;
    Ok(0)
}
