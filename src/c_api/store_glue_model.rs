//! R88q host-supplied VAS state. No fabricated server records or automatic responses.
use crate::lua_api::WowLuaEnv;
use rilua::Val;
use std::collections::HashMap;

/// Empty string represents the native empty GUID; nonempty strings are host identities.
pub type StoreGuid = String;
#[derive(Clone, Debug, Default)]
pub struct VasPurchaseRecord {
    pub product_id: i32,
    pub state: i32,
    pub errors: Vec<i32>,
}
#[derive(Clone, Debug)]
pub struct VasQueueCharacter {
    pub guid: StoreGuid,
    /// Opaque host correlation identity. No native byte layout/packet name claim.
    pub correlation: Vec<u8>,
}
#[derive(Clone, Debug)]
pub enum StoreRequest {
    Queue { correlation: Vec<u8>, sequence: u64 },
    Refresh,
}
#[derive(Debug, Default)]
pub struct StoreCallbacks {
    pub purchase_complete: bool,
    pub queue_status: bool,
    pub state_changed: bool,
}
#[derive(Debug, Default)]
pub struct StoreGlueState {
    pub purchase_states: HashMap<StoreGuid, VasPurchaseRecord>,
    pub queue_characters: Vec<VasQueueCharacter>,
    /// Resolved aliases are explicit inputs. General GUID grammar is UNKNOWN.
    pub resolved_guid_aliases: HashMap<Vec<u8>, StoreGuid>,
    pub disconnect_on_logout: bool,
    pub vas_product_ready: bool,
    pub connection_present: bool,
    /// Simulator observation counter; native width/overflow behavior is UNKNOWN.
    pub request_sequence: u64,
    pub submissions: Vec<StoreRequest>,
    pub completion_data: Vec<u8>,
    pub purchase_errors: Vec<i32>,
    pub callbacks: StoreCallbacks,
    /// Host supplies/clears this; no inferred consume-on-read behavior.
    pub pending_error: Option<String>,
    /// Explicit current-call native restriction metadata; not derived from Lua taint.
    pub restricted_execution: Option<RestrictedExecution>,
    /// Actual native argument-access mode when supplied; absent means unmodeled.
    pub secret_access_mode: Option<i32>,
}
impl StoreGlueState {
    pub fn prepare_queue_request(&mut self, guid: &str) {
        let Some(row) = self.queue_characters.iter().find(|row| row.guid == guid) else {
            return;
        };
        let correlation = row.correlation.clone();
        self.request_sequence = self
            .request_sequence
            .checked_add(1)
            .expect("simulator queue observation counter exhausted");
        if self.connection_present {
            self.submissions.push(StoreRequest::Queue {
                correlation,
                sequence: self.request_sequence,
            });
        }
    }
    pub fn request_refresh(&mut self) {
        if self.connection_present {
            self.submissions.push(StoreRequest::Refresh);
        }
    }
}
#[derive(Debug)]
pub struct CompletionInput {
    pub first: i32,
    pub second: i32,
    pub first_flag: bool,
    pub second_flag: bool,
    /// Opaque completion data, not interpreted as readiness or a purchase record.
    pub data: Vec<u8>,
}

pub fn receive_completion(env: &WowLuaEnv, input: CompletionInput) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        let store = &mut sim.store_glue;
        if input.first != 0 && input.second == 0 && input.first_flag && !input.second_flag {
            store.vas_product_ready = true;
        }
        store.completion_data = input.data;
        store.callbacks.purchase_complete
    };
    if notify {
        env.fire_event_with_args("STORE_VAS_PURCHASE_COMPLETE", &[])?;
    }
    Ok(())
}

pub fn receive_queue_response(
    env: &WowLuaEnv,
    correlation: &[u8],
    value: i32,
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    let guid = {
        let sim = env.state().borrow();
        let store = &sim.store_glue;
        if !store.callbacks.queue_status {
            return Ok(());
        }
        store
            .queue_characters
            .iter()
            .find(|row| row.correlation == correlation)
            .map(|row| row.guid.clone())
    };
    let Some(guid) = guid else { return Ok(()) };
    let guid = if guid.is_empty() {
        Val::Nil
    } else {
        env.lua_string(&guid)
    };
    env.fire_event_with_args(
        "VAS_CHARACTER_QUEUE_STATUS_UPDATE",
        &[guid, Val::Num(f64::from(value))],
    )
}

/// State-change payload is a host input, not a claimed native arity.
pub fn receive_bulk(
    env: &WowLuaEnv,
    records: Vec<(StoreGuid, VasPurchaseRecord)>,
    state_change_payload: &[Val],
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.purchase_states = records.into_iter().collect();
        sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", state_change_payload)?;
    }
    Ok(())
}

/// VERIFIED state0/1 clear-first, removal, then state-change notification.
/// Error-copy eligibility is an UNKNOWN predicate supplied explicitly by the host.
/// STORE_VAS_PURCHASE_ERROR is deliberately NOT dispatched: payload UNKNOWN (#1283).
pub fn receive_removal(
    env: &WowLuaEnv,
    guid: &str,
    errors: Vec<i32>,
    copy_eligible: bool,
    state_change_payload: &[Val],
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.purchase_errors.clear();
        // UNKNOWN copy predicate is an explicit host input; neutral default false.
        if copy_eligible {
            sim.store_glue.purchase_errors = errors;
        }
    }
    remove_purchase_record(env, guid, state_change_payload)
}

fn remove_purchase_record(
    env: &WowLuaEnv,
    guid: &str,
    state_change_payload: &[Val],
) -> crate::Result<()> {
    let notify = {
        let mut sim = env.state().borrow_mut();
        let removed = sim.store_glue.purchase_states.remove(guid).is_some();
        removed && sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", state_change_payload)?;
    }
    Ok(())
}

/// UNKNOWN service-mode predicate supplied explicitly; absent predicate means no update.
pub fn receive_service_update(
    env: &WowLuaEnv,
    guid: StoreGuid,
    record: VasPurchaseRecord,
    permitted: Option<bool>,
    state_change_payload: &[Val],
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    env.state().borrow_mut().store_glue.purchase_errors.clear();
    if permitted != Some(true) || record.state <= 1 {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.purchase_states.insert(guid, record);
        sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", state_change_payload)?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub enum RestrictedExecution {
    Addon {
        source: String,
        function: String,
        callback_present: bool,
    },
    Macro {
        function: String,
        callback_present: bool,
    },
}
