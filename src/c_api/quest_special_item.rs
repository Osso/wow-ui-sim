//! Host-supplied cache and object inputs for
//! `C_TooltipInfo.GetQuestLogSpecialItem`.
//!
//! Quest-ID request context, cached quest fields, and the selected inventory
//! object are separate because their native writers remain unknown.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct QuestSpecialItemInputs {
    /// One-based quest-log indexes resolved by the quest-ID request context.
    pub quest_ids_by_log_index: HashMap<i32, i32>,
    /// Special-item fields copied from the resolved quest-cache records.
    pub cached_quest_records: HashMap<i32, QuestSpecialItemCacheRecord>,
    /// Existing inventory item objects selected by the native item predicates.
    pub selected_item_objects: HashMap<i32, QuestSpecialItemObject>,
    /// Host-marked cache records that have not completed their lookup.
    pub not_ready_cache_records: HashSet<i32>,
    /// Request tokens from exact quest-key context matches; absence is the zero token.
    pub request_tokens_by_quest_id: HashMap<i32, QuestSpecialItemRequestToken>,
    /// Explicit host record arrival and destruction inputs, in supplied order.
    pub host_cache_updates: Vec<QuestSpecialItemCacheUpdate>,
    /// Pending callback entries, grouped by their owning quest-cache record.
    pub pending_callbacks: HashMap<i32, Vec<QuestSpecialItemPendingCallback>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QuestSpecialItemRequestToken {
    /// Opaque request token words. The marker is supplied separately by the host.
    pub words: [u64; 2],
    pub high_part_marker: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestSpecialItemCallback {
    QuestRecordData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestSpecialItemPendingCallback {
    pub callback: QuestSpecialItemCallback,
    pub token: QuestSpecialItemRequestToken,
    /// The quest-log tooltip caller supplies an empty extra context.
    pub extra_context: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum QuestSpecialItemCacheUpdate {
    RecordArrived {
        quest_id: i32,
        record: QuestSpecialItemCacheRecord,
        success: bool,
    },
    RecordDestroyed {
        quest_id: i32,
    },
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
