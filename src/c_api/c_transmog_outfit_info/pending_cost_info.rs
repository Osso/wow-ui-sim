//! Explicit host snapshot only; no pricing or transaction lifecycle.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingTransmogCost {
    pub cost: u64,
    pub modifier_flags: u32,
}
