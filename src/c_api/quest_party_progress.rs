//! Host-supplied rows for Retail `C_TooltipInfo.GetQuestPartyProgress`.
//!
//! The simulator has no native quest-progress service. An absent snapshot is
//! not replaced with inferred party or objective data.

#[derive(Debug, Clone, Default)]
pub struct QuestPartyProgressSnapshot {
    /// Title text and RGB supplied for the optional type-17 row.
    pub title: Option<QuestPartyProgressText>,
    pub participants: Vec<QuestPartyProgressParticipant>,
}

#[derive(Debug, Clone)]
pub struct QuestPartyProgressParticipant {
    /// The host supplies the participant grouping and name condition.
    pub heading: Option<QuestPartyProgressHeading>,
    /// The host supplies whether this participant is removed by the native
    /// `ignoreActivePlayer` query flag; no identity predicate is inferred.
    pub exclude_when_ignoring_active_player: bool,
    pub rows: Vec<QuestPartyProgressRow>,
}

#[derive(Debug, Clone)]
pub struct QuestPartyProgressHeading {
    pub name: String,
    pub guid: String,
    pub color: (f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct QuestPartyProgressText {
    pub text: String,
    pub color: (f64, f64, f64),
}

#[derive(Debug, Clone)]
pub struct QuestPartyProgressRow {
    pub text: String,
    pub color: (f64, f64, f64),
    pub completed: bool,
    pub num_required: Option<i32>,
    pub num_fulfilled: Option<i32>,
    pub progress_bar_percent: Option<f64>,
}
