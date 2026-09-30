//! Per-type configuration only; no world plate bounds or hit testing exist.

use super::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
