//! Quest-log state for `C_QuestLog.*` probes.
//!
//! The `quest_log: Vec<u32>` in `SimState` is the lightweight
//! accept/abandon list used by `quest_verbs.rs`; `QuestLogState` carries
//! the rich metadata surfaced to the 17 ported probe methods.

use super::GossipQuestRow;
use crate::c_api::quest_objectives::QuestObjectiveProgress;
use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;

#[cfg(feature = "client-wrath")]
const WRATH_DAILY_QUEST_LIMIT: usize = 25;

/// Rich quest-log entry used by `C_QuestLog.*` probes.
#[derive(Debug, Clone)]
pub struct QuestLogEntry {
    pub quest_id: i32,
    pub title: String,
    pub level: i32,
    /// Supplied legacy quest-log group recommendation; absence means no group.
    pub suggested_group_size: Option<u32>,
    pub is_complete: bool,
    pub is_failed: bool,
    pub is_meta: bool,
    /// Supplied account-wide classification, if known.
    pub is_account_quest: Option<bool>,
    pub is_world_quest: bool,
    pub is_replayable: bool,
    pub is_flagged_completed: bool,
    pub map_id: Option<i32>,
    /// Normalised map x/y (0.0–1.0).
    pub waypoint: Option<(f64, f64)>,
    /// `tagID` value for `GetQuestTagInfo`.
    pub tag_id: Option<i32>,
    /// Theme key returned by `GetQuestDetailsTheme`.
    pub details_theme: Option<String>,
    /// Optional spell objective shown in `QuestInfo_ShowSpecialObjectives`.
    pub criteria_spell_id: Option<i32>,
    pub criteria_spell_name: Option<String>,
    pub criteria_spell_texture: Option<String>,
    pub criteria_spell_finished: bool,
    /// Item rewards iterated by `GetNumQuestLogRewards` /
    /// `GetQuestLogRewardInfo`. Empty for quests with no item rewards.
    pub reward_items: Vec<QuestRewardItem>,
    /// Supplied dialog reward choices; empty means no choices.
    pub reward_choices: Vec<QuestRewardItem>,
    /// Supplied Wrath rewards; absent spell and zero points mean no reward.
    pub reward_spell: Option<QuestRewardSpell>,
    pub reward_arena_points: u32,
    pub reward_talents: u32,
    /// Resolved faction ID and amount, in the units consumed by FrameXML.
    pub reward_factions: Vec<(i32, i32)>,
    /// Currency rewards returned by
    /// `C_QuestLog.GetQuestRewardCurrencies`. Empty for quests with no
    /// currency rewards.
    pub currency_rewards: Vec<QuestRewardCurrency>,
}

/// One item-shaped reward entry surfaced by `GetQuestLogRewardInfo`.
/// `quality` is the `Enum.ItemQuality` ordinal (0=Poor … 7=Heirloom).
#[derive(Debug, Clone)]
pub struct QuestRewardItem {
    pub name: String,
    pub texture: String,
    pub count: i32,
    pub quality: i32,
    pub is_usable: bool,
}

/// Spell presentation returned to the original Wrath reward consumer.
#[derive(Debug, Clone)]
pub struct QuestRewardSpell {
    pub texture: String,
    pub name: String,
    pub is_trade_skill: bool,
    pub is_learned: bool,
}

/// One currency-reward entry surfaced by
/// `C_QuestLog.GetQuestRewardCurrencies`. `texture` is a Blizzard-style
/// `Interface\Icons\…` path; the dialog passes it through `SetTexture`.
/// `base_reward_amount` is the pre-bonus value retail surfaces only
/// when known — `None` means "no base/bonus distinction for this
/// currency".
#[derive(Debug, Clone)]
pub struct QuestRewardCurrency {
    pub currency_id: i32,
    pub name: String,
    pub texture: String,
    pub total_reward_amount: i32,
    pub base_reward_amount: Option<i32>,
}

/// Backing state for `C_QuestLog.*` probes.
#[derive(Debug, Clone, Default)]
pub struct QuestLogState {
    /// Supplied quest-to-campaign associations, including completed quests.
    pub campaign_ids: HashMap<i32, i32>,
    pub entries: Vec<QuestLogEntry>,
    pub additional_highlights: HashMap<i32, crate::c_api::c_quest_log::QuestAdditionalHighlights>,
    /// Build-12340 manual append order; IDs survive row reorder until log refresh.
    #[cfg(feature = "client-wrath")]
    pub manual_watch_ids: Vec<i32>,
    pub objective_progress: HashMap<i32, QuestObjectiveProgress>,
    /// Explicit quest-dialog materials; absence leaves the original UI default.
    pub background_materials: HashMap<i32, String>,
    pub readiness: crate::c_api::quest_readiness::QuestReadinessState,
    pub offer: crate::c_api::c_quest_offer::QuestOfferState,
    /// Shared prepared output: readers do not implicitly refresh on selection.
    pub processed_reward_factions: Vec<(i32, i32)>,
    /// Player daily fields, separate from active rows and completion history.
    #[cfg(feature = "client-wrath")]
    pub daily_quest_ids: [u32; WRATH_DAILY_QUEST_LIMIT],
    /// Quest IDs already turned in / permanently completed.
    /// Drives `GetAllCompletedQuestIDs` and `IsQuestFlaggedCompleted`.
    pub completed_quest_ids: HashSet<i32>,
    /// Simulator acceptance receipt, not native inventory acquisition.
    pub turn_in_choices: HashMap<i32, usize>,
}

impl QuestLogState {
    /// Refresh-time cleanup, deliberately not part of the abandon response.
    #[cfg(feature = "client-wrath")]
    pub fn prune_manual_watches(&mut self) {
        let entries = &self.entries;
        self.manual_watch_ids
            .retain(|id| entries.iter().any(|entry| entry.quest_id == *id));
    }

    pub fn process_reward_factions(&mut self, quest_id: Option<u32>) {
        self.processed_reward_factions = self
            .entries
            .iter()
            .find(|entry| Some(entry.quest_id as u32) == quest_id)
            .map(|entry| entry.reward_factions.clone())
            .unwrap_or_default();
    }

    /// Remove active membership and auxiliary state without completing the quest.
    pub fn abandon(&mut self, quest_id: i32) -> bool {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.quest_id != quest_id);
        if self.entries.len() == before {
            return false;
        }
        self.objective_progress.remove(&quest_id);
        self.background_materials.remove(&quest_id);
        self.readiness.remove_quest(quest_id);
        self.offer.major_faction_rewards.remove(&quest_id);
        true
    }

    /// Authored simulator transition, not native validation/reward behavior.
    /// History alone does not block a represented completed row (including seeds).
    pub fn turn_in(&mut self, quest_id: i32) -> bool {
        let eligible = quest_id > 0
            && self
                .entries
                .iter()
                .any(|entry| entry.quest_id == quest_id && entry.is_complete && !entry.is_failed);
        if !eligible {
            return false;
        }
        self.entries.retain(|entry| entry.quest_id != quest_id);
        self.objective_progress.remove(&quest_id);
        self.background_materials.remove(&quest_id);
        self.readiness.remove_quest(quest_id);
        self.offer.major_faction_rewards.remove(&quest_id);
        self.completed_quest_ids.insert(quest_id);
        true
    }

    /// Snapshot the selected offer, retaining known metadata for this exact quest.
    /// Unknown ordinary offers use authored simulator policy, not native defaults.
    pub fn snapshot_gossip_offer(
        &self,
        offer: &GossipQuestRow,
    ) -> Result<QuestLogEntry, TryFromIntError> {
        let quest_id = i32::try_from(offer.quest_id)?;
        let mut snapshot = self
            .entries
            .iter()
            .find(|entry| entry.quest_id == quest_id)
            .cloned()
            .unwrap_or_else(|| ordinary_offer_entry(quest_id, offer));
        snapshot.title = offer.title.clone();
        snapshot.level = offer.quest_level;
        snapshot.is_complete = offer.is_complete == Some(true);
        snapshot.is_meta = offer.is_meta;
        snapshot.is_flagged_completed =
            snapshot.is_flagged_completed || self.completed_quest_ids.contains(&quest_id);
        Ok(snapshot)
    }

    pub fn seeded() -> Self {
        Self {
            campaign_ids: HashMap::new(),
            entries: vec![
                lost_expedition_entry(),
                defending_the_gates_entry(),
                glittering_geodes_entry(),
                earthen_relic_recovery_entry(),
            ],
            #[cfg(feature = "client-wrath")]
            manual_watch_ids: Vec::new(),
            objective_progress: crate::c_api::quest_objectives::seeded_progress(),
            background_materials: HashMap::new(),
            additional_highlights: HashMap::new(),
            readiness: Default::default(),
            offer: Default::default(),
            processed_reward_factions: Vec::new(),
            #[cfg(feature = "client-wrath")]
            daily_quest_ids: [0; WRATH_DAILY_QUEST_LIMIT],
            completed_quest_ids: seeded_completed_quest_ids(),
            turn_in_choices: HashMap::new(),
        }
    }
}

/// Explicit initial state for simulator-authored ordinary gossip offers.
/// POI polygons supply a map, never a quest-giver waypoint; other unsupported
/// metadata stays absent and this scenario has no item/currency rewards.
fn ordinary_offer_entry(quest_id: i32, offer: &GossipQuestRow) -> QuestLogEntry {
    QuestLogEntry {
        quest_id,
        title: offer.title.clone(),
        level: offer.quest_level,
        suggested_group_size: None,
        is_complete: offer.is_complete == Some(true),
        is_failed: false,
        is_meta: offer.is_meta,
        is_account_quest: None,
        is_world_quest: false,
        is_replayable: false,
        is_flagged_completed: false,
        map_id: crate::quest_poi_blobs::get_quest_blobs(offer.quest_id)
            .first()
            .map(|blob| blob.map_id as i32),
        waypoint: None,
        tag_id: None,
        details_theme: None,
        criteria_spell_id: None,
        criteria_spell_name: None,
        criteria_spell_texture: None,
        criteria_spell_finished: false,
        reward_items: Vec::new(),
        reward_choices: Vec::new(),
        reward_spell: None,
        reward_arena_points: 0,
        reward_talents: 0,
        reward_factions: Vec::new(),
        currency_rewards: Vec::new(),
    }
}

/// Shared field defaults for seeded entries: level 80, Khaz Algar map
/// (2248), tag 0, all bool flags false, no waypoint / details theme.
/// Per-quest builders override the distinctive fields via struct-update
/// syntax.
fn seed_quest_defaults() -> QuestLogEntry {
    QuestLogEntry {
        quest_id: 0,
        title: String::new(),
        level: 80,
        suggested_group_size: None,
        is_complete: false,
        is_failed: false,
        is_meta: false,
        is_account_quest: None,
        is_world_quest: false,
        is_replayable: false,
        is_flagged_completed: false,
        map_id: Some(2248),
        waypoint: None,
        tag_id: Some(0),
        details_theme: None,
        criteria_spell_id: None,
        criteria_spell_name: None,
        criteria_spell_texture: None,
        criteria_spell_finished: false,
        reward_items: Vec::new(),
        reward_choices: Vec::new(),
        reward_spell: None,
        reward_arena_points: 0,
        reward_talents: 0,
        reward_factions: Vec::new(),
        currency_rewards: Vec::new(),
    }
}

fn lost_expedition_entry() -> QuestLogEntry {
    QuestLogEntry {
        quest_id: 80000,
        title: "The Lost Expedition".into(),
        waypoint: Some((0.45, 0.35)),
        ..seed_quest_defaults()
    }
}

fn defending_the_gates_entry() -> QuestLogEntry {
    QuestLogEntry {
        quest_id: 80001,
        title: "Defending the Gates".into(),
        is_complete: true,
        ..seed_quest_defaults()
    }
}

fn earthen_relic_recovery_entry() -> QuestLogEntry {
    QuestLogEntry {
        quest_id: 90101,
        title: "Earthen Relic Recovery".into(),
        is_world_quest: true,
        is_replayable: true,
        waypoint: Some((0.62, 0.58)),
        tag_id: Some(2),
        ..seed_quest_defaults()
    }
}

fn glittering_geodes_entry() -> QuestLogEntry {
    QuestLogEntry {
        quest_id: 90001,
        title: "Glittering Geodes".into(),
        is_world_quest: true,
        is_replayable: true,
        map_id: Some(2025),
        waypoint: Some((0.52, 0.63)),
        tag_id: Some(2),
        ..seed_quest_defaults()
    }
}

fn seeded_completed_quest_ids() -> HashSet<i32> {
    HashSet::from([79999, 80001])
}
