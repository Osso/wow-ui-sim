//! Explicit exterior selections and placements; native acquisition remains unmodeled.

#[path = "exterior/runtime.rs"]
mod runtime;

pub(super) const EXTERIOR_CUSTOMIZATION_MODE: i32 = 6;

pub(crate) const MODELED: bool = cfg!(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
));

pub(super) fn register(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<()> {
    if MODELED {
        runtime::register(state)
    } else {
        Ok(())
    }
}

use std::collections::BTreeMap;

use super::catalog::HousingCatalogEntryVariantID;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HouseExteriorState {
    /// Host-selected core family; empty unless explicitly supplied.
    pub core_fixture: Option<ExteriorCoreFixture>,
    pub selected_size: Option<i32>,
    pub selected_type_id: Option<u32>,
    pub size_options: Vec<ExteriorSizeOption>,
    pub type_options: Vec<ExteriorTypeOption>,
    pub selected_fixture_point: Option<ExteriorFixturePoint>,
    pub decor: BTreeMap<String, ExteriorDecorPlacement>,
}

/// One explicit core selection and its eligible replacements; no native population.
#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorCoreFixture {
    pub selected_fixture_id: u32,
    pub options: Vec<ExteriorCoreFixtureOption>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExteriorCoreFixtureOption {
    pub fixture_id: u32,
    /// Attachment parent identity in ExteriorDecorPlacement, supplied by the host.
    pub owner_hash: u32,
    /// Shared group means variants of the same style; None never implies equivalence.
    pub recolor_group: Option<u32>,
    pub is_locked: bool,
    pub is_invalid: bool,
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
