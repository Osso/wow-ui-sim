//! Bounded Retail 12.0.5 application display counts over existing typed auras.

use crate::lua_api::globals::auras::find_aura_by_instance_id;
use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::{is_secret_value, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const DEFAULT_MIN_DISPLAY_COUNT: f64 = 2.0;
const API_NAME: &str = "C_UnitAuras.GetAuraApplicationDisplayCount";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAuraApplicationDisplayCount",
        get_aura_application_display_count,
    )
}

fn get_aura_application_display_count(state: &mut LuaState) -> LuaResult<u32> {
    super::unit_aura_access::require_unit_aura_access(state, API_NAME)?;
    let unit = read_authenticated_unit(state)?;
    let instance_id = read_authenticated_instance_id(state)?;
    let minimum = if state.top.saturating_sub(state.base) < 3 {
        DEFAULT_MIN_DISPLAY_COUNT
    } else {
        read_public_threshold(state, 3)?
    };
    let maximum = match stack_val(state, 4) {
        Val::Nil => None,
        _ => Some(read_public_threshold(state, 4)?),
    };
    // Validate every documented position before the blocked-inclusive lookup.
    // Native unit access, valid-instance enforcement and output secrecy are unmodeled.
    let count = find_aura_by_instance_id(state, &unit, instance_id).map(|aura| aura.applications);
    let display = format_display_count(count, minimum, maximum);
    let result = create_string(state, &display);
    state.push(result);
    Ok(1)
}

fn read_authenticated_unit(state: &LuaState) -> LuaResult<String> {
    // VM authentication preserves caller taint and original secret identity.
    let value = unwrap_secret(state, stack_val(state, 1))?;
    // INFERRED: actual UTF-8 STRING only; no nil default or numeric coercion.
    val_to_string(state, value)
        .ok_or_else(|| runtime_error(format!("{API_NAME}: argument 1 must be a UTF-8 string")))
}

fn read_authenticated_instance_id(state: &LuaState) -> LuaResult<i32> {
    let value = unwrap_secret(state, stack_val(state, 2))?;
    // INFERRED: finite integral signed-i32 representation, without lossy casts.
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

fn read_public_threshold(state: &LuaState, position: i32) -> LuaResult<f64> {
    let value = stack_val(state, position);
    // NeverSecret is independent of caller security. Inspect actual VM wrappers;
    // do not unwrap thresholds or classify all host userdata as secret.
    if is_secret_value(state, value) {
        return Err(runtime_error(format!(
            "{API_NAME}: argument {position} must not be secret"
        )));
    }
    // INFERRED: finite actual numbers, including fractional/negative/large values.
    match value {
        Val::Num(number) if number.is_finite() => Ok(number),
        _ => Err(runtime_error(format!(
            "{API_NAME}: argument {position} must be a finite number"
        ))),
    }
}

fn format_display_count(count: Option<i32>, minimum: f64, maximum: Option<f64>) -> String {
    // INFERRED: missing instances yield empty public strings; minimum wins first.
    let Some(count) = count else {
        return String::new();
    };
    let applications = f64::from(count);
    if applications < minimum {
        return String::new();
    }
    if maximum.is_some_and(|maximum| applications > maximum) {
        return "*".to_owned();
    }
    count.to_string()
}
