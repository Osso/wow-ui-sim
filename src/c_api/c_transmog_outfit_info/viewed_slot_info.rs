//! Explicit (slot, type, option) inputs; no default outfit or query producer.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewedOutfitSlotInfo {
    pub transmog_id: i64,
    pub display_type: i32,
    pub is_transmogrified: bool,
    pub has_pending: bool,
    pub is_pending_collected: bool,
    pub can_transmogrify: bool,
    pub warning: i32,
    pub warning_text: String,
    pub error: i32,
    pub error_text: String,
    pub texture: Option<i64>,
    pub sheathe_category: i32,
}
