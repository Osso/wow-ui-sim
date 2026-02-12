//! Talent tree interactive state (ranks purchased, selections, currency mappings).

use std::collections::HashMap;

/// Talent tree interactive state.
pub struct TalentState {
    /// Per-node purchased ranks: node_id → ranks_purchased (default 0).
    pub node_ranks: HashMap<u32, u32>,
    /// Per-node selected entry (for choice nodes): node_id → entry_id.
    pub node_selections: HashMap<u32, u32>,
    /// Group → currency mapping (built at init from cond_type=0 conditions).
    pub group_currency_map: HashMap<u32, u32>,
    /// Node → currency mapping (built at init from group membership).
    pub node_currency_map: HashMap<u32, u32>,
}

impl TalentState {
    /// Build talent state with currency mappings derived from the trait databases.
    pub fn new() -> Self {
        use crate::traits::{TRAIT_COND_DB, TRAIT_NODE_DB};

        // Build group → currency map from gate conditions (cond_type == 0).
        let mut group_currency_map = HashMap::new();
        for (_, cond) in TRAIT_COND_DB.entries() {
            if cond.cond_type == 0 && cond.group_id != 0 && cond.currency_id != 0 {
                group_currency_map.insert(cond.group_id, cond.currency_id);
            }
        }

        // Build node → currency map from each node's group membership.
        let mut node_currency_map = HashMap::new();
        for (&node_id, node) in TRAIT_NODE_DB.entries() {
            for &gid in node.group_ids {
                if let Some(&cid) = group_currency_map.get(&gid) {
                    node_currency_map.insert(node_id, cid);
                    break;
                }
            }
        }

        Self {
            node_ranks: HashMap::new(),
            node_selections: HashMap::new(),
            group_currency_map,
            node_currency_map,
        }
    }

    /// Total points spent for a given currency across all nodes.
    pub fn spent_for_currency(&self, currency_id: u32) -> u32 {
        self.node_ranks.iter()
            .filter(|&(&nid, _)| self.node_currency_map.get(&nid) == Some(&currency_id))
            .map(|(_, &ranks)| ranks)
            .sum()
    }
}
