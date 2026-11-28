//! Traveler's Log data supplied by the simulator host, not a live rotation feed.
mod admin;
mod lua;
pub(crate) use admin::register_admin;
pub(crate) use lua::register;

pub(crate) fn active() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerksActivities {
    #[serde(rename = "activePerksMonth")]
    pub active_month: i32,
    #[serde(rename = "displayMonthName")]
    pub month_name: String,
    #[serde(rename = "secondsRemaining")]
    pub seconds_remaining: i64,
    pub activities: Vec<PerksActivity>,
    pub thresholds: Vec<PerksThreshold>,
    pub tags: Vec<String>,
    #[serde(rename = "pendingIDs")]
    pub pending_completion: Vec<i32>,
    #[serde(rename = "themePrefix")]
    pub theme_prefix: String,
    #[serde(rename = "trackedIDs")]
    pub tracked_ids: Vec<i32>,
}

impl Default for PerksActivities {
    fn default() -> Self {
        // INFERRED: retain the existing neutral bootstrap shape, with no live rows.
        Self {
            active_month: 1,
            month_name: String::new(),
            seconds_remaining: 0,
            activities: Vec::new(),
            thresholds: Vec::new(),
            tags: Vec::new(),
            pending_completion: Vec::new(),
            theme_prefix: String::new(),
            tracked_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerksActivity {
    #[serde(rename = "ID")]
    pub id: i32,
    #[serde(rename = "activityName")]
    pub name: String,
    pub description: String,
    #[serde(rename = "thresholdContributionAmount")]
    pub contribution: i32,
    pub completed: bool,
    #[serde(rename = "inProgress")]
    pub in_progress: bool,
    pub supersedes: i32,
    #[serde(rename = "uiPriority")]
    pub ui_priority: i32,
    #[serde(rename = "areAllConditionsMet")]
    pub all_conditions_met: bool,
    pub conditions: Vec<PerksCondition>,
    #[serde(rename = "eventName")]
    pub event_name: Option<String>,
    #[serde(rename = "eventStartTime")]
    pub event_start_time: Option<i64>,
    #[serde(rename = "eventEndTime")]
    pub event_end_time: Option<i64>,
    #[serde(rename = "requirementsList")]
    pub requirements: Vec<PerksRequirement>,
    #[serde(rename = "criteriaList")]
    pub criteria: Vec<PerksCriteria>,
    #[serde(rename = "tagNames")]
    pub tags: Vec<String>,
    #[serde(rename = "chatLink")]
    pub chat_link: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerksCondition {
    pub is_met: bool,
    pub text: String,
    pub ui_priority: i32,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerksRequirement {
    pub completed: bool,
    #[serde(rename = "requirementText")]
    pub text: String,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerksCriteria {
    #[serde(rename = "criteriaID")]
    pub id: i32,
    #[serde(rename = "requiredValue")]
    pub required_value: i64,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerksThreshold {
    #[serde(rename = "thresholdOrderIndex")]
    pub order_index: i32,
    #[serde(rename = "currencyAwardAmount")]
    pub currency_award: i32,
    #[serde(rename = "requiredContributionAmount")]
    pub required_contribution: i32,
    #[serde(rename = "pendingReward")]
    pub pending_reward: bool,
    #[serde(rename = "itemReward")]
    pub item_reward: Option<i32>,
}
