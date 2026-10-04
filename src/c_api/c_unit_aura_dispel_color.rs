//! Bounded Retail 12.0.5 dispel-color queries over existing typed auras.

use super::c_curve_util::{evaluate_curve_value, is_curve_object};
use crate::lua_api::globals::auras::find_aura_by_instance_id;
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret, wrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const API_NAME: &str = "C_UnitAuras.GetAuraDispelTypeColor";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAuraDispelTypeColor",
        get_aura_dispel_type_color,
    )
}

fn get_aura_dispel_type_color(state: &mut LuaState) -> LuaResult<u32> {
    super::unit_aura_access::require_unit_aura_access(state, API_NAME)?;
    let curve_input = stack_val(state, 3);
    let secret_curve = is_secret_value(state, curve_input);
    // Authenticate every documented argument before ANY type validation or lookup.
    // The actual VM guard preserves original wrappers and caller taint.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let instance_id = unwrap_secret(state, stack_val(state, 2))?;
    let curve = unwrap_secret(state, curve_input)?;

    let saved_top = state.top;
    // Rust Val locals are not GC roots. Retain authenticated payloads through
    // registry/key allocations and curve calls; original arguments stay rooted.
    state.push(unit);
    state.push(instance_id);
    state.push(curve);
    let result = evaluate_dispel_color(state, unit, instance_id, curve).and_then(|color| {
        // Retain the evaluated ColorMixin table while allocating its wrapper.
        state.push(color);
        if secret_curve {
            // INFERRED: generic curve-wrapper secrecy propagates to this result.
            let wrapped = wrap_secret(state, color)?;
            state.push(wrapped);
            Ok(wrapped)
        } else {
            Ok(color)
        }
    });
    // Restore temporary roots on both success and failure. No GC safe point
    // occurs between restoring the stack and pushing the successful result.
    state.top = saved_top;
    state.push(result?);
    Ok(1)
}

fn evaluate_dispel_color(
    state: &mut LuaState,
    unit: Val,
    instance_id: Val,
    curve: Val,
) -> LuaResult<Val> {
    // INFERRED: actual UTF-8 STRING, without defaults or coercion.
    let unit = val_to_string(state, unit)
        .ok_or_else(|| runtime_error(format!("{API_NAME}: argument 1 must be a UTF-8 string")))?;
    let instance_id = validate_instance_id(instance_id)?;
    if !is_curve_object(state, curve, "LuaColorCurveObject") {
        return Err(runtime_error(format!(
            "{API_NAME}: argument 3 must be a LuaColorCurveObject"
        )));
    }
    // Preserve blocked-inclusive lookup, stores and provider selection.
    // INFERRED: missing instances error; native aura access/output restrictions
    // and secrecy propagated from secret curve POINTS remain unmodeled.
    let aura = find_aura_by_instance_id(state, &unit, instance_id)
        .ok_or_else(|| runtime_error(format!("{API_NAME}: aura instance not found")))?;
    let dispel_id = map_dispel_id(aura.dispel_type.as_deref())?;
    evaluate_curve_value(state, curve, dispel_id)
}

fn validate_instance_id(value: Val) -> LuaResult<i32> {
    // INFERRED: finite integral signed-i32 representation, no positive-only cap.
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= i32::MIN as f64
                && number <= i32::MAX as f64 =>
        {
            Ok(number as i32)
        }
        _ => Err(runtime_error(format!(
            "{API_NAME}: argument 2 must be a finite integral i32 number"
        ))),
    }
}

fn map_dispel_id(dispel_type: Option<&str>) -> LuaResult<f64> {
    // INFERRED mapping and explicit unknown-string errors, not native evidence.
    match dispel_type {
        None => Ok(0.0),
        Some("Magic") => Ok(1.0),
        Some("Curse") => Ok(2.0),
        Some("Disease") => Ok(3.0),
        Some("Poison") => Ok(4.0),
        Some("Enrage") => Ok(9.0),
        Some(unknown) => Err(runtime_error(format!(
            "{API_NAME}: unknown stored dispel type {unknown:?}"
        ))),
    }
}
