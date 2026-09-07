use super::model::{Breakpoint, Component, Rounding, finite, validate_format};
use crate::lua_api::methods::{
    create_string, create_table, table_get, table_set_num, table_set_static,
};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn read_breakpoints(state: &mut LuaState, value: Val) -> LuaResult<Vec<Breakpoint>> {
    let mut rules: Vec<_> = array_values(state, value)?
        .into_iter()
        .map(|entry| read_breakpoint(state, entry))
        .collect::<LuaResult<_>>()?;
    rules.sort_by(|left, right| left.threshold.total_cmp(&right.threshold));
    if rules
        .windows(2)
        .any(|pair| pair[0].threshold == pair[1].threshold)
    {
        return Err(runtime_error(
            "NumericRuleFormatter duplicate thresholds are unsupported",
        ));
    }
    Ok(rules)
}

pub(super) fn read_breakpoint(state: &mut LuaState, value: Val) -> LuaResult<Breakpoint> {
    require_table(value)?;
    let threshold = required_number(state, value, "threshold")?;
    let threshold = if threshold == 0.0 { 0.0 } else { threshold };
    let step = read_step(state, value)?;
    let rounding = read_rounding(state, value)?;
    let min = optional_number(state, value, "min")?;
    let max = optional_number(state, value, "max")?;
    if min.zip(max).is_some_and(|(min, max)| min > max) {
        return Err(runtime_error(
            "NumericRuleFormatter minimum exceeds maximum",
        ));
    }
    let format = read_format(state, value)?;
    let components = read_components(state, value)?;
    validate_format(&format, components.as_ref().map(Vec::len))?;
    Ok(Breakpoint {
        threshold,
        step,
        rounding,
        min,
        max,
        format,
        components,
    })
}

fn require_table(value: Val) -> LuaResult<()> {
    if matches!(value, Val::Table(_)) {
        Ok(())
    } else {
        Err(runtime_error(
            "NumericRuleFormatter configuration must be a table",
        ))
    }
}

fn array_values(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error(
            "NumericRuleFormatter configuration array expected",
        ));
    };
    let table =
        state.gc.tables.get(reference).ok_or_else(|| {
            runtime_error("NumericRuleFormatter configuration table is unavailable")
        })?;
    Ok((1..=table.len(&state.gc.string_arena))
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn optional_number(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Option<f64>> {
    match table_get(state, table, key) {
        Val::Nil => Ok(None),
        Val::Num(number) => finite(number).map(Some),
        _ => Err(runtime_error(format!(
            "NumericRuleFormatter {key} must be numeric"
        ))),
    }
}

fn required_number(state: &mut LuaState, table: Val, key: &str) -> LuaResult<f64> {
    optional_number(state, table, key)?
        .ok_or_else(|| runtime_error(format!("NumericRuleFormatter requires {key}")))
}

fn read_step(state: &mut LuaState, table: Val) -> LuaResult<Option<f64>> {
    let step = optional_number(state, table, "step")?;
    if step.is_some_and(|step| step <= 0.0) {
        return Err(runtime_error("NumericRuleFormatter step must be positive"));
    }
    Ok(step)
}

fn read_rounding(state: &mut LuaState, table: Val) -> LuaResult<Rounding> {
    match optional_number(state, table, "rounding")? {
        None | Some(0.0) => Ok(Rounding::Nearest),
        Some(1.0) => Ok(Rounding::Up),
        Some(2.0) => Ok(Rounding::Down),
        _ => Err(runtime_error(
            "NumericRuleFormatter rounding must be Nearest, Up or Down",
        )),
    }
}

fn read_format(state: &mut LuaState, table: Val) -> LuaResult<String> {
    let Val::Str(reference) = table_get(state, table, "format") else {
        return Err(runtime_error(
            "NumericRuleFormatter requires a format string",
        ));
    };
    let bytes = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| runtime_error("NumericRuleFormatter format is unavailable"))?
        .data();
    String::from_utf8(bytes.to_vec())
        .map_err(|_| runtime_error("NumericRuleFormatter format must be UTF-8"))
}

fn read_components(state: &mut LuaState, table: Val) -> LuaResult<Option<Vec<Component>>> {
    let value = table_get(state, table, "components");
    if value == Val::Nil {
        return Ok(None);
    }
    array_values(state, value)?
        .into_iter()
        .map(|entry| read_component(state, entry))
        .collect::<LuaResult<Vec<_>>>()
        .map(Some)
}

fn read_component(state: &mut LuaState, table: Val) -> LuaResult<Component> {
    require_table(table)?;
    let div = nonzero_number(state, table, "div")?;
    let modulo = nonzero_number(state, table, "mod")?;
    let step = read_step(state, table)?;
    let rounding = read_rounding(state, table)?;
    Ok(Component {
        div,
        modulo,
        step,
        rounding,
    })
}

fn nonzero_number(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Option<f64>> {
    let number = optional_number(state, table, key)?;
    if number == Some(0.0) {
        return Err(runtime_error(format!(
            "NumericRuleFormatter {key} must not be zero"
        )));
    }
    Ok(number)
}

pub(super) fn write_breakpoints(state: &mut LuaState, rules: &[Breakpoint]) -> Val {
    let result = create_table(state);
    for (index, rule) in rules.iter().enumerate() {
        let entry = write_breakpoint(state, rule);
        if let Val::Table(table) = result {
            table_set_num(state, table, (index + 1) as f64, entry);
        }
    }
    result
}

fn write_breakpoint(state: &mut LuaState, rule: &Breakpoint) -> Val {
    let result = create_table(state);
    let format = create_string(state, &rule.format);
    table_set_static(state, result, "threshold", Val::Num(rule.threshold));
    table_set_static(state, result, "format", format);
    write_rounding(state, result, rule.step, rule.rounding);
    write_optional_number(state, result, "min", rule.min);
    write_optional_number(state, result, "max", rule.max);
    if let Some(components) = &rule.components {
        let entries = write_components(state, components);
        table_set_static(state, result, "components", entries);
    }
    result
}

fn write_components(state: &mut LuaState, components: &[Component]) -> Val {
    let result = create_table(state);
    for (index, component) in components.iter().enumerate() {
        let entry = create_table(state);
        write_optional_number(state, entry, "div", component.div);
        write_optional_number(state, entry, "mod", component.modulo);
        write_rounding(state, entry, component.step, component.rounding);
        if let Val::Table(table) = result {
            table_set_num(state, table, (index + 1) as f64, entry);
        }
    }
    result
}

fn write_rounding(state: &mut LuaState, table: Val, step: Option<f64>, rounding: Rounding) {
    write_optional_number(state, table, "step", step);
    table_set_static(state, table, "rounding", Val::Num(rounding as u8 as f64));
}

fn write_optional_number(state: &mut LuaState, table: Val, key: &'static str, value: Option<f64>) {
    if let Some(value) = value {
        table_set_static(state, table, key, Val::Num(value));
    }
}
