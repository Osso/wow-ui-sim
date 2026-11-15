//! Host-supplied cache and object inputs for
//! `C_TooltipInfo.GetQuestLogSpecialItem`.
//!
//! Cache writers remain host inputs. The Retail callback list models the
//! verified request, arrival and record-destruction paths.

use std::collections::{HashMap, HashSet};

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::event::{Event, EventArg};
use crate::lua_api::state::SimState;

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
    /// INFERRED host condition for the conditional QUEST_AUTOCOMPLETE callback.
    pub autocomplete_notification_quest_ids: HashSet<i32>,
}

impl QuestSpecialItemInputs {
    pub(crate) fn save_quest_record_callback(&mut self, quest_id: i32) {
        let token = self
            .request_tokens_by_quest_id
            .get(&quest_id)
            .copied()
            .unwrap_or_default();
        let incoming = QuestSpecialItemPendingCallback {
            callback: QuestSpecialItemCallback::QuestRecordData,
            token,
            extra_context: Vec::new(),
        };
        let callbacks = self.pending_callbacks.entry(quest_id).or_default();
        if callbacks
            .iter()
            .any(|saved| is_duplicate_callback(saved, &incoming))
        {
            return;
        }
        callbacks.push(incoming);
    }
}

fn is_duplicate_callback(
    saved: &QuestSpecialItemPendingCallback,
    incoming: &QuestSpecialItemPendingCallback,
) -> bool {
    let same_callback_and_context =
        saved.callback == incoming.callback && saved.extra_context == incoming.extra_context;
    let token_matches =
        !incoming.token.high_part_marker || saved.token.words == incoming.token.words;
    same_callback_and_context && token_matches
}

const CALLBACK_MODE_READY: u8 = 0;
const CALLBACK_MODE_UNREADY_REMOVAL: u8 = 1;
const CALLBACK_MODE_DESTROYED: u8 = 4;

pub(crate) fn process_host_cache_updates(sim: &mut SimState) {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let updates = std::mem::take(
        &mut sim
            .quest_log_entries
            .quest_special_item_inputs
            .host_cache_updates,
    );
    for update in updates {
        match update {
            QuestSpecialItemCacheUpdate::RecordArrived {
                quest_id,
                record,
                success,
            } => process_record_arrival(sim, quest_id, record, success),
            QuestSpecialItemCacheUpdate::RecordDestroyed { quest_id } => {
                process_record_destruction(sim, quest_id)
            }
        }
    }
}

fn process_record_arrival(
    sim: &mut SimState,
    quest_id: i32,
    record: QuestSpecialItemCacheRecord,
    success: bool,
) {
    let callbacks = {
        let inputs = &mut sim.quest_log_entries.quest_special_item_inputs;
        inputs.not_ready_cache_records.remove(&quest_id);
        inputs.cached_quest_records.insert(quest_id, record);
        inputs
            .pending_callbacks
            .get(&quest_id)
            .cloned()
            .unwrap_or_default()
    };
    invoke_pending_callbacks(sim, quest_id, &callbacks, CALLBACK_MODE_READY, success);
    sim.quest_log_entries
        .quest_special_item_inputs
        .pending_callbacks
        .remove(&quest_id);
}

fn process_record_destruction(sim: &mut SimState, quest_id: i32) {
    let (was_not_ready, callbacks) = {
        let inputs = &sim.quest_log_entries.quest_special_item_inputs;
        (
            inputs.not_ready_cache_records.contains(&quest_id),
            inputs
                .pending_callbacks
                .get(&quest_id)
                .cloned()
                .unwrap_or_default(),
        )
    };
    if was_not_ready {
        invoke_pending_callbacks(
            sim,
            quest_id,
            &callbacks,
            CALLBACK_MODE_UNREADY_REMOVAL,
            false,
        );
    }
    invoke_pending_callbacks(sim, quest_id, &callbacks, CALLBACK_MODE_DESTROYED, false);
    let inputs = &mut sim.quest_log_entries.quest_special_item_inputs;
    inputs.pending_callbacks.remove(&quest_id);
    inputs.not_ready_cache_records.remove(&quest_id);
    inputs.cached_quest_records.remove(&quest_id);
}

fn invoke_pending_callbacks(
    sim: &mut SimState,
    quest_id: i32,
    callbacks: &[QuestSpecialItemPendingCallback],
    mode: u8,
    success: bool,
) {
    for callback in callbacks {
        match callback.callback {
            QuestSpecialItemCallback::QuestRecordData => quest_record_data_callback(
                sim,
                quest_id,
                callback.token,
                &callback.extra_context,
                mode,
                success,
            ),
        }
    }
}

fn quest_record_data_callback(
    sim: &mut SimState,
    quest_id: i32,
    _token: QuestSpecialItemRequestToken,
    _extra_context: &[u8],
    mode: u8,
    success: bool,
) {
    if mode != CALLBACK_MODE_READY || !success {
        return;
    }
    queue_quest_log_update(sim);
    if sim
        .quest_log_entries
        .quest_special_item_inputs
        .autocomplete_notification_quest_ids
        .contains(&quest_id)
    {
        queue_quest_autocomplete(sim, quest_id);
    }
}

fn queue_quest_log_update(sim: &mut SimState) {
    if sim
        .events
        .pending()
        .iter()
        .any(|event| event.name == "QUEST_LOG_UPDATE")
    {
        return;
    }
    sim.events.push_simple("QUEST_LOG_UPDATE");
}

fn queue_quest_autocomplete(sim: &mut SimState, quest_id: i32) {
    // INFERRED (R88u.u3): the numeric event payload is the quest key.
    sim.events.push(Event {
        name: "QUEST_AUTOCOMPLETE".to_owned(),
        args: vec![EventArg::Number(f64::from(quest_id))],
    });
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
