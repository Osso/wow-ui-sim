//! Explicit empty-backed catalog entries, variants and filter-free search snapshots.
//! Entry metadata and variant stacks have distinct identifiers and records.

use std::collections::HashMap;

mod input;
mod queries;
mod snapshot;
mod storage;

pub(super) use input::{read_selector, read_variant_id};
pub(super) use queries::register;
pub(crate) use storage::set_variant_stored_count;

/// Base catalog identity; never synthesize a variant identifier for this key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HousingCatalogEntryID {
    pub record_id: i32,
    pub entry_type: i32,
}

/// Full stack identity from HousingCatalogConstantsDocumentation.lua.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HousingCatalogEntryVariantID {
    pub record_id: i32,
    pub entry_type: i32,
    pub variant_identifier: i32,
}

/// Only explicitly supplied base fields needed by this bounded fixture slice.
/// Other entry metadata remains unmodeled; aggregates are explicit, never variant sums.
#[derive(Clone, Debug)]
pub struct HousingCatalogEntryRecord {
    pub item_id: Option<i32>,
    pub name: String,
    pub is_unique_trophy: bool,
    /// Nonnegative explicit total across variants, excluding unredeemed instances.
    /// None marks a simulator data gap, not a native nil/default contract.
    pub total_num_stored: Option<u32>,
    /// Nonnegative explicit total across houses, plots and variants; not pending placement.
    /// None marks a simulator data gap, not a native nil/default contract.
    pub total_num_placed: Option<u32>,
}

/// Stored instances are not a destroyability decision.
#[derive(Clone, Debug)]
pub struct HousingCatalogVariantRecord {
    pub num_stored: i32,
    /// Explicit storage-limit-counting destroyable instances; independent of num_stored.
    pub destroyable_instance_count: i32,
    pub dye_slots: Vec<HousingDecorDyeSlot>,
}

#[derive(Clone, Debug)]
pub struct HousingDecorDyeSlot {
    pub id: i32,
    pub dye_color_category_id: i32,
    pub order_index: i32,
    pub channel: i32,
    pub dye_color_id: Option<i32>,
    pub dye_color_name: Option<String>,
}

/// Category identity is the map key; ownership is an explicit input, not a variant sum.
#[derive(Clone, Debug)]
pub struct HousingCatalogCategoryRecord {
    pub order_index: i32,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub subcategory_ids: Vec<i32>,
    pub any_stored_entries: bool,
}

/// Subcategory identity is the map key; parent identity does not alias a category record.
#[derive(Clone, Debug)]
pub struct HousingCatalogSubcategoryRecord {
    pub order_index: i32,
    pub parent_category_id: i32,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub any_stored_entries: bool,
}

/// Per-environment input; Default supplies no records or fallback data.
#[derive(Clone, Debug, Default)]
pub struct HousingCatalogState {
    pub categories: HashMap<i32, HousingCatalogCategoryRecord>,
    pub subcategories: HashMap<i32, HousingCatalogSubcategoryRecord>,
    pub entries: HashMap<HousingCatalogEntryID, HousingCatalogEntryRecord>,
    pub variants: HashMap<HousingCatalogEntryVariantID, HousingCatalogVariantRecord>,
}
