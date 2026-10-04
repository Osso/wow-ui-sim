//! Direct core attachments share the existing atomic Store/Detach transaction.

use super::mutation::{self, AttachedDecorAction};
use crate::c_api::c_housing::catalog::HousingCatalogEntryVariantID;
use crate::c_api::c_housing::exterior::ExteriorCoreFixtureOption;
use crate::lua_api::state::HousingState;
use rilua::{LuaResult, runtime_error};

pub(super) const API: &str = "C_HouseExterior.SelectCoreFixtureOption";
const SUCCESS: i32 = 0;

pub(super) fn update_core_fixture(
    housing: &mut HousingState,
    target: u32,
    action: AttachedDecorAction,
) -> LuaResult<(i32, Vec<HousingCatalogEntryVariantID>)> {
    mutation::validate_host(housing, API)?;
    let (old, new) = read_validated_options(housing, target)?;
    // INFERRED: reselecting the current fixture succeeds without attachment work.
    if old.fixture_id == new.fixture_id {
        return Ok((SUCCESS, Vec::new()));
    }
    let affected = mutation::find_affected_placements(housing, Some(old.owner_hash));
    let stored = if is_recolor(&old, &new) {
        reparent_placements(housing, &affected, new.owner_hash);
        Vec::new()
    } else {
        mutation::update_attachments(housing, &affected, action, API)?
    };
    housing
        .exterior
        .core_fixture
        .as_mut()
        .expect("validated core fixture")
        .selected_fixture_id = target;
    Ok((SUCCESS, stored))
}

fn read_validated_options(
    housing: &HousingState,
    target: u32,
) -> LuaResult<(ExteriorCoreFixtureOption, ExteriorCoreFixtureOption)> {
    let core = housing
        .exterior
        .core_fixture
        .as_ref()
        .ok_or_else(|| runtime_error(format!("{API}: requires explicit core fixture state")))?;
    let old = core
        .options
        .iter()
        .find(|option| option.fixture_id == core.selected_fixture_id)
        .ok_or_else(|| runtime_error(format!("{API}: current fixture has no host option")))?;
    let new = core
        .options
        .iter()
        .find(|option| option.fixture_id == target)
        .filter(|option| !option.is_locked && !option.is_invalid)
        .ok_or_else(|| runtime_error(format!("{API}: target is missing, locked, or invalid")))?;
    Ok((old.clone(), new.clone()))
}

fn is_recolor(old: &ExteriorCoreFixtureOption, new: &ExteriorCoreFixtureOption) -> bool {
    // No ID arithmetic or fabricated equivalence when the host supplied no group.
    old.recolor_group.is_some() && old.recolor_group == new.recolor_group
}

fn reparent_placements(housing: &mut HousingState, affected: &[String], owner_hash: u32) {
    for id in affected {
        housing
            .exterior
            .decor
            .get_mut(id)
            .expect("snapshot contains existing placement")
            .fixture_point_owner_hash = Some(owner_hash);
    }
}
