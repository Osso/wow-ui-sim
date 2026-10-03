//! Atomic explicit-host state transitions; rejection policies are inferred, not native codes.

use crate::c_api::c_housing::catalog::HousingCatalogEntryVariantID;
use crate::c_api::c_housing::exterior::{EXTERIOR_CUSTOMIZATION_MODE, HouseExteriorState};
use crate::lua_api::state::HousingState;

const SUCCESS: i32 = 0;
const ALREADY_SIZE: i32 = 42;
const ALREADY_TYPE: i32 = 43;
use rilua::{LuaResult, runtime_error};

#[path = "storage.rs"]
mod storage;

#[derive(Clone, Copy)]
pub(super) enum ExteriorChange {
    Fixture,
    Size,
    Type,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum AttachedDecorAction {
    Store,
    Detach,
}

impl ExteriorChange {
    pub(super) fn api(self) -> &'static str {
        match self {
            Self::Fixture => "C_HouseExterior.SelectFixtureOption",
            Self::Size => "C_HouseExterior.SetHouseExteriorSize",
            Self::Type => "C_HouseExterior.SetHouseExteriorType",
        }
    }

    pub(super) fn event(self) -> &'static str {
        match self {
            Self::Fixture => "HOUSING_SET_FIXTURE_RESPONSE",
            Self::Size => "HOUSING_SET_EXTERIOR_HOUSE_SIZE_RESPONSE",
            Self::Type => "HOUSING_SET_EXTERIOR_HOUSE_TYPE_RESPONSE",
        }
    }

    fn unchanged_response(self) -> i32 {
        match self {
            Self::Fixture => SUCCESS,
            Self::Size => ALREADY_SIZE,
            Self::Type => ALREADY_TYPE,
        }
    }
}

pub(super) fn update_exterior(
    housing: &mut HousingState,
    change: ExteriorChange,
    target: u32,
    action: AttachedDecorAction,
) -> LuaResult<(i32, Vec<HousingCatalogEntryVariantID>)> {
    validate_host(housing, change)?;
    let unchanged = validate_selection(housing, change, target)?;
    if unchanged {
        return Ok((change.unchanged_response(), Vec::new()));
    }
    let owner = fixture_owner(housing, change);
    let affected = find_affected_placements(housing, owner);
    let stored = if action == AttachedDecorAction::Store {
        storage::store_placements(housing, &affected, change.api())?
    } else {
        detach_placements(housing, &affected);
        Vec::new()
    };
    update_selection(housing, change, target);
    Ok((SUCCESS, stored))
}

fn validate_host(housing: &HousingState, change: ExteriorChange) -> LuaResult<()> {
    if !housing.inside_owned_plot || housing.active_house_editor_mode != EXTERIOR_CUSTOMIZATION_MODE
    {
        return Err(runtime_error(format!(
            "{}: requires an owned plot in ExteriorCustomization mode",
            change.api()
        )));
    }
    Ok(())
}

fn validate_selection(
    housing: &HousingState,
    change: ExteriorChange,
    target: u32,
) -> LuaResult<bool> {
    let exterior = &housing.exterior;
    if !option_is_available(exterior, change, target) {
        return Err(runtime_error(format!(
            "{}: selected host target is missing, locked, or invalid",
            change.api()
        )));
    }
    Ok(current_selection(exterior, change) == Some(target))
}

fn option_is_available(exterior: &HouseExteriorState, change: ExteriorChange, target: u32) -> bool {
    match change {
        ExteriorChange::Fixture => exterior
            .selected_fixture_point
            .as_ref()
            .is_some_and(|point| {
                point
                    .options
                    .iter()
                    .any(|option| option.id == target && !option.is_locked && !option.is_invalid)
            }),
        ExteriorChange::Size => {
            exterior.selected_size.is_some()
                && exterior
                    .size_options
                    .iter()
                    .any(|option| i64::from(option.size) == i64::from(target) && !option.is_locked)
        }
        ExteriorChange::Type => {
            exterior.selected_type_id.is_some()
                && exterior
                    .type_options
                    .iter()
                    .any(|option| option.id == target && !option.is_locked && !option.is_invalid)
        }
    }
}

fn current_selection(exterior: &HouseExteriorState, change: ExteriorChange) -> Option<u32> {
    match change {
        ExteriorChange::Fixture => exterior
            .selected_fixture_point
            .as_ref()
            .and_then(|point| point.selected_fixture_id),
        ExteriorChange::Size => exterior
            .selected_size
            .and_then(|size| u32::try_from(size).ok()),
        ExteriorChange::Type => exterior.selected_type_id,
    }
}

fn fixture_owner(housing: &HousingState, change: ExteriorChange) -> Option<u32> {
    match change {
        ExteriorChange::Fixture => housing
            .exterior
            .selected_fixture_point
            .as_ref()
            .map(|point| point.owner_hash),
        ExteriorChange::Size | ExteriorChange::Type => None,
    }
}

fn find_affected_placements(housing: &HousingState, owner: Option<u32>) -> Vec<String> {
    housing
        .exterior
        .decor
        .iter()
        .filter(|(_, placement)| match placement.fixture_point_owner_hash {
            Some(attached) => owner.is_none_or(|selected| attached == selected),
            None => false,
        })
        .map(|(id, _)| id.clone())
        .collect()
}

fn detach_placements(housing: &mut HousingState, affected: &[String]) {
    for id in affected {
        if let Some(placement) = housing.exterior.decor.get_mut(id) {
            placement.fixture_point_owner_hash = None;
        }
    }
}

fn update_selection(housing: &mut HousingState, change: ExteriorChange, target: u32) {
    let exterior = &mut housing.exterior;
    match change {
        ExteriorChange::Fixture => {
            if let Some(point) = exterior.selected_fixture_point.as_mut() {
                point.selected_fixture_id = Some(target);
            }
        }
        // The target matched an explicit i32 size option before mutation.
        ExteriorChange::Size => exterior.selected_size = Some(target as i32),
        ExteriorChange::Type => exterior.selected_type_id = Some(target),
    }
}
