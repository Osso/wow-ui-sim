//! Explicit slot-keyed use-count snapshots; no acquisition or consumption.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionUseCountInfo {
    pub spell_id: u32,
    pub count: u32,
}
