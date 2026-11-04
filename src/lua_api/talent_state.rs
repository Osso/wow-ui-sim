//! Talent tree interactive state (ranks purchased, selections, currency mappings).

use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy)]
pub struct SeededTalentConfig {
    pub id: i32,
}

#[derive(Clone, Default)]
pub struct TalentLoadoutState {
    pub node_ranks: HashMap<u32, u32>,
    pub node_selections: HashMap<u32, u32>,
}

const HOLY_CONFIGS: [SeededTalentConfig; 2] = [
    SeededTalentConfig { id: 101 },
    SeededTalentConfig { id: 102 },
];

const PROTECTION_CONFIGS: [SeededTalentConfig; 2] = [
    SeededTalentConfig { id: 201 },
    SeededTalentConfig { id: 202 },
];

const RETRIBUTION_CONFIGS: [SeededTalentConfig; 2] = [
    SeededTalentConfig { id: 301 },
    SeededTalentConfig { id: 302 },
];

pub fn seeded_class_talent_configs(spec_id: u32) -> &'static [SeededTalentConfig] {
    match spec_id {
        65 => &HOLY_CONFIGS,
        66 => &PROTECTION_CONFIGS,
        70 => &RETRIBUTION_CONFIGS,
        _ => &[],
    }
}

pub fn default_class_talent_config_id(spec_id: u32) -> Option<i32> {
    seeded_class_talent_configs(spec_id)
        .first()
        .map(|config| config.id)
}

fn build_group_currency_map() -> HashMap<u32, u32> {
    use crate::traits::TRAIT_COND_DB;
    let mut map = HashMap::new();
    for (_, cond) in TRAIT_COND_DB.entries() {
        if cond.cond_type == 0 && cond.group_id != 0 && cond.currency_id != 0 {
            map.insert(cond.group_id, cond.currency_id);
        }
    }
    map
}

fn subtree_currency_id(subtree_id: u32) -> Option<u32> {
    match subtree_id {
        48 => Some(2986),
        49 => Some(2987),
        50 => Some(2988),
        _ => None,
    }
}

fn build_node_currency_map(group_currency_map: &HashMap<u32, u32>) -> HashMap<u32, u32> {
    use crate::traits::TRAIT_NODE_DB;
    let mut map = HashMap::new();
    for (&node_id, node) in TRAIT_NODE_DB.entries() {
        if let Some(currency_id) = subtree_currency_id(node.sub_tree_id) {
            map.insert(node_id, currency_id);
            continue;
        }
        for &gid in node.group_ids {
            if let Some(&cid) = group_currency_map.get(&gid) {
                map.insert(node_id, cid);
                break;
            }
        }
    }
    map
}

fn seed_hero_spec_nodes(active_spec_id: u32) -> (HashMap<u32, u32>, HashMap<u32, u32>) {
    let mut node_ranks = HashMap::new();
    let mut node_selections = HashMap::new();
    super::globals::hero_talents::auto_select_hero_spec_for_spec(
        active_spec_id,
        &mut node_ranks,
        &mut node_selections,
    );
    (node_ranks, node_selections)
}

fn tally_currency_spent(
    node_ranks: &HashMap<u32, u32>,
    node_currency_map: &HashMap<u32, u32>,
) -> HashMap<u32, u32> {
    let mut spent = HashMap::new();
    for (&node_id, &ranks) in node_ranks {
        if let Some(&currency_id) = node_currency_map.get(&node_id) {
            *spent.entry(currency_id).or_insert(0) += ranks;
        }
    }
    spent
}

fn detect_active_hero_subtree(node_selections: &HashMap<u32, u32>) -> Option<u32> {
    node_selections.values().find_map(|entry_id| {
        crate::traits::TRAIT_ENTRY_DB
            .get(entry_id)
            .and_then(|entry| (entry.sub_tree_id != 0).then_some(entry.sub_tree_id))
    })
}

fn default_last_selected_config_ids() -> HashMap<u32, i32> {
    [65u32, 66, 70]
        .into_iter()
        .filter_map(|spec_id| {
            default_class_talent_config_id(spec_id).map(|config_id| (spec_id, config_id))
        })
        .collect()
}

fn default_loadout_state_for_spec(active_spec_id: u32) -> TalentLoadoutState {
    let (node_ranks, node_selections) = seed_hero_spec_nodes(active_spec_id);
    TalentLoadoutState {
        node_ranks,
        node_selections,
    }
}

fn default_loadout_states_for_spec(active_spec_id: u32) -> HashMap<i32, TalentLoadoutState> {
    let state = default_loadout_state_for_spec(active_spec_id);
    seeded_class_talent_configs(active_spec_id)
        .iter()
        .map(|config| (config.id, state.clone()))
        .collect()
}

/// Talent tree interactive state.
pub struct TalentState {
    /// EMULATED Forever legacy grid inputs, separate from Retail trait data.
    pub forever: crate::c_api::forever_talents::ForeverTalentState,
    /// Supplied lifetime talent-award history; false means empty host history.
    /// Native acquisition and its relationship to level/current ranks are unknown.
    pub has_earned_talent_point: bool,
    /// Scenario input, not a native specialization eligibility rule.
    pub spec_selection_enabled: bool,
    /// Active spec ID for the current loadout state.
    pub active_spec_id: u32,
    /// Spec currently loaded into the talent viewer surface.
    pub view_spec_id: Option<u32>,
    /// Per-node purchased ranks: node_id → ranks_purchased (default 0).
    pub node_ranks: HashMap<u32, u32>,
    /// Per-node selected entry (for choice nodes): node_id → entry_id.
    pub node_selections: HashMap<u32, u32>,
    /// Group → currency mapping (built at init from cond_type=0 conditions).
    pub group_currency_map: HashMap<u32, u32>,
    /// Node → currency mapping (built at init from group membership).
    pub node_currency_map: HashMap<u32, u32>,
    /// Cached spent totals by currency for fast talent condition checks.
    pub currency_spent: HashMap<u32, u32>,
    /// Cached currently selected hero subtree, if any.
    pub active_hero_subtree_id: Option<u32>,
    /// Currently active seeded config for the active specialization.
    pub active_config_id: i32,
    /// Last selected seeded config per specialization.
    pub last_selected_config_id_by_spec_id: HashMap<u32, i32>,
    /// Whether the player can currently change talents. Drives
    /// `C_ClassTalents.CanChangeTalents`. Seeded true (out of combat,
    /// not in arena preparation). Flip off before firing combat enter
    /// events in tests.
    pub can_change_talents: bool,
    /// Whether the active class has a starter build available. Drives
    /// `C_ClassTalents.GetHasStarterBuild`. Seeded false.
    pub has_starter_build: bool,
    /// Whether the active talent config is the starter build. Drives
    /// `C_ClassTalents.IsStarterBuildActive`. Seeded false.
    pub is_starter_build_active: bool,
    /// Working loadout state per config ID.
    pub config_states: HashMap<i32, TalentLoadoutState>,
    /// Committed loadout state per config ID.
    pub committed_config_states: HashMap<i32, TalentLoadoutState>,
}

impl TalentState {
    /// Build talent state with currency mappings derived from the trait databases.
    pub fn new() -> Self {
        Self::for_spec_id(66)
    }

    /// Build talent state seeded for a specific specialization.
    pub fn for_spec_id(active_spec_id: u32) -> Self {
        let group_currency_map = build_group_currency_map();
        let node_currency_map = build_node_currency_map(&group_currency_map);
        let (node_ranks, node_selections) = seed_hero_spec_nodes(active_spec_id);
        let currency_spent = tally_currency_spent(&node_ranks, &node_currency_map);
        let active_hero_subtree_id = detect_active_hero_subtree(&node_selections);
        let last_selected_config_id_by_spec_id = default_last_selected_config_ids();
        let active_config_id = last_selected_config_id_by_spec_id
            .get(&active_spec_id)
            .copied()
            .unwrap_or(1);
        let config_states = default_loadout_states_for_spec(active_spec_id);

        Self {
            forever: crate::c_api::forever_talents::ForeverTalentState::default(),
            has_earned_talent_point: false,
            spec_selection_enabled: true,
            active_spec_id,
            view_spec_id: None,
            node_ranks,
            node_selections,
            group_currency_map,
            node_currency_map,
            currency_spent,
            active_hero_subtree_id,
            active_config_id,
            last_selected_config_id_by_spec_id,
            can_change_talents: true,
            has_starter_build: false,
            is_starter_build_active: false,
            config_states: config_states.clone(),
            committed_config_states: config_states,
        }
    }

    fn active_loadout_state(&self) -> TalentLoadoutState {
        TalentLoadoutState {
            node_ranks: self.node_ranks.clone(),
            node_selections: self.node_selections.clone(),
        }
    }

    fn apply_loadout_state(&mut self, state: &TalentLoadoutState) {
        self.node_ranks = state.node_ranks.clone();
        self.node_selections = state.node_selections.clone();
        self.currency_spent = tally_currency_spent(&self.node_ranks, &self.node_currency_map);
        self.active_hero_subtree_id = detect_active_hero_subtree(&self.node_selections);
    }

    fn persist_active_loadout_state(&mut self) {
        self.config_states
            .insert(self.active_config_id, self.active_loadout_state());
    }

    fn ensure_config_state(&mut self, config_id: i32) {
        if self.config_states.contains_key(&config_id) {
            return;
        }
        let default_state = default_loadout_state_for_spec(self.active_spec_id);
        self.config_states.insert(config_id, default_state.clone());
        self.committed_config_states
            .insert(config_id, default_state);
    }

    /// Total points spent for a given currency across all nodes.
    pub fn spent_for_currency(&self, currency_id: u32) -> u32 {
        self.currency_spent.get(&currency_id).copied().unwrap_or(0)
    }

    /// Set a node's purchased rank and update cached currency totals.
    pub fn set_node_rank(&mut self, node_id: u32, new_rank: u32) {
        let old_rank = self.node_ranks.get(&node_id).copied().unwrap_or(0);
        if old_rank == new_rank {
            return;
        }

        if new_rank == 0 {
            self.node_ranks.remove(&node_id);
        } else {
            self.node_ranks.insert(node_id, new_rank);
        }

        if let Some(&currency_id) = self.node_currency_map.get(&node_id) {
            let entry = self.currency_spent.entry(currency_id).or_insert(0);
            if new_rank >= old_rank {
                *entry += new_rank - old_rank;
            } else {
                *entry -= old_rank - new_rank;
            }
            if *entry == 0 {
                self.currency_spent.remove(&currency_id);
            }
        }
        self.persist_active_loadout_state();
    }

    /// Clear all purchased ranks and cached spent totals.
    pub fn clear_ranks(&mut self) {
        self.node_ranks.clear();
        self.currency_spent.clear();
        self.persist_active_loadout_state();
    }

    /// Update a node's selected entry and refresh the cached hero subtree when relevant.
    pub fn set_node_selection(&mut self, node_id: u32, entry_id: Option<u32>) {
        let Some(entry_id) = entry_id else {
            self.deselect_node(node_id);
            return;
        };
        self.node_selections.insert(node_id, entry_id);
        if let Some(entry) = crate::traits::TRAIT_ENTRY_DB.get(&entry_id) {
            if entry.sub_tree_id != 0 {
                self.active_hero_subtree_id = Some(entry.sub_tree_id);
            }
        }
        self.persist_active_loadout_state();
    }

    fn deselect_node(&mut self, node_id: u32) {
        let removed_entry_id = self.node_selections.remove(&node_id);
        let removed_sub_tree = removed_entry_id
            .and_then(|eid| crate::traits::TRAIT_ENTRY_DB.get(&eid))
            .map(|entry| entry.sub_tree_id);
        let was_active = removed_sub_tree
            .zip(self.active_hero_subtree_id)
            .is_some_and(|(removed, active)| removed == active);
        if was_active {
            self.active_hero_subtree_id = detect_active_hero_subtree(&self.node_selections);
        }
    }

    /// Return the currently selected hero subtree, if any.
    pub fn active_hero_subtree(&self) -> Option<u32> {
        self.active_hero_subtree_id
    }

    pub fn is_active_config(&self, config_id: i32) -> bool {
        config_id == self.active_config_id
    }

    pub fn has_staged_changes(&self, config_id: i32) -> bool {
        !self.staged_purchases(config_id).is_empty()
            || !self.staged_refunds(config_id).is_empty()
            || !self.staged_selection_swaps(config_id).is_empty()
    }

    pub fn staged_purchases(&self, config_id: i32) -> Vec<u32> {
        self.staged_rank_changes(config_id, |working, committed| working > committed)
    }

    pub fn staged_refunds(&self, config_id: i32) -> Vec<u32> {
        self.staged_rank_changes(config_id, |working, committed| working < committed)
    }

    pub fn staged_selection_swaps(&self, config_id: i32) -> Vec<u32> {
        let Some(working) = self.working_loadout_state(config_id) else {
            return Vec::new();
        };
        let Some(committed) = self.committed_loadout_state(config_id) else {
            return Vec::new();
        };

        selection_change_ids(working, committed)
            .into_iter()
            .filter(|node_id| {
                matches!(
                    (
                        working.node_selections.get(node_id),
                        committed.node_selections.get(node_id),
                    ),
                    (Some(working_entry), Some(committed_entry))
                        if working_entry != committed_entry
                )
            })
            .collect()
    }

    pub fn staged_cost_deltas(&self, config_id: i32) -> Vec<(u32, i32)> {
        let Some(working) = self.working_loadout_state(config_id) else {
            return Vec::new();
        };
        let Some(committed) = self.committed_loadout_state(config_id) else {
            return Vec::new();
        };

        let working_spent = tally_currency_spent(&working.node_ranks, &self.node_currency_map);
        let committed_spent = tally_currency_spent(&committed.node_ranks, &self.node_currency_map);

        currency_change_ids(&working_spent, &committed_spent)
            .into_iter()
            .filter_map(|currency_id| {
                let working_amount = working_spent.get(&currency_id).copied().unwrap_or(0) as i32;
                let committed_amount =
                    committed_spent.get(&currency_id).copied().unwrap_or(0) as i32;
                let delta = working_amount - committed_amount;
                (delta != 0).then_some((currency_id, delta))
            })
            .collect()
    }

    pub fn switch_to_spec(&mut self, spec_id: u32) {
        self.persist_active_loadout_state();
        let last_selected = self.last_selected_config_id_by_spec_id.clone();
        let can_change = self.can_change_talents;
        let has_starter = self.has_starter_build;
        let config_states = self.config_states.clone();
        let committed_config_states = self.committed_config_states.clone();
        *self = Self::for_spec_id(spec_id);
        self.config_states.extend(config_states);
        self.committed_config_states.extend(committed_config_states);
        self.last_selected_config_id_by_spec_id
            .extend(last_selected);
        self.active_spec_id = spec_id;
        self.active_config_id = self
            .last_selected_config_id_by_spec_id
            .get(&spec_id)
            .copied()
            .or_else(|| default_class_talent_config_id(spec_id))
            .unwrap_or(self.active_config_id);
        if let Some(state) = self.config_states.get(&self.active_config_id).cloned() {
            self.apply_loadout_state(&state);
        }
        self.ensure_config_state(self.active_config_id);
        self.can_change_talents = can_change;
        self.has_starter_build = has_starter;
    }

    pub fn initialize_view_loadout(&mut self, spec_id: u32) {
        self.view_spec_id = Some(spec_id);
    }

    pub fn switch_to_loadout(&mut self, spec_id: u32, config_id: i32) {
        self.persist_active_loadout_state();
        self.active_config_id = config_id;
        self.last_selected_config_id_by_spec_id
            .insert(spec_id, config_id);
        self.ensure_config_state(config_id);
        if let Some(state) = self.config_states.get(&config_id).cloned() {
            self.apply_loadout_state(&state);
        }
    }

    pub fn update_active_loadout_state(&mut self) {
        self.persist_active_loadout_state();
    }

    pub fn committed_loadout_state(&self, config_id: i32) -> Option<&TalentLoadoutState> {
        self.committed_config_states.get(&config_id)
    }

    pub fn working_loadout_state(&self, config_id: i32) -> Option<&TalentLoadoutState> {
        self.config_states.get(&config_id)
    }

    fn staged_rank_changes(
        &self,
        config_id: i32,
        include_node: impl Fn(u32, u32) -> bool,
    ) -> Vec<u32> {
        let Some(working) = self.working_loadout_state(config_id) else {
            return Vec::new();
        };
        let Some(committed) = self.committed_loadout_state(config_id) else {
            return Vec::new();
        };

        rank_change_ids(working, committed)
            .into_iter()
            .filter(|node_id| {
                let working_rank = working.node_ranks.get(node_id).copied().unwrap_or(0);
                let committed_rank = committed.node_ranks.get(node_id).copied().unwrap_or(0);
                include_node(working_rank, committed_rank)
            })
            .collect()
    }
}

fn rank_change_ids(working: &TalentLoadoutState, committed: &TalentLoadoutState) -> BTreeSet<u32> {
    working
        .node_ranks
        .keys()
        .chain(committed.node_ranks.keys())
        .copied()
        .collect()
}

fn selection_change_ids(
    working: &TalentLoadoutState,
    committed: &TalentLoadoutState,
) -> BTreeSet<u32> {
    working
        .node_selections
        .keys()
        .chain(committed.node_selections.keys())
        .copied()
        .collect()
}

fn currency_change_ids(
    working: &HashMap<u32, u32>,
    committed: &HashMap<u32, u32>,
) -> BTreeSet<u32> {
    working.keys().chain(committed.keys()).copied().collect()
}
