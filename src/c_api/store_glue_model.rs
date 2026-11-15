//! R88q host-supplied VAS state. No fabricated server records or automatic responses.
use crate::lua_api::WowLuaEnv;
use rilua::Val;
use std::collections::HashMap;

/// Host display text; ordinary parsing supplies lookup identity (R88w.w2, #1310).
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
    pub purchase_error: bool,
    pub queue_status: bool,
    pub state_changed: bool,
}
#[derive(Debug, Default)]
pub struct StoreGlueState {
    pub purchase_states: HashMap<StoreGuid, VasPurchaseRecord>,
    pub queue_characters: Vec<VasQueueCharacter>,
    /// VERIFIED exact-case Opaque- u32 aliases; missing/None resumes ordinary parsing.
    pub opaque_guid_aliases: HashMap<u32, Option<super::store_guid::Identity>>,
    pub disconnect_on_logout: bool,
    pub vas_product_ready: bool,
    pub connection_present: bool,
    /// Simulator observation counter; native width/overflow behavior is UNKNOWN.
    pub request_sequence: u64,
    pub submissions: Vec<StoreRequest>,
    pub completion_data: Vec<u8>,
    pub purchase_errors: Vec<i32>,
    /// UNKNOWN rejection cause; explicit host entry-check result (#1310), default false.
    pub vas_errors_entry_rejected: bool,
    pub callbacks: StoreCallbacks,
    /// Host supplies/clears this; no inferred consume-on-read behavior.
    pub pending_error: Option<String>,
    /// Explicit current-call native restriction metadata; not derived from Lua taint.
    pub restricted_execution: Option<RestrictedExecution>,
    /// Actual native argument-access mode when supplied; absent means unmodeled.
    pub secret_access_mode: Option<i32>,
}
impl StoreGlueState {
    /// VERIFIED ordinary grammar; Opaque alias lookup belongs only to API arguments.
    pub fn resolve_guid(&self, bytes: &[u8]) -> super::store_guid::Outcome {
        use super::store_guid::{Outcome, opaque_key, parse};
        if let Some(identity) = opaque_key(bytes)
            .and_then(|key| self.opaque_guid_aliases.get(&key))
            .and_then(Option::as_ref)
        {
            return Outcome::Known(identity.clone());
        }
        parse(bytes)
    }
    pub fn purchase_record(&self, guid: &super::store_guid::Outcome) -> VasPurchaseRecord {
        self.purchase_states
            .iter()
            .find(|(text, _)| guid_matches(guid, text))
            .map(|(_, record)| record.clone())
            .unwrap_or_default()
    }
    pub fn remove_record(&mut self, text: &str) -> bool {
        let guid = super::store_guid::parse(text.as_bytes());
        let key = self
            .purchase_states
            .keys()
            .find(|key| guid_matches(&guid, key))
            .cloned();
        key.is_some_and(|key| self.purchase_states.remove(&key).is_some())
    }
    pub fn insert_record(&mut self, text: StoreGuid, record: VasPurchaseRecord) -> bool {
        // UNKNOWN host strings are not purchase identities and cannot match each other.
        if matches!(
            super::store_guid::parse(text.as_bytes()),
            super::store_guid::Outcome::Unknown
        ) {
            return false;
        }
        self.remove_record(&text);
        self.purchase_states.insert(text, record);
        true
    }
    pub fn replace_records(&mut self, records: Vec<(StoreGuid, VasPurchaseRecord)>) {
        self.purchase_states.clear();
        for (text, record) in records {
            self.insert_record(text, record);
        }
    }

    /// VERIFIED R88w.w1 (#1308): sort/compact before snapshot; retain source count.
    pub fn vas_errors_snapshot(&mut self) -> Vec<i32> {
        self.purchase_errors.sort_unstable();
        let mut distinct = 0;
        for index in 0..self.purchase_errors.len() {
            let value = self.purchase_errors[index];
            if distinct == 0 || self.purchase_errors[distinct - 1] != value {
                self.purchase_errors[distinct] = value;
                distinct += 1;
            }
        }
        self.purchase_errors[..distinct].to_vec()
    }

    pub fn prepare_queue_request(&mut self, guid: &super::store_guid::Outcome) {
        let Some(row) = self
            .queue_characters
            .iter()
            .find(|row| guid_matches(guid, &row.guid))
        else {
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

/// VERIFIED R88w.w3: full replacement emits one zero-payload notification.
pub fn receive_bulk(
    env: &WowLuaEnv,
    records: Vec<(StoreGuid, VasPurchaseRecord)>,
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.replace_records(records);
        sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", &[])?;
    }
    Ok(())
}

/// VERIFIED state0/1 clear-first, removal, then state-change notification.
/// VERIFIED R88w.w4: this state0/1 path copies a nonempty, first-nonzero list.
/// R88w.w1/w4: gated zero-payload error notification precedes record search/removal.
pub fn receive_removal(env: &WowLuaEnv, guid: &str, errors: Vec<i32>) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.purchase_errors = Vec::new();
        if errors.first().is_some_and(|value| *value != 0) {
            sim.store_glue.purchase_errors = errors;
            sim.store_glue.callbacks.purchase_error
        } else {
            false
        }
    };
    if notify {
        env.fire_event_with_args("STORE_VAS_PURCHASE_ERROR", &[])?;
    }
    remove_purchase_record(env, guid)
}

fn remove_purchase_record(env: &WowLuaEnv, guid: &str) -> crate::Result<()> {
    let notify = {
        let mut sim = env.state().borrow_mut();
        let removed = sim.store_glue.remove_record(guid);
        removed && sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", &[])?;
    }
    Ok(())
}

/// UNKNOWN service-mode predicate supplied explicitly; absent predicate means no update.
pub fn receive_service_update(
    env: &WowLuaEnv,
    guid: StoreGuid,
    record: VasPurchaseRecord,
    permitted: Option<bool>,
) -> crate::Result<()> {
    if !super::c_store_glue::enabled() {
        return Ok(());
    }
    env.state().borrow_mut().store_glue.purchase_errors = Vec::new();
    if permitted != Some(true) || record.state <= 1 {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        let stored = sim.store_glue.insert_record(guid, record);
        stored && sim.store_glue.callbacks.state_changed
    };
    if notify {
        env.fire_event_with_args("VAS_CHARACTER_STATE_CHANGED", &[])?;
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

// UNKNOWN inputs never match, including another UNKNOWN or an explicit empty record.
fn guid_matches(guid: &super::store_guid::Outcome, host: &str) -> bool {
    use super::store_guid::{Outcome, parse};
    match (guid, parse(host.as_bytes())) {
        (Outcome::Known(left), Outcome::Known(right)) => *left == right,
        (Outcome::KnownEmpty, Outcome::KnownEmpty) => true,
        _ => false,
    }
}
