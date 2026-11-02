//! Retail currency display projection. Catalog positions identify headers; Lua indices do not.
use std::collections::{HashMap, HashSet};

use crate::lua_api::globals::currency_data::{self, CurrencyEntry};

pub(crate) type CurrencyListRow = (i32, CurrencyEntry);

pub struct CurrencyListState {
    collapsed_headers: HashSet<i32>,
    watched_overrides: HashMap<i32, bool>,
    filter: i32,
    pub account_currency_request: bool,
    pub account_loading: bool,
    pub transaction_loading: bool,
    pub pending_character_requests: usize,
    account_rows: Vec<CurrencyEntry>,
    account_rows_visible: bool,
}

impl Default for CurrencyListState {
    fn default() -> Self {
        Self {
            collapsed_headers: HashSet::new(),
            watched_overrides: HashMap::new(),
            filter: 1,
            account_currency_request: false,
            account_loading: false,
            transaction_loading: false,
            pending_character_requests: 0,
            account_rows: Vec::new(),
            account_rows_visible: false,
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
        if filter == 2 && self.account_currency_request {
            self.account_loading = true;
        }
        if filter != 2 || self.account_data_ready() {
            self.account_rows_visible = filter == 2;
        }
    }

    pub fn complete_account_request(&mut self, rows: Vec<CurrencyEntry>) {
        self.account_loading = false;
        self.transaction_loading = false;
        self.pending_character_requests = 0;
        self.account_rows = rows;
        if self.filter == 2 {
            self.account_rows_visible = true;
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
        if self.account_rows_visible {
            for (offset, &entry) in self.account_rows.iter().enumerate() {
                rows.push((
                    currency_data::currency_list_size() + offset as i32 + 1,
                    entry,
                ));
            }
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
