//! Supplied inputs for modeled `C_ProfSpecs` queries.
//!
//! The simulator has no profession trait configuration, purchase-eligibility,
//! condition or cost model, nor ranks for related SkillLines.

use std::collections::{HashMap, HashSet};

/// Supplied profession specialization results. Empty defaults make both
/// reminders report nothing.
#[derive(Debug, Default, Clone)]
pub struct ProfessionSpecInputs {
    /// Ranks of related SkillLines for the name query's learned gate; base
    /// professions read `profession_data`.
    pub related_skill_ranks: HashMap<u32, SkillRank>,
    /// SkillLines whose cached specialization-enabled flag is set.
    pub enabled_skill_lines: HashSet<u32>,
    /// `Enum.ProfessionsSpecTabState` per tab (TraitTree ID).
    pub tab_states: HashMap<u32, i32>,
    /// INFERRED (simulator mapping): ProfTraitPathNode path IDs to group IDs.
    pub path_group_ids: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): groups to ordered TraitNode IDs.
    pub group_node_ids: HashMap<i32, Vec<i32>>,
    /// INFERRED (simulator mapping): supplied runtime trait configurations.
    pub trait_config_ids: HashSet<i32>,
    /// INFERRED (simulator mapping): path type-9 entries to their required rank.
    pub path_required_ranks: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): current and effective maximum ranks by path and config.
    pub path_ranks_by_config: HashMap<(i32, i32), ProfessionPathRanks>,
    /// INFERRED (simulator mapping): path type-7 spend entries.
    pub path_spend_entry_ids: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): path type-9 unlock entries.
    pub path_unlock_entry_ids: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): first qualifying type-1 spend currency by path.
    pub path_spend_currency_ids: HashMap<i32, i32>,
    /// SkillLines whose configuration has an eligible unlock-kind-9/spend-kind-7 node.
    pub points_eligible_skill_lines: HashSet<u32>,
}

impl ProfessionSpecInputs {
    /// INFERRED (simulator mapping): return supplied children for one path ID.
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    pub(crate) fn children_for_path(&self, path_id: i32) -> Vec<i32> {
        self.path_group_ids
            .get(&path_id)
            .and_then(|group_id| self.group_node_ids.get(group_id))
            .cloned()
            .unwrap_or_default()
    }
}

/// INFERRED (simulator mapping): rank values supplied for one path and config.
#[derive(Debug, Clone, Copy)]
pub struct ProfessionPathRanks {
    pub current: i32,
    pub effective_max: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct SkillRank {
    pub current: i32,
    pub bonus: i32,
}
