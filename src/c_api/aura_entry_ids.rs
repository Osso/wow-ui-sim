//! Host-supplied aura instance identifiers for the next encounter/M+/PvP entry.

use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct AuraEntryIds {
    /// One replacement per stored aura: player, then party members in roster order;
    /// each member's helpful auras precede harmful auras. No random/default allocator.
    pub pending: Option<Vec<i32>>,
    /// IDs retired by entry transitions cannot become live again on later entries.
    pub(crate) retired: HashSet<i32>,
}
