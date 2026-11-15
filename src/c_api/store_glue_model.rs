//! R88q host-supplied VAS state. No fabricated server records or automatic responses.
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
#[derive(Debug)]
pub struct CompletionInput {
    pub first: i32,
    pub second: i32,
    pub first_flag: bool,
    pub second_flag: bool,
    /// Opaque completion data, not interpreted as readiness or a purchase record.
    pub data: Vec<u8>,
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
