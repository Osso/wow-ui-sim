//! Mythic+, scenario, and death-recap substate types.

use std::collections::HashMap;

/// One affix in the active Mythic+ season. Drives
/// `C_MythicPlus.GetCurrentAffixes`.
#[derive(Debug, Clone)]
pub struct MythicPlusAffix {
    /// Dungeon Journal affix id (e.g. 9 = Tyrannical).
    pub id: i32,
    /// Season in which this affix rotates.
    pub season_id: i32,
}

/// Weekly best result for a specific Mythic+ map. Drives
/// `C_MythicPlus.GetWeeklyBestForMap`.
#[derive(Debug, Clone)]
pub struct MythicPlusWeeklyBest {
    /// Challenge-mode dungeon id.
    pub map_challenge_mode_id: i32,
    /// Keystone level completed this week.
    pub level: i32,
    /// Completion time in seconds.
    pub duration_sec: i32,
    /// Mythic+ score contribution for this run.
    pub score: f64,
    #[cfg(feature = "retail-12-0-7")]
    pub completion_date: Option<crate::c_api::c_mythic_plus_calendar::CalendarTime>,
    #[cfg(feature = "retail-12-0-7")]
    pub affix_ids: Vec<i32>,
    #[cfg(feature = "retail-12-0-7")]
    pub members: Vec<crate::c_api::c_mythic_plus_calendar::MythicPlusMember>,
}

/// One completed or in-progress Mythic+ run. Drives
/// `C_MythicPlus.GetRunHistory`.
#[derive(Debug, Clone)]
pub struct MythicPlusRun {
    /// Challenge-mode dungeon id.
    pub map_challenge_mode_id: i32,
    /// Keystone level.
    pub level: i32,
    /// True when the run was completed in time.
    pub completed: bool,
    /// Season the run belongs to.
    pub season: i32,
    /// Run score contribution.
    pub run_score: f64,
    /// Whether this run is from the current week.
    pub this_week: bool,
    /// Completion time in seconds.
    pub duration_sec: i32,
    #[cfg(feature = "retail-12-0-7")]
    pub completion_date: Option<crate::c_api::c_mythic_plus_calendar::CalendarTime>,
}

/// Backing state for `C_MythicPlus.*` probes.
#[derive(Debug, Clone)]
pub struct MythicPlusState {
    /// Active affixes for the current weekly rotation.
    pub current_affixes: Vec<MythicPlusAffix>,
    /// Current M+ season id (e.g. 14 = Dragonflight Season 4).
    pub current_season: i32,
    /// Keystone level of the key the player currently owns.
    /// 0 = no key.
    pub owned_keystone_level: i32,
    /// Run history for the player.
    pub run_history: Vec<MythicPlusRun>,
    /// Per-map weekly best, keyed by mapChallengeModeID.
    pub weekly_best_per_map: HashMap<i32, MythicPlusWeeklyBest>,
    #[cfg(feature = "retail-12-0-7")]
    pub season_best_per_map:
        HashMap<i32, (Option<MythicPlusWeeklyBest>, Option<MythicPlusWeeklyBest>)>,
    /// Whether a Mythic+ run is currently in progress.
    pub is_active: bool,
    /// Whether the weekly Mythic+ reward is available to claim.
    pub is_weekly_reward_available: bool,
}

impl Default for MythicPlusState {
    fn default() -> Self {
        Self {
            current_affixes: vec![MythicPlusAffix {
                id: 9,
                season_id: 14,
            }],
            current_season: 14,
            owned_keystone_level: 0,
            run_history: Vec::new(),
            weekly_best_per_map: HashMap::new(),
            #[cfg(feature = "retail-12-0-7")]
            season_best_per_map: HashMap::new(),
            is_active: false,
            is_weekly_reward_available: false,
        }
    }
}

/// One step within an active scenario. Drives
/// `C_ScenarioInfo.GetScenarioStepInfo(stepID)`.
#[derive(Debug, Clone)]
pub struct ScenarioStep {
    /// Scenario step id (1-based index used by `GetScenarioStepInfo`).
    pub step_id: i32,
    /// Display title shown in the scenario tracker.
    pub title: String,
    /// Longer description shown in the step panel.
    pub description: String,
    /// Number of objectives/criteria for this step.
    pub num_criteria: i32,
    /// Whether this step has been completed.
    pub completed: bool,
    /// True when this is an optional bonus step.
    pub is_bonus_step: bool,
    /// Quest rewarded upon completing a bonus step, if any.
    /// Drives `C_ScenarioInfo.GetScenarioBonusStepRewardQuestID`.
    pub bonus_reward_quest_id: Option<i32>,
}

/// Backing state for `C_ScenarioInfo.*` probes.
/// `in_scenario` defaults to false (no active scenario).
#[derive(Debug, Clone)]
pub struct ScenarioState {
    /// Whether the player is currently inside a scenario.
    pub in_scenario: bool,
    /// Display name of the scenario (e.g. "Assault on Violet Hold").
    pub name: String,
    /// Scenario id as reported by the server.
    pub scenario_id: i32,
    /// Current active step index (1-based).
    pub current_step: i32,
    /// Total number of steps in the scenario.
    pub num_steps: i32,
    /// Scenario type flag (matches `Enum.ScenarioType` values).
    pub scenario_type: i32,
    /// UI texture kit name for the scenario tracker background.
    pub texture_kit: String,
    /// Whether this is a tiered entrance scenario. Drives
    /// `C_ScenarioInfo.IsTieredEntranceScenario`.
    pub is_tiered_entrance: bool,
    /// Ordered list of steps. Indexed by `step_id - 1`.
    pub steps: Vec<ScenarioStep>,
    /// Supplied per-token Enemy Forces credit; absent rows have no result.
    pub unit_criteria: HashMap<String, crate::c_api::c_scenario_info::UnitCriteriaProgress>,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            in_scenario: false,
            name: String::new(),
            scenario_id: 0,
            current_step: 1,
            num_steps: 0,
            scenario_type: 0,
            texture_kit: String::new(),
            is_tiered_entrance: false,
            steps: Vec::new(),
            unit_criteria: HashMap::new(),
        }
    }
}

/// One killing-blow entry within a death recap event. Drives
/// `C_DeathRecap.GetKillingBlows`. The spell_id / ability_name /
/// caster_name / amount fields mirror the retail multiret documented
/// on Wowhead/WoWDB.
#[derive(Debug, Clone)]
pub struct KillingBlowInfo {
    /// Spell or ability ID responsible for the killing blow.
    pub spell_id: u32,
    /// Display name of the spell/ability (may be empty if unknown).
    pub ability_name: String,
    /// Name of the caster that delivered the blow.
    pub caster_name: String,
    /// Raw damage or healing amount for the killing hit.
    pub amount: i64,
    /// True when the hit was an overkill (damage > remaining health).
    pub is_overkill: bool,
}

/// One death event on the `SimState.death_recaps` list. Each entry
/// represents a single death and carries zero or more killing blows.
/// Drives `C_DeathRecap.GetKillingBlows` (blows for the most recent
/// entry) and `C_DeathRecap.GetMostRecentDeathRecap`.
#[derive(Debug, Clone)]
pub struct DeathRecapEntry {
    /// Internal recap id — mirrors the `recapID` concept used by
    /// `C_DeathRecap.GetRecapEvents`; 1-based.
    pub recap_id: u32,
    /// Zone / encounter name where the player died (informational).
    pub zone_name: String,
    /// Killing blows list (ordered from most damaging to least).
    pub killing_blows: Vec<KillingBlowInfo>,
}
