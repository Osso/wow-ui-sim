//! INFERRED host representation for R88o.1 tree and cache observations.
//! Existing display metadata and visible criteria catalogs do not contain
//! these runtime structures or the associated world-state cache.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct AchievementEligibilityInputs {
    /// Configured tree IDs for rows in the existing achievement catalog.
    pub achievement_tree_ids: HashMap<i32, i32>,
    /// Absent key: no runtime group. Present None: group with a null tree.
    pub groups: HashMap<i32, Option<AchievementEligibilityNode>>,
    /// Simulated CriteriaTree rows, projected to the verified operation classes.
    pub criteria_tree: HashMap<i32, AchievementEligibilityOperation>,
    /// Supplied Criteria records, keyed separately from runtime tree nodes.
    pub criteria: HashMap<i32, AchievementEligibilityCriterion>,
    /// Only the cached value is read; absence is distinct from cached zero.
    /// There is no supported shared-cache update route in this simulator.
    pub world_states: HashMap<i32, i32>,
}

#[derive(Debug, Default)]
pub struct AchievementEligibilityNode {
    pub id: i32,
    pub children: Vec<Self>,
}

#[derive(Clone, Copy, Debug)]
pub enum AchievementEligibilityOperation {
    Leaf { criterion_id: i32 },
    All,
    DefaultTrue,
}

/// Field names are inferred from the older 70391 Criteria export, not a
/// claim of matching 70688 schema or of any installed value.
#[derive(Clone, Copy, Debug)]
pub struct AchievementEligibilityCriterion {
    pub world_state_id: i32,
    pub required_value: i32,
}
