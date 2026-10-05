//! Modeled unit criteria input for `C_ScenarioInfo`.

#[cfg(feature = "retail-12-0-5")]
use {
    super::ensure_namespace,
    crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static},
    crate::lua_bridge::{stack_val, table_set_rust_fn_static},
    rilua::table_security::{unwrap_secret, wrap_host_secret_number, wrap_host_secret_string},
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
    )?;
    table_set_rust_fn_static(state, namespace, "GetDisplayInfo", get_display_info)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetTieredEntranceActiveSpells",
        get_tiered_entrance_active_spells,
    )
}

/// Returns nothing outside a scenario or when it has no theme color.
#[cfg(feature = "retail-12-0-5")]
fn get_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let theme_color = {
        let sim = borrow_state(state)?;
        sim.scenario
            .in_scenario
            .then_some(sim.scenario.display_theme_color)
            .flatten()
    };
    let Some((r, g, b)) = theme_color else {
        return Ok(0);
    };
    let theme_color = super::c_allied_races::create_color_mixin(state, r, g, b);
    state.push(theme_color);
    let info = create_table(state);
    table_set_static(state, info, "themeColor", theme_color);
    state.pop();
    state.push(info);
    Ok(1)
}

/// Nil unless inside a tiered entrance scenario.
#[cfg(feature = "retail-12-0-5")]
fn get_tiered_entrance_active_spells(state: &mut LuaState) -> LuaResult<u32> {
    let spells = {
        let sim = borrow_state(state)?;
        let scenario = &sim.scenario;
        (scenario.in_scenario && scenario.is_tiered_entrance)
            .then(|| scenario.tiered_entrance_active_spells.clone())
    };
    let Some(spells) = spells else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    for (index, spell_id) in spells.iter().enumerate() {
        super::helpers::set_table_array(
            state,
            table,
            index as i64 + 1,
            Val::Num(f64::from(*spell_id)),
        );
    }
    state.push(table);
    Ok(1)
}

#[cfg(feature = "retail-12-0-5")]
fn get_unit_criteria_progress_values(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_unit_token(state)?;
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
    push_progress_results(state, &progress);
    Ok(3)
}

#[cfg(feature = "retail-12-0-5")]
fn read_unit_token(state: &LuaState) -> LuaResult<String> {
    // Secret Lua input retains the VM's untainted-caller guard.
    let unit = unwrap_secret(state, stack_val(state, 1))?;
    let Val::Str(unit) = unit else {
        return Err(rilua::runtime_error(
            "GetUnitCriteriaProgressValues requires a UnitToken string at argument 1",
        ));
    };
    let string = state
        .gc
        .string_arena
        .get(unit)
        .ok_or_else(|| rilua::runtime_error("UnitToken string has been collected"))?;
    std::str::from_utf8(string.data())
        .map(str::to_owned)
        .map_err(|_| rilua::runtime_error("UnitToken must be a UTF-8 string"))
}

#[cfg(feature = "retail-12-0-5")]
fn push_progress_results(state: &mut LuaState, progress: &UnitCriteriaProgress) {
    for number in [progress.actual_value as f64, progress.percent_value] {
        let value = if progress.identity_restricted {
            wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        state.push(value);
    }
    let display = if progress.identity_restricted {
        wrap_host_secret_string(state, &progress.percent_value_string)
    } else {
        create_string(state, &progress.percent_value_string)
    };
    state.push(display);
}
