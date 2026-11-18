//! Supplied inputs for modeled `C_ProfSpecs` queries.
//!
//! The simulator has no profession trait configuration, purchase-eligibility,
//! condition or cost model, nor ranks for related SkillLines.

use std::collections::{HashMap, HashSet};

#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PATH_LOCKED: i32 = 0;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PATH_PROGRESSING: i32 = 1;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PATH_COMPLETED: i32 = 2;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PERK_UNEARNED: i32 = 0;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PERK_PENDING: i32 = 1;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PERK_EARNED: i32 = 2;
#[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
pub(crate) const PROF_SPEC_PERK_IS_MAJOR_BONUS: u32 = 2;

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
    /// INFERRED (simulator mapping): path right groups used to enumerate perks.
    pub path_perk_group_ids: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): perk-node IDs to ProfTraitPerkNode flags.
    pub perk_flags_by_node: HashMap<i32, u32>,
    /// INFERRED (simulator mapping): perk-node IDs to required ranks.
    pub perk_unlock_ranks: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): first type-8 entry ID by perk-node ID.
    pub perk_entry_ids: HashMap<i32, i32>,
    /// INFERRED (simulator mapping): matching rank entries by perk-node and config.
    pub perk_rank_entries_by_config: HashMap<(i32, i32), Vec<ProfessionPerkRankEntry>>,
    /// INFERRED (simulator mapping): stored rank by perk-node and config.
    pub stored_perk_ranks_by_config: HashMap<(i32, i32), i32>,
    /// SkillLines whose configuration has an eligible unlock-kind-9/spend-kind-7 node.
    pub points_eligible_skill_lines: HashSet<u32>,
}

impl ProfessionSpecInputs {
    /// INFERRED (simulator mapping): return perk nodes in the supplied path group order.
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    pub(crate) fn perks_for_path(&self, path_id: i32) -> Vec<(i32, bool)> {
        self.path_perk_group_ids
            .get(&path_id)
            .and_then(|group_id| self.group_node_ids.get(group_id))
            .into_iter()
            .flatten()
            .filter_map(|node_id| {
                self.perk_flags_by_node
                    .get(node_id)
                    .map(|flags| (*node_id, flags & PROF_SPEC_PERK_IS_MAJOR_BONUS != 0))
            })
            .collect()
    }

    /// INFERRED (simulator mapping): classify supplied perk ranks for one config.
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    pub(crate) fn state_for_perk(&self, perk_id: i32, config_id: i32) -> i32 {
        if !self.trait_config_ids.contains(&config_id) {
            return PROF_SPEC_PERK_UNEARNED;
        }
        let ranks = self
            .perk_rank_entries_by_config
            .get(&(perk_id, config_id))
            .into_iter()
            .flatten()
            .map(|entry| {
                (i64::from(entry.learned) + i64::from(entry.granted))
                    .min(i64::from(entry.max_ranks))
            })
            .sum::<i64>();
        if ranks <= 0 {
            return PROF_SPEC_PERK_UNEARNED;
        }
        if self
            .stored_perk_ranks_by_config
            .get(&(perk_id, config_id))
            .copied()
            .unwrap_or_default()
            > 0
        {
            PROF_SPEC_PERK_PENDING
        } else {
            PROF_SPEC_PERK_EARNED
        }
    }

    /// INFERRED (simulator mapping): classify supplied path ranks for one config.
    #[cfg(all(feature = "client-retail", feature = "retail-12-1-5"))]
    pub(crate) fn state_for_path(&self, path_id: i32, config_id: i32) -> i32 {
        if !self.trait_config_ids.contains(&config_id) {
            return PROF_SPEC_PATH_LOCKED;
        }
        let Some(required) = self.path_required_ranks.get(&path_id) else {
            return PROF_SPEC_PATH_LOCKED;
        };
        let Some(ranks) = self.path_ranks_by_config.get(&(path_id, config_id)) else {
            return PROF_SPEC_PATH_LOCKED;
        };
        if ranks.current < *required {
            PROF_SPEC_PATH_LOCKED
        } else if ranks.current == ranks.effective_max {
            PROF_SPEC_PATH_COMPLETED
        } else {
            PROF_SPEC_PATH_PROGRESSING
        }
    }

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

/// INFERRED (simulator mapping): one runtime perk rank entry and its DB2 cap.
#[derive(Debug, Clone, Copy)]
pub struct ProfessionPerkRankEntry {
    pub learned: i32,
    pub granted: i32,
    pub max_ranks: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct SkillRank {
    pub current: i32,
    pub bonus: i32,
}
