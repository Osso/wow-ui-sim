//! Host-supplied initiative records. No native catalog, persistence or progression engine.

use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct NeighborhoodInitiativeState {
    pub active_neighborhood: String,
    pub viewing_neighborhood: String,
    pub neighborhoods: BTreeMap<String, NeighborhoodInitiativeInfo>,
    pub activity_logs: BTreeMap<String, InitiativeActivityLogInfo>,
    pub required_level: i32,
    /// Entitlement is separate from the level gate, as the dashboard probes both.
    pub player_has_access: bool,
    /// Neighborhoods for which the current party satisfies the neighborhood-group rule.
    pub group_neighborhoods: BTreeSet<String>,
    pub(crate) pending_requests: BTreeMap<u64, String>,
}

impl Default for NeighborhoodInitiativeState {
    fn default() -> Self {
        Self {
            active_neighborhood: String::new(),
            viewing_neighborhood: String::new(),
            neighborhoods: BTreeMap::new(),
            activity_logs: BTreeMap::new(),
            // INFERRED: unconfigured host policy permits level 1; no native default claimed.
            required_level: 1,
            player_has_access: false,
            group_neighborhoods: BTreeSet::new(),
            pending_requests: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeighborhoodInitiativeInfo {
    pub is_loaded: bool,
    #[serde(rename = "neighborhoodGUID")]
    pub neighborhood_guid: String,
    #[serde(rename = "initiativeID")]
    pub initiative_id: i32,
    #[serde(rename = "currentCycleID")]
    pub current_cycle_id: i32,
    pub progress_required: i32,
    pub current_progress: i32,
    pub player_total_contribution: i32,
    pub duration: i64,
    pub tasks: Vec<InitiativeTaskInfo>,
    pub milestones: Vec<InitiativeMilestoneInfo>,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeTaskInfo {
    #[serde(rename = "ID")]
    pub id: i32,
    pub task_name: String,
    pub description: String,
    pub progress_contribution_amount: i32,
    pub tracked: bool,
    pub supersedes: i32,
    pub times_completed: i32,
    pub completed: bool,
    pub in_progress: bool,
    pub task_type: i32,
    pub sort_order: i32,
    #[serde(rename = "rewardQuestID")]
    pub reward_quest_id: i32,
    pub requirements_list: Vec<CriteriaRequirement>,
    pub criteria_list: Vec<CriteriaRequiredValue>,
    /// Host-supplied native link, excluded from the documented task structure.
    #[serde(skip)]
    pub chat_link: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriteriaRequirement {
    pub completed: bool,
    pub requirement_text: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriteriaRequiredValue {
    #[serde(rename = "criteriaID")]
    pub criteria_id: i32,
    pub required_value: i32,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeMilestoneInfo {
    pub milestone_order_index: i32,
    pub required_contribution_amount: i32,
    pub rewards: Vec<InitiativeMilestoneRewardInfo>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeMilestoneRewardInfo {
    pub title: String,
    pub description: String,
    #[serde(rename = "decorID")]
    pub decor_id: i32,
    pub decor_quantity: i32,
    pub favor: i32,
    pub money: i64,
    #[serde(rename = "rewardQuestID")]
    pub reward_quest_id: i32,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeActivityLogInfo {
    pub is_loaded: bool,
    #[serde(rename = "neighborhoodGUID")]
    pub neighborhood_guid: String,
    pub next_update_time: i64,
    pub task_activity: Vec<InitiativeActivityLogEntry>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitiativeActivityLogEntry {
    #[serde(rename = "taskID")]
    pub task_id: i32,
    pub player_name: String,
    pub task_name: String,
    pub completion_time: i64,
    pub amount: i32,
}
