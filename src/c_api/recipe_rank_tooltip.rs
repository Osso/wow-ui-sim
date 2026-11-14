//! Host-supplied records for Retail recipe-rank tooltip composition.
//!
//! The simulator has no writer for the native mutable recipe-display cache.
//! Hosts may supply records; the cache starts empty.

use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct RecipeRankTooltipCache {
    entries: HashMap<i32, RecipeRankTooltipRecord>,
}

#[derive(Clone, Debug)]
pub struct RecipeRankTooltipRecord {
    pub parent_recipe_id: i32,
    pub next_recipe_id: i32,
    pub spell_id: u32,
    /// Final label text from the host's recipe-label lookup, when available.
    pub label: Option<String>,
    /// Final source text from the host's player/faction lookup, when available.
    pub next_rank_source: Option<String>,
}

impl RecipeRankTooltipCache {
    /// Insert one display record. Native duplicate keys keep the existing entry.
    pub fn insert(&mut self, recipe_id: i32, record: RecipeRankTooltipRecord) {
        self.entries.entry(recipe_id).or_insert(record);
    }

    pub(crate) fn get(&self, recipe_id: i32) -> Option<&RecipeRankTooltipRecord> {
        self.entries.get(&recipe_id)
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}
