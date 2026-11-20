//! Traveler's Log data supplied by the simulator host, not a live rotation feed.
mod lua;
pub(crate) use lua::register;

pub(crate) fn active() -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

#[derive(Clone, Debug)]
pub struct PerksActivities {
    pub active_month: i32,
    pub month_name: String,
    pub seconds_remaining: i64,
    pub activities: Vec<PerksActivity>,
    pub thresholds: Vec<PerksThreshold>,
    pub tags: Vec<String>,
    pub pending_completion: Vec<i32>,
    pub theme_prefix: String,
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

#[derive(Clone, Debug, Default)]
pub struct PerksActivity {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub contribution: i32,
    pub completed: bool,
    pub in_progress: bool,
    pub supersedes: i32,
    pub ui_priority: i32,
    pub all_conditions_met: bool,
    pub conditions: Vec<PerksCondition>,
    pub event_name: Option<String>,
    pub event_start_time: Option<i64>,
    pub event_end_time: Option<i64>,
    pub requirements: Vec<PerksRequirement>,
    pub criteria: Vec<PerksCriteria>,
    pub tags: Vec<String>,
    pub chat_link: String,
}

#[derive(Clone, Debug, Default)]
pub struct PerksCondition {
    pub is_met: bool,
    pub text: String,
    pub ui_priority: i32,
}

#[derive(Clone, Debug, Default)]
pub struct PerksRequirement {
    pub completed: bool,
    pub text: String,
}

#[derive(Clone, Debug, Default)]
pub struct PerksCriteria {
    pub id: i32,
    pub required_value: i64,
}

#[derive(Clone, Debug, Default)]
pub struct PerksThreshold {
    pub order_index: i32,
    pub currency_award: i32,
    pub required_contribution: i32,
    pub pending_reward: bool,
    pub item_reward: Option<i32>,
}
