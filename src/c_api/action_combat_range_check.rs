//! WM93 PTR 70668 counted action-combat channel; pinned UI is 70493.

use std::collections::HashMap;

/// Explicit simulator input, not a computed geometric or target-selection result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SuppliedCombatRangeResult {
    pub in_range: bool,
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatRangeRegistration {
    pub references: i32,
    pub cached_result: Option<SuppliedCombatRangeResult>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionCombatRangeState {
    /// Native converted keys, including -1 and 408 for ordinary slots 0 and 409.
    pub registrations: HashMap<i32, CombatRangeRegistration>,
    /// No snapshot means no supplied evaluation. Never invent a range Boolean.
    pub supplied_results: HashMap<i32, SuppliedCombatRangeResult>,
}
