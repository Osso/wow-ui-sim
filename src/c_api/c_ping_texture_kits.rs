//! Exact host-supplied ping kits; no native catalog or inferred substitute.

use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table_with_capacity, table_get,
    table_set_static, val_to_string,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};
#[cfg(feature = "retail-12-0-0")]
use std::collections::HashMap;

#[cfg(feature = "retail-12-0-0")]
const INVALID_PING_TYPE: &str =
    "C_Ping.GetTextureKitForType: ping type must be a finite i32 integer";
const INVALID_PING_OPTIONS: &str =
    "A_Admin.SetDefaultPingOptions: expected an array of option tables";

/// Whole host input; empty by default, with no native mapping catalog.
#[cfg(feature = "retail-12-0-0")]
#[derive(Default)]
pub struct PingTextureKits {
    pub by_type: HashMap<i32, String>,
}

/// Host-supplied rows used to build the default ping wheel.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PingDefaultOptions {
    pub rows: Vec<PingDefaultOption>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PingDefaultOption {
    pub ping_type: i32,
    pub order_index: i32,
    pub ui_texture_kit_id: Option<String>,
}

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Ping")?;
    table_set_rust_fn_static(state, namespace, "GetTextureKitForType", query_texture_kit)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_default_options(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Ping")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetDefaultPingOptions",
        get_default_ping_options,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetDefaultPingOptions", set_default_ping_options)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn get_default_ping_options(state: &mut LuaState) -> LuaResult<u32> {
    let rows = borrow_state(state)?.ping_default_options.rows.clone();
    let options = create_table_with_capacity(state, rows.len());
    for (index, row) in rows.iter().enumerate() {
        let option = create_table_with_capacity(state, 3);
        table_set_static(state, option, "type", Val::Num(f64::from(row.ping_type)));
        table_set_static(
            state,
            option,
            "orderIndex",
            Val::Num(f64::from(row.order_index)),
        );
        if let Some(kit) = &row.ui_texture_kit_id {
            let kit = create_string(state, kit);
            table_set_static(state, option, "uiTextureKitID", kit);
        }
        set_table_array(state, options, index as i64 + 1, option);
    }
    state.push(options);
    Ok(1)
}

fn set_default_ping_options(state: &mut LuaState) -> LuaResult<u32> {
    let options = read_default_ping_options(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.ping_default_options.rows = options;
    Ok(0)
}

fn read_default_ping_options(
    state: &mut LuaState,
    value: Val,
) -> LuaResult<Vec<PingDefaultOption>> {
    let Val::Table(array) = value else {
        return Err(runtime_error(INVALID_PING_OPTIONS));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(INVALID_PING_OPTIONS))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let entry = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            read_default_ping_option(state, entry)
        })
        .collect()
}

fn read_default_ping_option(state: &mut LuaState, value: Val) -> LuaResult<PingDefaultOption> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INVALID_PING_OPTIONS));
    };
    Ok(PingDefaultOption {
        ping_type: read_option_integer(state, table, "type")?,
        order_index: read_option_integer(state, table, "orderIndex")?,
        ui_texture_kit_id: read_optional_option_string(state, table, "uiTextureKitID")?,
    })
}

fn read_option_integer(state: &mut LuaState, table: GcRef<Table>, key: &str) -> LuaResult<i32> {
    let Val::Num(value) = table_get(state, Val::Table(table), key) else {
        return Err(invalid_option_field(key));
    };
    let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value);
    if !value.is_finite() || value.fract() != 0.0 || !in_range {
        return Err(invalid_option_field(key));
    }
    Ok(value as i32)
}

fn read_optional_option_string(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &str,
) -> LuaResult<Option<String>> {
    match table_get(state, Val::Table(table), key) {
        Val::Nil => Ok(None),
        value => val_to_string(state, value)
            .map(Some)
            .ok_or_else(|| invalid_option_field(key)),
    }
}

fn invalid_option_field(key: &str) -> rilua::LuaError {
    runtime_error(format!("A_Admin.SetDefaultPingOptions: invalid {key}"))
}

#[cfg(feature = "retail-12-0-0")]
fn read_ping_type(value: Val) -> LuaResult<i32> {
    let Val::Num(number) = value else {
        return Err(runtime_error(INVALID_PING_TYPE));
    };
    let in_range = (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number);
    let represented = number.is_finite() && number.fract() == 0.0 && in_range;
    if !represented {
        return Err(runtime_error(INVALID_PING_TYPE));
    }
    Ok(number as i32)
}

#[cfg(feature = "retail-12-0-0")]
fn query_texture_kit(state: &mut LuaState) -> LuaResult<u32> {
    // AllowedWhenUntainted: authenticate before validation or any map lookup.
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let kind = read_ping_type(value)?;
    let kit = borrow_state(state)?
        .ping_texture_kits
        .by_type
        .get(&kind)
        .cloned()
        .ok_or_else(|| {
            runtime_error(format!(
                "unmapped ping type {kind}: host-supplied mapping required"
            ))
        })?;
    let result = create_string(state, &kit);
    state.push(result);
    Ok(1)
}
