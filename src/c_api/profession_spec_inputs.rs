//! Supplied inputs for the `C_ProfSpecs` reminder queries (WM49 part A).
//!
//! The simulator has no profession trait configuration, purchase-eligibility,
//! condition or cost model, nor ranks for related SkillLines.

use std::collections::{HashMap, HashSet};

/// Supplied profession specialization results. Empty defaults make both
/// reminders report nothing.
#[derive(Debug, Default, Clone)]
pub struct ProfessionSpecInputs {
    /// Ranks of related SkillLines; base professions read `profession_data`.
    pub related_skill_ranks: HashMap<u32, SkillRank>,
    /// SkillLines whose cached specialization-enabled flag is set.
    pub enabled_skill_lines: HashSet<u32>,
    /// `Enum.ProfessionsSpecTabState` per tab (TraitTree ID).
    pub tab_states: HashMap<u32, i32>,
    /// SkillLines whose configuration has an eligible unlock-kind-9/spend-kind-7 node.
    pub points_eligible_skill_lines: HashSet<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct SkillRank {
    pub current: i32,
    pub bonus: i32,
}
