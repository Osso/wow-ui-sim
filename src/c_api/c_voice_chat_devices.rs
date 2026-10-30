//! Supplied voice audio device catalogs for the settings dropdowns.
//! Retail, PTR and Forever VoiceChatDocumentation.lua declare both queries with one
//! nilable `VoiceAudioDevice` table result. No device enumeration or transport.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// One `VoiceAudioDevice` row; field names follow the generated documentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceAudioDevice {
    pub device_id: String,
    pub display_name: String,
    pub is_active: bool,
    pub is_system_default: bool,
    pub is_comms_default: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VoiceChat")?;
    table_set_rust_fn_static(state, namespace, "GetAvailableInputDevices", input_devices)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAvailableOutputDevices",
        output_devices,
    )
}

fn input_devices(state: &mut LuaState) -> LuaResult<u32> {
    let devices = borrow_state(state)?.voice_chat.input_devices.clone();
    push_devices(state, devices)
}

fn output_devices(state: &mut LuaState) -> LuaResult<u32> {
    let devices = borrow_state(state)?.voice_chat.output_devices.clone();
    push_devices(state, devices)
}

/// A supplied `None` answers the declared nil; each call returns a fresh array
/// because Audio.lua sorts the result in place.
fn push_devices(state: &mut LuaState, devices: Option<Vec<VoiceAudioDevice>>) -> LuaResult<u32> {
    let Some(devices) = devices else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let list = create_table(state);
    state.push(list);
    let Val::Table(array) = list else {
        unreachable!()
    };
    for (index, device) in devices.iter().enumerate() {
        let row = create_table(state);
        table_set_num(state, array, (index + 1) as f64, row);
        write_device(state, row, device);
    }
    Ok(1)
}

fn write_device(state: &mut LuaState, row: Val, device: &VoiceAudioDevice) {
    for (field, value) in [
        ("deviceID", &device.device_id),
        ("displayName", &device.display_name),
    ] {
        let value = create_string(state, value);
        table_set(state, row, field, value);
    }
    for (field, value) in [
        ("isActive", device.is_active),
        ("isSystemDefault", device.is_system_default),
        ("isCommsDefault", device.is_comms_default),
    ] {
        table_set(state, row, field, Val::Bool(value));
    }
}
