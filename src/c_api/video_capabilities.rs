//! R63 video capability inputs, separate from high-resolution textures.
//!
//! Native answer tasklt-a667#110 verifies all five members in Game and Glue on
//! PTR 70668 and Forever 70707. Retail 70688 registers only spell density; the
//! other four exact members are absent, while dynamic aliases remain UNKNOWN.
//! The inspected image bytes are PTR (0, 0, 0, 0, 1), Forever (1, 1, 1, 1, 1),
//! and Retail spell density 1, in the field order below. Holding those image
//! values as environment defaults is INFERRED policy, not a live capability
//! measurement. Ordinary boolean projections are INFERRED: nonzero except for
//! the inverted outline-disabled byte. Forever's no-argument boolean shape is
//! verified; PTR return tails remain UNKNOWN. Other argument behavior is not
//! claimed. These bytes are neither CVars nor renderer/device probes.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Explicit, environment-local video capability inputs. MSAA is a runtime/device
/// capability separate from R60 high-resolution textures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VideoCapabilityBytes {
    pub linear_enabled_on_start: u8,
    pub outline_disabled: u8,
    pub pbr_water_supported: u8,
    pub secondary_lighting_supported: u8,
    pub spell_visual_density_supported: u8,
    pub multisample_anti_aliasing_supported: bool,
    pub anti_aliasing_supported: bool,
}

impl Default for VideoCapabilityBytes {
    fn default() -> Self {
        let forever = u8::from(ACTIVE == ClientProfile::WowForever);
        Self {
            linear_enabled_on_start: forever,
            outline_disabled: forever,
            pbr_water_supported: forever,
            secondary_lighting_supported: forever,
            spell_visual_density_supported: u8::from(publishes_capabilities()),
            multisample_anti_aliasing_supported: false,
            anti_aliasing_supported: false,
        }
        // Unpublished fields are inert zero host inputs, not native evidence.
    }
}

pub(crate) fn publishes_capabilities() -> bool {
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !publishes_capabilities() {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VideoOptions")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsSpellVisualDensitySystemSupported",
        is_spell_visual_density_system_supported,
    )?;
    if ACTIVE == ClientProfile::Retail {
        return Ok(());
    }
    if ACTIVE == ClientProfile::WowForever {
        table_set_rust_fn_static(
            state,
            state.global,
            "MultiSampleAntiAliasingSupported",
            is_multisample_anti_aliasing_supported,
        )?;
        table_set_rust_fn_static(
            state,
            state.global,
            "AntiAliasingSupported",
            is_anti_aliasing_supported,
        )?;
    }
    table_set_rust_fn_static(
        state,
        namespace,
        "IsLinearEnabledOnStart",
        is_linear_enabled_on_start,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsOutlineModeSupported",
        is_outline_mode_supported,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsPBRWaterSupported",
        is_pbr_water_supported,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsSecondaryLightingSupported",
        is_secondary_lighting_supported,
    )
}

fn is_linear_enabled_on_start(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .video_capability_bytes
        .linear_enabled_on_start
        != 0;
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_outline_mode_supported(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.video_capability_bytes.outline_disabled == 0;
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_pbr_water_supported(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .video_capability_bytes
        .pbr_water_supported
        != 0;
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_secondary_lighting_supported(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .video_capability_bytes
        .secondary_lighting_supported
        != 0;
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let supported = borrow_state(state)?
        .video_capability_bytes
        .anti_aliasing_supported;
    state.push(Val::Bool(supported));
    Ok(1)
}

fn is_multisample_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let supported = borrow_state(state)?
        .video_capability_bytes
        .multisample_anti_aliasing_supported;
    state.push(Val::Bool(supported));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE == ClientProfile::WowForever {
        builder
            .set_function(
                "SetMultiSampleAntiAliasingSupported",
                set_multisample_anti_aliasing_supported,
            )?
            .set_function("SetAntiAliasingSupported", set_anti_aliasing_supported)
    } else {
        Ok(builder)
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let supported = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .video_capability_bytes
        .anti_aliasing_supported = supported;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_multisample_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let supported = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .video_capability_bytes
        .multisample_anti_aliasing_supported = supported;
    Ok(0)
}

fn is_spell_visual_density_system_supported(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .video_capability_bytes
        .spell_visual_density_supported
        != 0;
    state.push(Val::Bool(value));
    Ok(1)
}
