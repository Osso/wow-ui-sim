//! INFERRED supplied warning inputs for the Glue character-select consumer.
//! See R88du (native evidence pending); empty state means no host warnings.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_get_num,
    table_set_num,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Debug)]
pub(crate) struct ConfigurationWarning {
    warning: u32,
    text: String,
    seen: bool,
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    rilua::LuaApiMut::register_function(
        lua,
        "HasCheckedSystemRequirements",
        has_checked_system_requirements,
    )?;
    rilua::LuaApiMut::register_function(
        lua,
        "SetCheckedSystemRequirements",
        set_checked_system_requirements,
    )?;
    let state = lua.state_mut();
    let namespace = ensure_namespace(state, "C_ConfigurationWarnings")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetConfigurationWarningSeen",
        get_warning_seen,
    )?;
    table_set_rust_fn_static(state, namespace, "GetConfigurationWarnings", get_warnings)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetConfigurationWarningString",
        get_warning_string,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetConfigurationWarningSeen",
        set_warning_seen,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetConfigurationWarnings", set_warnings)?
        .set_function(
            "SetSystemRequirementsChecked",
            set_system_requirements_checked,
        )
}

fn has_checked_system_requirements(state: &mut LuaState) -> LuaResult<u32> {
    let checked = borrow_state(state)?.system_requirements_checked;
    state.push(Val::Bool(checked));
    Ok(1)
}

fn set_checked_system_requirements(state: &mut LuaState) -> LuaResult<u32> {
    let checked = read_checked(state, 1)?;
    borrow_state_mut(state)?.system_requirements_checked = checked;
    Ok(0)
}

fn set_system_requirements_checked(state: &mut LuaState) -> LuaResult<u32> {
    let checked = read_checked(state, 1)?;
    borrow_state_mut(state)?.system_requirements_checked = checked;
    Ok(0)
}

fn read_checked(state: &LuaState, index: i32) -> LuaResult<bool> {
    match stack_val(state, index) {
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error("checked must be a boolean")),
    }
}

fn set_warnings(state: &mut LuaState) -> LuaResult<u32> {
    let warnings = parse_warnings(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.configuration_warnings = warnings;
    Ok(0)
}

fn parse_warnings(state: &mut LuaState, input: Val) -> LuaResult<Vec<ConfigurationWarning>> {
    let Val::Table(reference) = input else {
        return Err(runtime_error("SetConfigurationWarnings requires an array"));
    };
    let count = state
        .gc
        .tables
        .get(reference)
        .expect("live warning input")
        .array_slice()
        .len();
    (1..=count)
        .map(|index| {
            let row = table_get_num(state, reference, index as f64);
            parse_warning(state, row, index)
        })
        .collect()
}

fn parse_warning(state: &mut LuaState, row: Val, index: usize) -> LuaResult<ConfigurationWarning> {
    if !matches!(row, Val::Table(_)) {
        return Err(invalid_warning(index));
    }
    let warning_value = table_get(state, row, "warning");
    let warning = read_warning_id(warning_value, index)?;
    let text_value = table_get(state, row, "text");
    let text = read_warning_text(state, text_value, index)?;
    let seen = read_warning_seen(table_get(state, row, "seen"), index)?;
    Ok(ConfigurationWarning {
        warning,
        text,
        seen,
    })
}

fn read_warning_id(value: Val, index: usize) -> LuaResult<u32> {
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && (0.0..=u32::MAX as f64).contains(&number) =>
        {
            Ok(number as u32)
        }
        _ => Err(runtime_error(format!(
            "SetConfigurationWarnings: warnings[{index}].warning must be a non-negative integer"
        ))),
    }
}

fn read_warning_text(state: &LuaState, value: Val, index: usize) -> LuaResult<String> {
    let Val::Str(reference) = value else {
        return Err(runtime_error(format!(
            "SetConfigurationWarnings: warnings[{index}].text must be a string"
        )));
    };
    let text = state
        .gc
        .string_arena
        .get(reference)
        .ok_or_else(|| invalid_warning(index))?;
    String::from_utf8(text.data().to_vec()).map_err(|_| invalid_warning(index))
}

fn read_warning_seen(value: Val, index: usize) -> LuaResult<bool> {
    match value {
        Val::Nil => Ok(false),
        Val::Bool(seen) => Ok(seen),
        _ => Err(runtime_error(format!(
            "SetConfigurationWarnings: warnings[{index}].seen must be a boolean"
        ))),
    }
}

fn invalid_warning(index: usize) -> rilua::LuaError {
    runtime_error(format!(
        "SetConfigurationWarnings: warnings[{index}] must contain a warning ID and text"
    ))
}

fn get_warning_seen(state: &mut LuaState) -> LuaResult<u32> {
    let warning = read_warning_id(stack_val(state, 1), 1)?;
    let seen = borrow_state(state)?
        .configuration_warnings
        .iter()
        .find(|entry| entry.warning == warning)
        .is_some_and(|entry| entry.seen);
    state.push(Val::Bool(seen));
    Ok(1)
}

fn get_warnings(state: &mut LuaState) -> LuaResult<u32> {
    let include_seen = matches!(stack_val(state, 1), Val::Bool(true));
    let warnings: Vec<u32> = borrow_state(state)?
        .configuration_warnings
        .iter()
        .filter(|warning| include_seen || !warning.seen)
        .map(|warning| warning.warning)
        .collect();
    let output = create_table(state);
    let Val::Table(reference) = output else {
        unreachable!("create_table returns a table")
    };
    state.push(output);
    for (index, warning) in warnings.into_iter().enumerate() {
        table_set_num(
            state,
            reference,
            (index + 1) as f64,
            Val::Num(warning as f64),
        );
    }
    Ok(1)
}

fn set_warning_seen(state: &mut LuaState) -> LuaResult<u32> {
    let warning = read_warning_id(stack_val(state, 1), 1)?;
    if let Some(entry) = borrow_state_mut(state)?
        .configuration_warnings
        .iter_mut()
        .find(|entry| entry.warning == warning)
    {
        entry.seen = true;
    }
    Ok(0)
}

fn get_warning_string(state: &mut LuaState) -> LuaResult<u32> {
    let warning = read_warning_id(stack_val(state, 1), 1)?;
    let text = borrow_state(state)?
        .configuration_warnings
        .iter()
        .find(|entry| entry.warning == warning)
        .map(|entry| entry.text.clone());
    let value = match text {
        Some(text) => create_string(state, &text),
        None => Val::Nil,
    };
    state.push(value);
    Ok(1)
}
