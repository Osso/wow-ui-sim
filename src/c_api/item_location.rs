//! Host evidence for R88s ItemLocation inventory bounds.
//!
//! The simulator has no inventory-count or inventory-mode producer. Missing
//! values stay UNKNOWN; callers must reject locations that require them.

use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct R88sInventoryInputs {
    /// Host-supplied native player-inventory entry count. `None` is UNKNOWN.
    pub inventory_count: Option<i32>,
    /// Host-supplied native inventory-mode state. `None` is UNKNOWN.
    pub inventory_mode_active: Option<bool>,
    /// Host observations for special equipment slots when mode is active.
    /// Missing slots are UNKNOWN and reject.
    pub available_special_equipment_slots: HashSet<i32>,
}
