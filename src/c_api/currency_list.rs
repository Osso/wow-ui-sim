//! Retail currency display projection. Catalog positions identify headers; Lua indices do not.
use std::collections::{HashMap, HashSet};

use crate::lua_api::globals::currency_data::{self, CurrencyEntry};

pub(crate) type CurrencyListRow = (i32, CurrencyEntry);

#[derive(Default)]
pub struct CurrencyTransferRules {
    pub faction_key_0: Option<bool>,
    pub faction_key_1: Option<bool>,
    pub award_condition_met: Option<bool>,
    pub quantity_cap_reached: Option<bool>,
    pub weekly_cap_reached: Option<bool>,
}

pub struct CurrencyListState {
    collapsed_headers: HashSet<i32>,
    watched_overrides: HashMap<i32, bool>,
    filter: i32,
    pub server_enabled: bool,
    pub account_context_present: bool,
    pub account_cache_invalidated: bool,
    pub cached_transaction_count: usize,
    pub transfer_rules: HashMap<i32, CurrencyTransferRules>,
    pub current_character_guid: Option<String>,
    pub account_currency_sources: HashMap<i32, Vec<(String, u64)>>,
    pub account_loading: bool,
    pub transaction_loading: bool,
    pub pending_character_requests: usize,
    account_rows: Vec<CurrencyEntry>,
    projected_account_rows: Vec<CurrencyEntry>,
}

impl Default for CurrencyListState {
    fn default() -> Self {
        Self {
            collapsed_headers: HashSet::new(),
            watched_overrides: HashMap::new(),
            filter: 1,
            server_enabled: false,
            account_context_present: true,
            account_cache_invalidated: false,
            cached_transaction_count: 0,
            transfer_rules: HashMap::new(),
            current_character_guid: None,
            account_currency_sources: HashMap::new(),
            account_loading: false,
            transaction_loading: false,
            pending_character_requests: 0,
            account_rows: Vec::new(),
            projected_account_rows: Vec::new(),
        }
    }
}

impl CurrencyListState {
    pub fn filter(&self) -> i32 {
        self.filter
    }

    pub fn account_data_ready(&self) -> bool {
        !self.account_loading && !self.transaction_loading && self.pending_character_requests == 0
    }

    pub(crate) fn set_filter(&mut self, filter: i32) {
        self.filter = filter;
        if filter == 2 {
            self.request_account_data();
        }
        if filter != 2 {
            self.projected_account_rows.clear();
        } else if self.account_data_ready() {
            self.projected_account_rows.clone_from(&self.account_rows);
        }
    }

    pub(crate) fn request_account_data(&mut self) {
        if !self.server_enabled || !self.account_context_present || self.account_loading {
            return;
        }
        if self.account_cache_invalidated {
            self.account_rows.clear();
            self.account_currency_sources.clear();
            self.cached_transaction_count = 0;
            self.account_cache_invalidated = false;
        }
        if self.account_rows.is_empty() && self.account_currency_sources.is_empty() {
            self.account_loading = true;
        }
    }

    pub fn complete_account_request(&mut self, rows: Vec<CurrencyEntry>) {
        self.account_loading = false;
        self.account_rows = rows;
        self.account_currency_sources.clear();
        if self.filter == 2 && self.account_data_ready() {
            self.projected_account_rows.clone_from(&self.account_rows);
        }
    }

    pub(crate) fn is_watched(&self, entry: &CurrencyEntry) -> bool {
        self.watched_overrides
            .get(&entry.currency_id)
            .copied()
            .unwrap_or(entry.is_show_in_backpack)
    }

    pub(crate) fn set_watched(&mut self, currency_id: i32, watched: bool) {
        self.watched_overrides.insert(currency_id, watched);
    }

    pub(crate) fn displayed_entries(&self) -> Vec<CurrencyListRow> {
        let mut hidden_below_depth = None;
        self.ordered_entries()
            .into_iter()
            .filter(|(key, entry)| {
                if hidden_below_depth.is_some_and(|depth| entry.depth > depth) {
                    return false;
                }
                hidden_below_depth = if entry.is_header && self.collapsed_headers.contains(key) {
                    Some(entry.depth)
                } else {
                    None
                };
                true
            })
            .collect()
    }

    fn ordered_entries(&self) -> Vec<CurrencyListRow> {
        let mut rows: Vec<_> = (1..=currency_data::currency_list_size())
            .filter_map(|index| {
                currency_data::get_currency_list_entry(index).map(|entry| (index, *entry))
            })
            .collect();
        for (offset, &entry) in self.projected_account_rows.iter().enumerate() {
            rows.push((
                currency_data::currency_list_size() + offset as i32 + 1,
                entry,
            ));
        }
        rows
    }

    pub(crate) fn displayed_entry(&self, index: i32) -> Option<CurrencyListRow> {
        let offset = usize::try_from(index.checked_sub(1)?).ok()?;
        self.displayed_entries().get(offset).copied()
    }

    pub(crate) fn header_expanded(&self, index: i32) -> bool {
        self.displayed_entry(index)
            .is_some_and(|(key, entry)| entry.is_header && !self.collapsed_headers.contains(&key))
    }

    pub(crate) fn set_header_expanded(&mut self, index: i32, expanded: bool) {
        let Some((key, entry)) = self.displayed_entry(index) else {
            return;
        };
        if !entry.is_header {
            return;
        }
        if expanded {
            self.collapsed_headers.remove(&key);
        } else {
            self.collapsed_headers.insert(key);
        }
    }
}
