//! Host-supplied resolved item-object selection for
//! `C_TooltipInfo.GetQuestLogSpecialItem`.
//!
//! The native quest-cache and inventory/object writers are not modeled. An
//! absent selection is not reconstructed from a catalog item ID.

#[derive(Debug, Clone, Copy)]
pub struct QuestSpecialItemSelection {
    /// Item identity carried by the selected player-owned object.
    pub item_id: u32,
    /// Existing simulator bag location for that object.
    pub bag: i32,
    pub slot: i32,
}
