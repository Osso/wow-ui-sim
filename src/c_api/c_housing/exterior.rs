//! Explicit exterior selections and placements; no native acquisition or mutation behavior.

use std::collections::BTreeMap;

use super::catalog::HousingCatalogEntryVariantID;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HouseExteriorState {
    pub selected_size: Option<i32>,
    pub selected_type_id: Option<u32>,
    pub size_options: Vec<ExteriorSizeOption>,
    pub type_options: Vec<ExteriorTypeOption>,
    pub selected_fixture_point: Option<ExteriorFixturePoint>,
    pub decor: BTreeMap<String, ExteriorDecorPlacement>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorSizeOption {
    pub size: i32,
    pub name: String,
    pub is_locked: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorTypeOption {
    pub id: u32,
    pub name: String,
    pub is_locked: bool,
    pub is_invalid: bool,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorFixturePoint {
    pub owner_hash: u32,
    pub selected_fixture_id: Option<u32>,
    pub options: Vec<ExteriorFixtureOption>,
    pub can_remove: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorFixtureOption {
    pub id: u32,
    pub name: String,
    pub type_id: u32,
    pub type_name: String,
    pub is_locked: bool,
    pub is_invalid: bool,
    pub reason: String,
    pub color_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorDecorPlacement {
    pub variant_id: HousingCatalogEntryVariantID,
    pub position: [f64; 3],
    /// None is an explicit floating exterior placement, not missing placement data.
    pub fixture_point_owner_hash: Option<u32>,
}
