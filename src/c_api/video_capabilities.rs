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
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::methods::{borrow_state, create_string};
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::methods::{table_get_static, val_to_string};
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use rilua::{LuaError, RuntimeError};
use rilua::{LuaResult, Val};

/// A supported Forever multisample antialiasing option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultisampleAntialiasingGroup {
    pub msaa_quality: String,
    pub sample_count: i32,
    pub coverage_count: i32,
}

/// Explicit, environment-local video capability inputs. MSAA is a runtime/device
/// capability separate from R60 high-resolution textures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VideoCapabilityBytes {
    pub linear_enabled_on_start: u8,
    pub outline_disabled: u8,
    pub pbr_water_supported: u8,
    pub secondary_lighting_supported: u8,
    pub spell_visual_density_supported: u8,
    pub multisample_anti_aliasing_groups: Vec<MultisampleAntialiasingGroup>,
    pub cmaa_supported: bool,
    pub cmaa2_supported: bool,
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
            multisample_anti_aliasing_groups: Vec::new(),
            cmaa_supported: false,
            cmaa2_supported: false,
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
    register_spell_visual_density(state, namespace)?;
    if ACTIVE == ClientProfile::Retail {
        return Ok(());
    }
    register_legacy_antialiasing_globals(state)?;
    register_video_capability_queries(state, namespace)
}

fn register_spell_visual_density(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "IsSpellVisualDensitySystemSupported",
        is_spell_visual_density_system_supported,
    )
}

fn register_legacy_antialiasing_globals(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
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
    )
}

fn register_video_capability_queries(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
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
    let (cmaa_supported, cmaa2_supported) = {
        let capabilities = borrow_state(state)?;
        let inputs = &capabilities.video_capability_bytes;
        (inputs.cmaa_supported, inputs.cmaa2_supported)
    };
    state.push(Val::Bool(true));
    state.push(Val::Bool(cmaa_supported));
    state.push(Val::Bool(cmaa2_supported));
    Ok(3)
}

fn is_multisample_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let groups = borrow_state(state)?
        .video_capability_bytes
        .multisample_anti_aliasing_groups
        .clone();
    let result_count = (groups.len() * 3) as u32;
    for group in &groups {
        let quality = create_string(state, &group.msaa_quality);
        state.push(quality);
        state.push(Val::Num(f64::from(group.sample_count)));
        state.push(Val::Num(f64::from(group.coverage_count)));
    }
    Ok(result_count)
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
    let cmaa_supported = bool::from_stack(state, 1)?;
    let cmaa2_supported = bool::from_stack(state, 2)?;
    let mut simulator = borrow_state_mut(state)?;
    let inputs = &mut simulator.video_capability_bytes;
    inputs.cmaa_supported = cmaa_supported;
    inputs.cmaa2_supported = cmaa2_supported;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_multisample_anti_aliasing_supported(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Table(groups_table) = Val::from_stack(state, 1)? else {
        return Err(input_error(
            "SetMultiSampleAntiAliasingSupported expects an array of groups",
        ));
    };
    let group_values = state
        .gc
        .tables
        .get(groups_table)
        .ok_or_else(|| {
            input_error("SetMultiSampleAntiAliasingSupported received an invalid table")
        })?
        .array_slice()
        .iter()
        .copied()
        .take_while(|value| !matches!(value, Val::Nil))
        .collect::<Vec<_>>();
    let groups = group_values
        .into_iter()
        .map(|group| read_multisample_group(state, group))
        .collect::<LuaResult<Vec<_>>>()?;
    borrow_state_mut(state)?
        .video_capability_bytes
        .multisample_anti_aliasing_groups = groups;
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_multisample_group(
    state: &mut LuaState,
    group: Val,
) -> LuaResult<MultisampleAntialiasingGroup> {
    let quality_value = table_get_static(state, group, "msaaQuality");
    let msaa_quality = val_to_string(state, quality_value)
        .ok_or_else(|| input_error("MSAA group msaaQuality must be a string"))?;
    let sample_count = read_group_integer(state, group, "sampleCount")?;
    let coverage_count = read_group_integer(state, group, "coverageCount")?;
    Ok(MultisampleAntialiasingGroup {
        msaa_quality,
        sample_count,
        coverage_count,
    })
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_group_integer(state: &mut LuaState, group: Val, key: &'static str) -> LuaResult<i32> {
    let value = table_get_static(state, group, key);
    let Val::Num(value) = value else {
        return Err(input_error(format!("MSAA group {key} must be an integer")));
    };
    if !is_i32_input(value) {
        return Err(input_error(format!("MSAA group {key} must be an integer")));
    }
    Ok(value as i32)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_i32_input(value: f64) -> bool {
    if !value.is_finite() {
        return false;
    }
    if value.fract() != 0.0 {
        return false;
    }
    value >= f64::from(i32::MIN) && value <= f64::from(i32::MAX)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn input_error(message: impl Into<String>) -> LuaError {
    LuaError::Runtime(RuntimeError {
        message: message.into(),
        level: 1,
        traceback: vec![],
    })
}

fn is_spell_visual_density_system_supported(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .video_capability_bytes
        .spell_visual_density_supported
        != 0;
    state.push(Val::Bool(value));
    Ok(1)
}
