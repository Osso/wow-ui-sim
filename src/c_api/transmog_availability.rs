//! Supplied transmog availability; no native default is inferred.
//! Declarations: TransmogOutfitInfoDocumentation.lua, Retail 70448 / PTR 70493
//! lines 631–638; Mists 70268 lines 586–593; Forever 70532 lines 631–638.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Host-supplied availability for one environment.
#[derive(Debug, Default)]
pub struct TransmogAvailability {
    pub enabled: Option<bool>,
}

impl TransmogAvailability {
    pub(crate) fn require_enabled(&self) -> LuaResult<bool> {
        self.enabled.ok_or_else(|| {
            runtime_error(
                "C_TransmogOutfitInfo.IsTransmogEnabled: transmog availability has not been supplied",
            )
        })
    }
}

fn publishes_availability() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Retail
            | ClientProfile::Ptr
            | ClientProfile::Mists
            | ClientProfile::WowForever
    )
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_availability() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_TransmogOutfitInfo")?;
    table_set_rust_fn_static(state, namespace, "IsTransmogEnabled", is_transmog_enabled)
}

/// `A_Admin.SetTransmogAvailability(enabled)`: supplies the observation.
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_availability() {
        return Ok(builder);
    }
    builder.set_function("SetTransmogAvailability", set_transmog_availability)
}

fn set_transmog_availability(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(runtime_error(
            "A_Admin.SetTransmogAvailability: enabled must be a boolean",
        ));
    };
    borrow_state_mut(state)?.transmog_availability.enabled = Some(enabled);
    Ok(0)
}

fn is_transmog_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .transmog_availability
        .require_enabled()?;
    state.push(Val::Bool(enabled));
    Ok(1)
}
