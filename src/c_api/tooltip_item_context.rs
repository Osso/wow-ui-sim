//! Explicit host key for inferred item-level tooltip variants; no native mapping.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemTooltipContext {
    pub item_id: u32,
    pub item_context: Option<u32>,
    pub treasure_context_level: Option<u32>,
}
