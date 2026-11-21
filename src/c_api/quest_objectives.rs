//! Supplied objective snapshots. Catalogs bootstrap state; queries never reread them.

use std::collections::HashMap;

use crate::lua_api::sim_substates::QuestLogState;

mod queries;
pub(crate) use queries::{
    get_legacy_leaderboard, get_legacy_objective_count, get_legacy_quest_text, get_percentage,
    read_selected_objective_text, register,
};

pub const ENABLED: bool = cfg!(any(
    feature = "client-retail",
    feature = "client-wowforever"
));

#[derive(Debug, Clone, PartialEq)]
pub struct QuestObjective {
    pub text: String,
    pub objective_type: String,
    pub finished: bool,
    pub num_fulfilled: i32,
    pub num_required: i32,
    pub criteria_id: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct QuestObjectiveProgress {
    pub objectives: Vec<QuestObjective>,
    pub percentage: Option<f64>,
    /// Bootstrap cardinality only when catalog rows are unknown. Supply clears it.
    pub declared_objective_count: Option<i32>,
}

/// Supplied item row for the original quest-giver progress panel.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestProgressItem {
    pub name: String,
    pub texture: String,
    pub required_amount: i32,
    /// INFERRED from QuestFrame.lua:252; zero from IsQuestItemHidden shows the row.
    pub hidden: bool,
}

/// Supplied currency row for the original quest-giver progress panel.
#[derive(Debug, Clone, PartialEq)]
pub struct QuestProgressCurrency {
    pub currency_id: i32,
    pub name: String,
    pub texture: i32,
    pub quality: i32,
    pub required_amount: i32,
}

/// Inputs read by Blizzard's original quest-giver progress and turn-in panels.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QuestProgressRequirements {
    pub money: i32,
    pub hide_required_items: bool,
    pub items: Vec<QuestProgressItem>,
    pub currencies: Vec<QuestProgressCurrency>,
    pub text: String,
    pub portrait_display_id: i32,
    pub portrait_name: String,
    pub portrait_text: String,
}

pub fn valid_progress_requirements(requirements: &QuestProgressRequirements) -> bool {
    let valid_dialog = requirements.money >= 0 && requirements.portrait_display_id >= 0;
    let valid_items = requirements
        .items
        .iter()
        .all(|item| item.required_amount > 0);
    let valid_currencies = requirements.currencies.iter().all(|currency| {
        let valid_identity = currency.currency_id > 0 && currency.texture > 0;
        let valid_details = currency.quality >= 0 && currency.required_amount > 0;
        valid_identity && valid_details
    });
    valid_dialog && valid_items && valid_currencies
}

pub fn commit_progress_requirements(
    log: &mut QuestLogState,
    quest_id: i32,
    requirements: QuestProgressRequirements,
) -> bool {
    let active = quest_id > 0 && log.entries.iter().any(|entry| entry.quest_id == quest_id);
    if !active || !valid_progress_requirements(&requirements) {
        return false;
    }
    if log.progress_requirements.get(&quest_id) == Some(&requirements) {
        return false;
    }
    log.progress_requirements.insert(quest_id, requirements);
    true
}

pub fn requirements_for_active_dialog(
    sim: &crate::lua_api::SimState,
) -> Option<&QuestProgressRequirements> {
    let (quest_id, _) = sim.quest_log_entries.readiness.dialog?;
    let selected = sim
        .selected_quest_log_id
        .and_then(|id| i32::try_from(id).ok());
    if selected != Some(quest_id)
        || !crate::c_api::quest_readiness::has_active_npc_quest(sim, quest_id)
    {
        return None;
    }
    sim.quest_log_entries.progress_requirements.get(&quest_id)
}

pub fn progress_requirements_for_active_dialog(
    sim: &crate::lua_api::SimState,
) -> Option<&QuestProgressRequirements> {
    let (_, phase) = sim.quest_log_entries.readiness.dialog?;
    if phase != crate::c_api::quest_readiness::QuestDialogPhase::Progress {
        return None;
    }
    requirements_for_active_dialog(sim)
}

pub fn objectives_for_quest(log: &QuestLogState, quest_id: i32) -> &[QuestObjective] {
    log.objective_progress
        .get(&quest_id)
        .map(|progress| progress.objectives.as_slice())
        .unwrap_or_default()
}

pub fn objective_count(log: &QuestLogState, quest_id: i32) -> i32 {
    log.objective_progress
        .get(&quest_id)
        .map(|progress| {
            progress
                .declared_objective_count
                .unwrap_or(progress.objectives.len() as i32)
        })
        .unwrap_or(0)
}

pub fn valid_progress(progress: &QuestObjectiveProgress) -> bool {
    let valid_percentage = progress
        .percentage
        .is_none_or(|value| value.is_finite() && (0.0..=100.0).contains(&value));
    let valid_objectives = progress.objectives.iter().all(|objective| {
        let valid_counts = objective.num_fulfilled >= 0 && objective.num_required >= 0;
        let valid_criteria = objective.criteria_id.is_none_or(|id| id > 0);
        valid_counts && valid_criteria
    });
    valid_percentage && valid_objectives
}

/// Authored mutation only. Caller commits other projections before dispatching.
pub fn commit_objective_progress(
    log: &mut QuestLogState,
    quest_id: i32,
    mut progress: QuestObjectiveProgress,
) -> bool {
    let active = quest_id > 0 && log.entries.iter().any(|entry| entry.quest_id == quest_id);
    if !active || !valid_progress(&progress) {
        return false;
    }
    progress.declared_objective_count = None;
    if log.objective_progress.get(&quest_id) == Some(&progress) {
        return false;
    }
    log.objective_progress.insert(quest_id, progress);
    true
}

pub(crate) fn changed_criteria(
    previous: Option<&QuestObjectiveProgress>,
    progress: &QuestObjectiveProgress,
) -> Vec<QuestObjective> {
    progress
        .objectives
        .iter()
        .filter(|objective| {
            let Some(id) = objective.criteria_id else {
                return false;
            };
            let old = previous.and_then(|snapshot| {
                snapshot
                    .objectives
                    .iter()
                    .find(|row| row.criteria_id == Some(id))
            });
            old != Some(*objective)
        })
        .cloned()
        .collect()
}

pub(crate) fn seeded_progress() -> HashMap<i32, QuestObjectiveProgress> {
    use crate::lua_api::globals::quest_surface::data::{QUEST_LOG, WORLD_QUESTS};
    let ordinary = QUEST_LOG.iter().filter_map(seed_ordinary_progress);
    let world = WORLD_QUESTS.iter().map(|quest| {
        (
            quest.quest_id,
            QuestObjectiveProgress {
                declared_objective_count: Some(quest.num_objectives),
                ..Default::default()
            },
        )
    });
    ordinary.chain(world).collect()
}

fn seed_ordinary_progress(
    entry: &crate::lua_api::globals::quest_surface::data::QuestLogEntry,
) -> Option<(i32, QuestObjectiveProgress)> {
    use crate::lua_api::globals::quest_surface::data::QuestLogEntry;
    let QuestLogEntry::Quest {
        quest_id,
        objectives,
        ..
    } = entry
    else {
        return None;
    };
    let rows = objectives.iter().map(seed_objective).collect();
    Some((
        *quest_id,
        QuestObjectiveProgress {
            objectives: rows,
            ..Default::default()
        },
    ))
}

fn seed_objective(
    objective: &crate::lua_api::globals::quest_surface::data::Objective,
) -> QuestObjective {
    QuestObjective {
        text: objective.text.into(),
        objective_type: objective.obj_type.into(),
        finished: objective.finished,
        num_fulfilled: objective.num_fulfilled,
        num_required: objective.num_required,
        criteria_id: None,
    }
}
