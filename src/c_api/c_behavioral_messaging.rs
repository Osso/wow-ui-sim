//! Supplied behavioral notifications and their consumer receipts.
//!
//! The Retail and Forever BehavioralMessagingDocumentation.lua declarations
//! define the event payload and receipt arguments. Scenario behavior is
//! INFERRED (R88fm (native evidence pending)).

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug)]
pub(crate) struct BehavioralMessagingReceipt {
    pub db_id: u32,
    pub open_time_seconds: f64,
    pub read_time_seconds: f64,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_BehavioralMessaging")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SendNotificationReceipt",
        send_notification_receipt,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("QueueBehavioralNotification", queue_behavioral_notification)?
        .set_function(
            "GetBehavioralNotificationReceipts",
            get_behavioral_notification_receipts,
        )
}

fn queue_behavioral_notification(state: &mut LuaState) -> LuaResult<u32> {
    let notification_type = String::from_stack(state, 1)?;
    let db_id = u32::from_stack(state, 2)?;
    let notification_type = Val::Str(state.gc.intern_string(notification_type.as_bytes()));
    let payload = [notification_type, Val::Num(f64::from(db_id))];
    dispatch_event_now(state, "BEHAVIORAL_NOTIFICATION", &payload)?;
    Ok(0)
}

fn send_notification_receipt(state: &mut LuaState) -> LuaResult<u32> {
    let receipt = BehavioralMessagingReceipt {
        db_id: u32::from_stack(state, 1)?,
        open_time_seconds: f64::from_stack(state, 2)?,
        read_time_seconds: f64::from_stack(state, 3)?,
    };
    borrow_state_mut(state)?
        .behavioral_messaging_receipts
        .push(receipt);
    Ok(0)
}

fn get_behavioral_notification_receipts(state: &mut LuaState) -> LuaResult<u32> {
    let receipts = borrow_state(state)?.behavioral_messaging_receipts.clone();
    let list = create_table(state);
    let Val::Table(list_ref) = list else {
        return Ok(0);
    };
    for (index, receipt) in receipts.iter().enumerate() {
        let entry = build_receipt_table(state, receipt)?;
        if let Some(table) = state.gc.tables.get_mut(list_ref) {
            let _ = table.raw_set(Val::Num((index + 1) as f64), entry, &state.gc.string_arena);
        }
        state.gc.barrier_back(list_ref);
    }
    state.push(list);
    Ok(1)
}

fn build_receipt_table(
    state: &mut LuaState,
    receipt: &BehavioralMessagingReceipt,
) -> LuaResult<Val> {
    TableBuilder::new(state)
        .set("dbId", receipt.db_id)?
        .set("openTimeSeconds", receipt.open_time_seconds)?
        .set("readTimeSeconds", receipt.read_time_seconds)
        .map(TableBuilder::build)
}
