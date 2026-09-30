//! Modeled unit criteria input for `C_ScenarioInfo`.

#[cfg(feature = "retail-12-0-5")]
use {
    super::ensure_namespace,
    crate::lua_api::methods::{borrow_state, create_string},
    crate::lua_bridge::{stack_val, table_set_rust_fn_static},
    rilua::table_security::{unwrap_secret, wrap_secret},
    rilua::vm::state::LuaState,
    rilua::{LuaResult, Val},
};

/// Supplied M+ Enemy Forces credit; percent and display text are not derived.
#[derive(Debug, Clone)]
pub struct UnitCriteriaProgress {
    pub actual_value: i32,
    pub percent_value: f64,
    pub percent_value_string: String,
    /// Inferred input-boundary classification, not a token-name heuristic.
    /// Replace with modeled identity restriction once native evidence exists.
    pub identity_restricted: bool,
}

#[cfg(feature = "retail-12-0-5")]
pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ScenarioInfo")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetUnitCriteriaProgressValues",
        get_unit_criteria_progress_values,
    )
}

#[cfg(feature = "retail-12-0-5")]
fn get_unit_criteria_progress_values(state: &mut LuaState) -> LuaResult<u32> {
    // The VM enforces AllowedWhenUntainted; do not declassify secret input.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(unit) = unit else {
        return Err(rilua::runtime_error(
            "GetUnitCriteriaProgressValues requires a UnitToken string at argument 1",
        ));
    };
    let unit = state
        .gc
        .string_arena
        .get(unit)
        .ok_or_else(|| rilua::runtime_error("UnitToken string has been collected"))?;
    let unit = std::str::from_utf8(unit.data())
        .map_err(|_| rilua::runtime_error("UnitToken must be a UTF-8 string"))?
        .to_owned();
    let progress = {
        let sim = borrow_state(state)?;
        if !sim.scenario.in_scenario {
            return Ok(0);
        }
        sim.scenario.unit_criteria.get(&unit).cloned()
    };
    // Inferred no-data policy: preserve absence, not fabricated zero progress.
    let Some(progress) = progress else {
        return Ok(0);
    };
    for value in [
        Val::Num(progress.actual_value as f64),
        Val::Num(progress.percent_value),
    ] {
        push_progress_value(state, value, progress.identity_restricted)?;
    }
    let display = create_string(state, &progress.percent_value_string);
    push_progress_value(state, display, progress.identity_restricted)?;
    Ok(3)
}

#[cfg(feature = "retail-12-0-5")]
fn push_progress_value(state: &mut LuaState, value: Val, restricted: bool) -> LuaResult<()> {
    // Current VM limitation: restricted output rejects tainted callers at this
    // guard. No bypass; this is not native-verified caller behavior.
    let value = if restricted {
        wrap_secret(state, value)?
    } else {
        value
    };
    state.push(value);
    Ok(())
}
