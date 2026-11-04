//! WM92 PTR 70668 SpellRangeVisualization inputs and local stores.
//! The pinned PTR UI is 70493; this is not same-build native evidence.

use std::collections::HashMap;

/// A supplied action record and its optional successful visualization lookup.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionVisualizationRecord {
    pub identity: String,
    pub visualization_value: Option<u32>,
}

/// Empty inputs mean unavailable records/installed object, not native defaults.
/// Events, tooltip ownership, combat and taint policy remain UNKNOWN; no extra
/// effects are modeled beyond the recovered local stores.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpellRangeVisualizationState {
    pub installed_object_identity: Option<String>,
    pub records: HashMap<u32, ActionVisualizationRecord>,
    /// Native converted slot, zero-based. Leave preserves it.
    pub stored_slot: Option<u32>,
    pub value: u32,
    pub enabled: bool,
}
