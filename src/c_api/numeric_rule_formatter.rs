//! Stateful numeric rule formatters exposed through `C_StringUtil`.

mod config;
mod model;

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{
    call_function_state, create_string, create_table, registry_get, registry_set, table_get,
    table_set_static,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use model::Breakpoint;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table, value::Userdata};
use rilua::{LuaResult, Val, runtime_error};

const METATABLE: &str = "NumericRuleFormatter";
const PRINTF: &str = "__wow_numeric_rule_printf";

#[derive(Clone, Debug, Default)]
struct NumericRuleFormatter {
    rules: Vec<Breakpoint>,
}

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    if registry_get(state, PRINTF) == Val::Nil {
        let strings = table_get(state, Val::Table(state.global), "string");
        let format = table_get(state, strings, "format");
        if !matches!(format, Val::Function(_)) {
            return Err(runtime_error(
                "numeric rule formatter requires string.format",
            ));
        }
        registry_set(state, PRINTF, format);
    }
    table_set_rust_fn_static(state, namespace, "CreateNumericRuleFormatter", create)?;
    register_rounding_enum(state)
}

fn register_rounding_enum(state: &mut LuaState) -> LuaResult<()> {
    let enums = ensure_namespace(state, "Enum")?;
    let values = create_table(state);
    for (name, value) in [("Nearest", 0.0), ("Up", 1.0), ("Down", 2.0)] {
        table_set_static(state, values, name, Val::Num(value));
    }
    table_set_static(
        state,
        Val::Table(enums),
        "NumericRuleFormatRounding",
        values,
    );
    let metadata = create_table(state);
    for (name, value) in [("MinValue", 0.0), ("MaxValue", 2.0), ("NumValues", 3.0)] {
        table_set_static(state, metadata, name, Val::Num(value));
    }
    let enum_meta = ensure_namespace(state, "EnumMeta")?;
    table_set_static(
        state,
        Val::Table(enum_meta),
        "NumericRuleFormatRounding",
        metadata,
    );
    Ok(())
}

fn metatable(state: &mut LuaState) -> LuaResult<GcRef<Table>> {
    let table = rilua::stdlib::new_metatable(state, METATABLE)?;
    if !matches!(
        table_get(state, Val::Table(table), "FormatNumber"),
        Val::Function(_)
    ) {
        table_set_static(state, Val::Table(table), "__index", Val::Table(table));
        for (name, method) in [
            ("FormatNumber", format_number as rilua::RustFn),
            ("AddBreakpoint", add_breakpoint),
            ("ClearBreakpoints", clear_breakpoints),
            ("Copy", copy),
            ("GetBreakpoints", get_breakpoints),
            ("SetBreakpoints", set_breakpoints),
        ] {
            table_set_rust_fn_static(state, table, name, method)?;
        }
    }
    Ok(table)
}

fn push_formatter(state: &mut LuaState, formatter: NumericRuleFormatter) -> LuaResult<u32> {
    let table = metatable(state)?;
    let object = state
        .gc
        .alloc_userdata(Userdata::with_metatable(Box::new(formatter), table));
    state.push(Val::Userdata(object));
    Ok(1)
}

fn create(state: &mut LuaState) -> LuaResult<u32> {
    push_formatter(state, NumericRuleFormatter::default())
}

fn formatter(state: &LuaState) -> LuaResult<&NumericRuleFormatter> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error(
            "NumericRuleFormatter method requires formatter self",
        ));
    };
    state
        .gc
        .userdata
        .get(reference)
        .and_then(|value| value.downcast_ref())
        .ok_or_else(|| runtime_error("incompatible NumericRuleFormatter receiver"))
}

fn formatter_mut(state: &mut LuaState) -> LuaResult<&mut NumericRuleFormatter> {
    let Val::Userdata(reference) = stack_val(state, 1) else {
        return Err(runtime_error(
            "NumericRuleFormatter method requires formatter self",
        ));
    };
    state
        .gc
        .userdata
        .get_mut(reference)
        .and_then(|value| value.downcast_mut())
        .ok_or_else(|| runtime_error("incompatible NumericRuleFormatter receiver"))
}

fn format_number(state: &mut LuaState) -> LuaResult<u32> {
    let input = model::finite(read_format_input(state)?)?;
    let rules = &formatter(state)?.rules;
    let end = rules.partition_point(|rule| rule.threshold <= input);
    let rule = end
        .checked_sub(1)
        .and_then(|index| rules.get(index))
        .cloned()
        .ok_or_else(|| runtime_error("NumericRuleFormatter has no matching breakpoint"))?;
    let numbers = rule.arguments(input)?;
    let result = apply_format(state, &rule.format, &numbers)?;
    state.push(result);
    Ok(1)
}

fn read_format_input(state: &LuaState) -> LuaResult<f64> {
    let value = stack_val(state, 2);
    if !rilua::table_security::is_secret_value(state, value) {
        return f64::from_stack(state, 2);
    }
    match rilua::table_security::unwrap_secret(state, value)? {
        Val::Num(number) => Ok(number),
        _ => Err(runtime_error(
            "NumericRuleFormatter secret input must contain a number",
        )),
    }
}

fn apply_format(state: &mut LuaState, format: &str, numbers: &[f64]) -> LuaResult<Val> {
    let mut args = Vec::with_capacity(numbers.len() + 1);
    args.push(create_string(state, format));
    args.extend(numbers.iter().copied().map(Val::Num));
    let printf = registry_get(state, PRINTF);
    call_function_state(state, printf, &args)
}

fn validate_printf(state: &mut LuaState, rule: &Breakpoint) -> LuaResult<()> {
    let count = rule.components.as_ref().map_or(1, Vec::len);
    apply_format(state, &rule.format, &vec![0.0; count])?;
    Ok(())
}

fn set_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    formatter(state)?;
    let rules = config::read_breakpoints(state, stack_val(state, 2))?;
    for rule in &rules {
        validate_printf(state, rule)?;
    }
    formatter_mut(state)?.rules = rules;
    Ok(0)
}

fn add_breakpoint(state: &mut LuaState) -> LuaResult<u32> {
    formatter(state)?;
    let rule = config::read_breakpoint(state, stack_val(state, 2))?;
    validate_printf(state, &rule)?;
    let rules = &mut formatter_mut(state)?.rules;
    let position = rules.binary_search_by(|existing| existing.threshold.total_cmp(&rule.threshold));
    match position {
        Ok(_) => Err(runtime_error(
            "NumericRuleFormatter duplicate thresholds are unsupported",
        )),
        Err(index) => {
            rules.insert(index, rule);
            Ok(0)
        }
    }
}

fn clear_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    formatter_mut(state)?.rules.clear();
    Ok(0)
}

fn get_breakpoints(state: &mut LuaState) -> LuaResult<u32> {
    let rules = formatter(state)?.rules.clone();
    let result = config::write_breakpoints(state, &rules);
    state.push(result);
    Ok(1)
}

fn copy(state: &mut LuaState) -> LuaResult<u32> {
    let copied = formatter(state)?.clone();
    push_formatter(state, copied)
}
