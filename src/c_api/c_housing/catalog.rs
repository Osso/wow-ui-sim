//! Explicit empty-backed catalog entries, variants and filter-free search snapshots.
//! Entry metadata and variant stacks have distinct identifiers and records.

use std::collections::HashMap;

mod queries;
mod snapshot;

pub(super) use queries::register;

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
/// Other entry metadata, aggregate counts and their policies remain unmodeled.
#[derive(Clone, Debug)]
pub struct HousingCatalogEntryRecord {
    pub item_id: Option<i32>,
    pub name: String,
    pub is_unique_trophy: bool,
}

/// Stored instances are not a destroyability decision.
#[derive(Clone, Debug)]
pub struct HousingCatalogVariantRecord {
    pub num_stored: i32,
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

/// Per-environment input; Default supplies no records or fallback data.
#[derive(Clone, Debug, Default)]
pub struct HousingCatalogState {
    pub entries: HashMap<HousingCatalogEntryID, HousingCatalogEntryRecord>,
    pub variants: HashMap<HousingCatalogEntryVariantID, HousingCatalogVariantRecord>,
}
