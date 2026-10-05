//! Per-type configuration only; no world plate bounds or hit testing exist.

use super::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// INFERRED configuration only. Host camera snapshots do not render a 3D world.
#[derive(Debug, Default)]
pub struct NamePlateConfiguration {
    pub size: [f64; 2],
    pub behind_camera: std::collections::HashSet<String>,
    pub simplified: std::collections::HashSet<String>,
}

#[cfg(feature = "retail-12-0-0")]
pub(super) fn register_configuration(state: &mut LuaState) -> LuaResult<()> {
    let plates = ensure_namespace(state, "C_NamePlate")?;
    table_set_rust_fn_static(state, plates, "SetNamePlateSize", set_size)?;
    table_set_rust_fn_static(state, plates, "GetNamePlateSize", |s| {
        let size = borrow_state(s)?.nameplate_configuration.size;
        for value in size {
            s.push(Val::Num(value));
        }
        Ok(2)
    })?;
    let manager = ensure_namespace(state, "C_NamePlateManager")?;
    table_set_rust_fn_static(state, manager, "IsNamePlateUnitBehindCamera", |s| {
        let token = read_token(s)?;
        let value = borrow_state(s)?
            .nameplate_configuration
            .behind_camera
            .contains(&token);
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, manager, "SetNamePlateSimplified", |s| {
        let token = read_token(s)?;
        let value = unwrap_secret(s, stack_val(s, 2))?;
        let Val::Bool(simplified) = value else {
            return Err(rilua::runtime_error("isSimplified must be a boolean"));
        };
        let mut sim = borrow_state_mut(s)?;
        if simplified {
            sim.nameplate_configuration.simplified.insert(token);
        } else {
            sim.nameplate_configuration.simplified.remove(&token);
        }
        Ok(0)
    })
}

#[cfg(feature = "retail-12-0-0")]
fn read_token(state: &LuaState) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(value, Val::Str(_)) {
        return Err(rilua::runtime_error("unitToken must be a string"));
    }
    crate::lua_api::methods::val_to_string(state, value)
        .ok_or_else(|| rilua::runtime_error("unitToken must be UTF-8"))
}

#[cfg(feature = "retail-12-0-0")]
fn set_size(state: &mut LuaState) -> LuaResult<u32> {
    let mut size = [0.0; 2];
    for (offset, dimension) in size.iter_mut().enumerate() {
        let Val::Num(value) = stack_val(state, offset as i32 + 1) else {
            return Err(rilua::runtime_error("nameplate dimensions must be numbers"));
        };
        if !value.is_finite() || value < 0.0 {
            return Err(rilua::runtime_error(
                "nameplate dimensions must be finite and nonnegative",
            ));
        }
        *dimension = value;
    }
    borrow_state_mut(state)?.nameplate_configuration.size = size;
    Ok(0)
}

/// Official NamePlateType keys index independent configuration tuples.
#[derive(Debug, Default)]
pub(crate) struct NamePlateHitTestInsets {
    // Inferred empty default, not a native-observed initial configuration.
    by_type: [[f64; 4]; 2],
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_NamePlateManager")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNamePlateHitTestInsets",
        get_name_plate_hit_test_insets,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetNamePlateHitTestInsets",
        set_name_plate_hit_test_insets,
    )
}

fn parse_name_plate_type(value: Val) -> LuaResult<usize> {
    match value {
        Val::Num(0.0) => Ok(0), // Enum.NamePlateType.Friendly
        Val::Num(1.0) => Ok(1), // Enum.NamePlateType.Enemy
        _ => Err(rilua::runtime_error(
            "NamePlateType must be Friendly (0) or Enemy (1)",
        )),
    }
}

fn get_name_plate_hit_test_insets(state: &mut LuaState) -> LuaResult<u32> {
    // VM authentication implements AllowedWhenUntainted without clearing taint.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let kind = parse_name_plate_type(value)?;
    let insets = borrow_state(state)?.nameplate_hit_test_insets.by_type[kind];
    for value in insets {
        state.push(Val::Num(value));
    }
    Ok(4)
}

fn require_setter_access(state: &LuaState) -> LuaResult<()> {
    for index in 1..=5 {
        if is_secret_value(state, stack_val(state, index)) {
            return Err(rilua::runtime_error(
                "SetNamePlateHitTestInsets does not allow secret arguments",
            ));
        }
    }
    // Inferred ordinary HasRestrictions policy, not native-verified access rules.
    if !rilua::api::state_is_secure(state) {
        return Err(rilua::runtime_error(
            "SetNamePlateHitTestInsets requires an untainted caller",
        ));
    }
    Ok(())
}

fn read_inset(state: &LuaState, index: i32) -> LuaResult<f64> {
    let Val::Num(value) = stack_val(state, index) else {
        return Err(rilua::runtime_error(format!(
            "SetNamePlateHitTestInsets argument {index} must be a finite number",
        )));
    };
    if !value.is_finite() {
        return Err(rilua::runtime_error(format!(
            "SetNamePlateHitTestInsets argument {index} must be a finite number",
        )));
    }
    Ok(value)
}

fn set_name_plate_hit_test_insets(state: &mut LuaState) -> LuaResult<u32> {
    require_setter_access(state)?;
    let kind = parse_name_plate_type(stack_val(state, 1))?;
    let insets = [
        read_inset(state, 2)?,
        read_inset(state, 3)?,
        read_inset(state, 4)?,
        read_inset(state, 5)?,
    ];
    borrow_state_mut(state)?.nameplate_hit_test_insets.by_type[kind] = insets;
    Ok(0)
}
