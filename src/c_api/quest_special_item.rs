//! Host-supplied cache and object inputs for
//! `C_TooltipInfo.GetQuestLogSpecialItem`.
//!
//! Quest-ID request context, cached quest fields, and the selected inventory
//! object are separate because their native writers remain unknown.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct QuestSpecialItemInputs {
    /// One-based quest-log indexes resolved by the quest-ID request context.
    pub quest_ids_by_log_index: HashMap<i32, i32>,
    /// Special-item fields copied from the resolved quest-cache records.
    pub cached_quest_records: HashMap<i32, QuestSpecialItemCacheRecord>,
    /// Existing inventory item objects selected by the native item predicates.
    pub selected_item_objects: HashMap<i32, QuestSpecialItemObject>,
}

#[derive(Debug, Clone, Default)]
pub struct QuestSpecialItemCacheRecord {
    /// Primary cache item ID; zero and absence do not provide an item ID.
    pub special_item_id: Option<u32>,
    /// Native marker that enables checking up to four candidate IDs.
    pub has_candidate_items: bool,
    /// Additional IDs from the cache record, in native candidate order.
    pub candidate_item_ids: [Option<u32>; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct QuestSpecialItemObject {
    pub item_id: u32,
    /// Existing simulator bag location for the selected object.
    pub bag: i32,
    pub slot: i32,
}
