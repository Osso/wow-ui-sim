//! INFERRED (R88fd (native evidence pending)): host-supplied kiosk session inputs.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct KioskSession {
    pub enabled: bool,
    pub competitive: bool,
    pub login_info: Option<KioskLoginInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KioskLoginInfo {
    pub account_name: String,
    pub password: String,
    pub realm_address: String,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "Kiosk")?;
    table_set_rust_fn_static(state, namespace, "IsEnabled", is_enabled)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsCompetitiveModeEnabled",
        is_competitive_mode_enabled,
    )?;
    table_set_rust_fn_static(state, namespace, "GetKioskLoginInfo", get_kiosk_login_info)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetKioskSession", set_kiosk_session)
}

fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(borrow_state(state)?.kiosk_session.enabled));
    Ok(1)
}

fn is_competitive_mode_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(borrow_state(state)?.kiosk_session.competitive));
    Ok(1)
}

fn get_kiosk_login_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = borrow_state(state)?.kiosk_session.login_info.clone();
    let Some(info) = info else {
        return Ok(0);
    };
    for value in [&info.account_name, &info.password, &info.realm_address] {
        state.push(Val::Str(state.gc.intern_string(value.as_bytes())));
    }
    Ok(3)
}

fn set_kiosk_session(state: &mut LuaState) -> LuaResult<u32> {
    let session = read_kiosk_session(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.kiosk_session = session;
    Ok(0)
}

fn read_kiosk_session(state: &mut LuaState, value: Val) -> LuaResult<KioskSession> {
    let Val::Table(table) = value else {
        return Err(runtime_error(
            "A_Admin.SetKioskSession: expected a session table",
        ));
    };
    Ok(KioskSession {
        enabled: read_boolean(state, table, "enabled")?,
        competitive: read_boolean(state, table, "competitive")?,
        login_info: read_login_info(state, table)?,
    })
}

fn read_boolean(state: &LuaState, table: GcRef<Table>, key: &str) -> LuaResult<bool> {
    match table_get(state, Val::Table(table), key) {
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "A_Admin.SetKioskSession: {key} must be a boolean"
        ))),
    }
}

fn read_login_info(state: &LuaState, table: GcRef<Table>) -> LuaResult<Option<KioskLoginInfo>> {
    let values = ["accountName", "password", "realmAddress"]
        .map(|key| read_optional_string(state, table, key));
    match values {
        [
            Ok(Some(account_name)),
            Ok(Some(password)),
            Ok(Some(realm_address)),
        ] => Ok(Some(KioskLoginInfo {
            account_name,
            password,
            realm_address,
        })),
        [Ok(None), Ok(None), Ok(None)] => Ok(None),
        _ => Err(runtime_error(
            "A_Admin.SetKioskSession: login fields must be supplied together",
        )),
    }
}

fn read_optional_string(
    state: &LuaState,
    table: GcRef<Table>,
    key: &str,
) -> LuaResult<Option<String>> {
    let value = table_get(state, Val::Table(table), key);
    match value {
        Val::Nil => Ok(None),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|value| std::str::from_utf8(value.data()).ok())
            .map(str::to_owned)
            .map(Some)
            .ok_or_else(|| runtime_error(format!("A_Admin.SetKioskSession: invalid {key}"))),
        _ => Err(runtime_error(format!(
            "A_Admin.SetKioskSession: {key} must be a string or nil"
        ))),
    }
}
