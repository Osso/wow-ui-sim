//! VERIFIED returning paths from R88q; unknown cleanup/wrapper exits are not modeled.
use super::helpers::{bad_argument, set_table_array};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, registry_get, table_get_static,
    table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

type Member = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);
const GLUE: &[Member] = &[
    ("GetVASPurchaseStateInfo", get_purchase),
    ("GetDisconnectOnLogout", get_disconnect),
    ("GetVASProductReady", get_ready),
    ("ClearVASProductReady", clear_ready),
    ("RequestCharacterQueueTime", request_queue),
    ("UpdateVASPurchaseStates", request_refresh),
];
const SECURE: &[Member] = &[
    ("GetVASErrors", get_vas_errors),
    ("SetDisconnectOnLogout", set_disconnect),
    ("SetVASProductReady", set_ready),
];
const PURCHASE_USAGE: &str =
    "Usage: local productID, state, errors = C_StoreGlue.GetVASPurchaseStateInfo(guid)";
const QUEUE_USAGE: &str = "Usage: C_StoreGlue.RequestCharacterQueueTime(guid)";

pub(crate) fn enabled() -> bool {
    // INFERRED PTR/Forever: R88q Retail facts + selected unconditional Glue consumers.
    // Their pinned generated Store declarations are absent (lt-941b #1283).
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
}
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !enabled() {
        return Ok(());
    }
    publish_namespace(state, "C_StoreGlue", GLUE)?;
    publish_namespace(state, "C_StoreSecure", SECURE)
}
fn publish_namespace(
    state: &mut LuaState,
    name: &'static str,
    members: &[Member],
) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, name)?;
    publish_members(state, namespace, members)?;
    if let Val::Table(secure) = registry_get(state, "__secureenv") {
        let value = table_get_static(state, Val::Table(secure), name);
        let namespace = match value {
            Val::Table(table) => table,
            _ => {
                let Val::Table(table) = create_table(state) else {
                    unreachable!()
                };
                table_set_static(state, Val::Table(secure), name, Val::Table(table));
                table
            }
        };
        publish_members(state, namespace, members)?;
    }
    Ok(())
}
fn publish_members(state: &mut LuaState, table: GcRef<Table>, members: &[Member]) -> LuaResult<()> {
    for &(name, callback) in members {
        table_set_rust_fn_static(state, table, name, callback)?;
    }
    Ok(())
}
fn pending_error(state: &LuaState, name: &str) -> LuaResult<()> {
    if let Some(message) = &borrow_state(state)?.store_glue.pending_error {
        return Err(runtime_error(format!("{name}(): {message}")));
    }
    Ok(())
}
fn read_argument(state: &LuaState, name: &str, usage: &str) -> LuaResult<Val> {
    let value = stack_val(state, 1);
    if value == Val::Nil {
        return Err(bad_argument(1, name, usage));
    }
    if !rilua::table_security::is_secret_value(state, value) {
        return Ok(value);
    }
    let mode = borrow_state(state)?.store_glue.secret_access_mode;
    let reason = match mode {
        Some(0) => "Secret values are not allowed for this argument.",
        Some(1) => return rilua::table_security::unwrap_secret(state, value).map_err(|_| {
            bad_argument(1, name, &format!("{usage}. Secret values are only allowed during untainted execution for this argument."))
        }),
        Some(_) => "Unknown secret access mode.",
        None => return Err(runtime_error("C_StoreGlue secret-access metadata is not modeled")),
    };
    Err(bad_argument(1, name, &format!("{usage}. {reason}")))
}
fn read_guid(state: &LuaState, name: &str, usage: &str) -> LuaResult<super::store_guid::Outcome> {
    let value = read_argument(state, name, usage)?;
    let bytes = match value {
        Val::Num(number) => Val::Num(number).to_string().into_bytes(),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .map(|value| value.data().to_vec())
            .unwrap_or_default(),
        _ => return Err(bad_argument(1, name, usage)),
    };
    Ok(borrow_state(state)?.store_glue.resolve_guid(&bytes))
}
fn get_purchase(state: &mut LuaState) -> LuaResult<u32> {
    let guid = read_guid(state, "GetVASPurchaseStateInfo", PURCHASE_USAGE)?;
    let record = borrow_state(state)?.store_glue.purchase_record(&guid);
    pending_error(state, "GetVASPurchaseStateInfo")?;
    state.push(Val::Num(f64::from(record.product_id)));
    state.push(Val::Num(f64::from(record.state)));
    let errors = create_table(state);
    state.push(errors);
    if record.errors.first().is_some_and(|value| *value != 0) {
        for (index, value) in record.errors.iter().enumerate() {
            set_table_array(state, errors, index as i64 + 1, Val::Num(f64::from(*value)));
        }
    }
    Ok(3)
}
// VERIFIED R88w.w1 (#1308): rejection precedes source access; the cause is UNKNOWN.
fn get_vas_errors(state: &mut LuaState) -> LuaResult<u32> {
    if borrow_state(state)?.store_glue.vas_errors_entry_rejected {
        return Ok(0);
    }
    let snapshot = borrow_state_mut(state)?.store_glue.vas_errors_snapshot();
    pending_error(state, "GetVASErrors")?;
    let errors = create_table(state);
    state.push(errors);
    for (index, value) in snapshot.into_iter().enumerate() {
        set_table_array(state, errors, index as i64 + 1, Val::Num(f64::from(value)));
    }
    Ok(1)
}

fn get_disconnect(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.store_glue.disconnect_on_logout;
    pending_error(state, "GetDisconnectOnLogout")?;
    state.push(Val::Bool(value));
    Ok(1)
}
fn get_ready(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.store_glue.vas_product_ready;
    pending_error(state, "GetVASProductReady")?;
    state.push(Val::Bool(value));
    Ok(1)
}
fn clear_ready(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.store_glue.vas_product_ready = false;
    pending_error(state, "ClearVASProductReady")?;
    Ok(0)
}
fn read_truthy_argument(state: &LuaState, name: &str, usage: &str) -> LuaResult<bool> {
    let value = read_argument(state, name, usage)?;
    if value == Val::Nil {
        return Err(bad_argument(1, name, usage));
    }
    Ok(value != Val::Bool(false))
}
fn set_disconnect(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let usage = "Usage: C_StoreSecure.SetDisconnectOnLogout(disconnectOnLogout)";
    let value = read_truthy_argument(state, "SetDisconnectOnLogout", usage)?;
    borrow_state_mut(state)?.store_glue.disconnect_on_logout = value;
    pending_error(state, "SetDisconnectOnLogout")?;
    Ok(0)
}
fn set_ready(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let usage = "Usage: C_StoreSecure.SetVASProductReady(isReady)";
    let value = read_truthy_argument(state, "SetVASProductReady", usage)?;
    borrow_state_mut(state)?.store_glue.vas_product_ready = value;
    pending_error(state, "SetVASProductReady")?;
    Ok(0)
}
fn restricted_call(state: &mut LuaState) -> LuaResult<bool> {
    use super::store_glue_model::RestrictedExecution;
    let metadata = borrow_state(state)?.store_glue.restricted_execution.clone();
    let Some(metadata) = metadata else {
        return Ok(false);
    };
    match metadata {
        RestrictedExecution::Addon {
            source,
            function,
            callback_present,
        } if callback_present => {
            dispatch_forbidden(state, "ADDON_ACTION_FORBIDDEN", &[source, function])?;
        }
        RestrictedExecution::Macro {
            function,
            callback_present,
        } if callback_present => {
            dispatch_forbidden(state, "MACRO_ACTION_FORBIDDEN", &[function])?;
        }
        _ => {}
    }
    Ok(true)
}
fn dispatch_forbidden(state: &mut LuaState, event: &str, values: &[String]) -> LuaResult<()> {
    let saved_top = state.top;
    let mut args = Vec::new();
    for value in values {
        let value = create_string(state, value);
        state.push(value);
        args.push(value);
    }
    let result = dispatch_event_now(state, event, &args);
    state.top = saved_top;
    result
}
fn request_queue(state: &mut LuaState) -> LuaResult<u32> {
    let guid = read_guid(state, "RequestCharacterQueueTime", QUEUE_USAGE)?;
    borrow_state_mut(state)?
        .store_glue
        .prepare_queue_request(&guid);
    pending_error(state, "RequestCharacterQueueTime")?;
    Ok(0)
}
fn request_refresh(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.store_glue.request_refresh();
    pending_error(state, "UpdateVASPurchaseStates")?;
    Ok(0)
}

pub use super::store_glue_model::{
    CompletionInput, RestrictedExecution, StoreGlueState, StoreGuid, StoreRequest,
    VasPurchaseRecord, VasQueueCharacter, receive_bulk, receive_completion, receive_queue_response,
    receive_removal, receive_service_update,
};
