//! Validate all existing catalog counts before committing any placement or inventory change.

use crate::c_api::c_housing::catalog::{HousingCatalogEntryID, HousingCatalogEntryVariantID};
use crate::lua_api::state::HousingState;
use rilua::{LuaResult, runtime_error};
use std::collections::HashMap;

type VariantCounts = HashMap<HousingCatalogEntryVariantID, u32>;
type BaseCounts = HashMap<HousingCatalogEntryID, u32>;

struct StorePlan {
    variants: Vec<(HousingCatalogEntryVariantID, i32)>,
    totals: Vec<(HousingCatalogEntryID, Option<u32>, Option<u32>)>,
}

pub(super) fn store_placements(
    housing: &mut HousingState,
    affected: &[String],
    api: &str,
) -> LuaResult<Vec<HousingCatalogEntryVariantID>> {
    let (variants, bases) = count_placements(housing, affected, api)?;
    let plan = validate_counts(housing, variants, bases, api)?;
    let published = plan.variants.iter().map(|(id, _)| *id).collect();
    commit_storage(housing, affected, plan);
    Ok(published)
}

fn count_placements(
    housing: &HousingState,
    affected: &[String],
    api: &str,
) -> LuaResult<(VariantCounts, BaseCounts)> {
    let mut variants = VariantCounts::new();
    let mut bases = BaseCounts::new();
    for id in affected {
        let variant = housing.exterior.decor[id].variant_id;
        let base = HousingCatalogEntryID {
            record_id: variant.record_id,
            entry_type: variant.entry_type,
        };
        increment_count(variants.entry(variant).or_default(), api)?;
        increment_count(bases.entry(base).or_default(), api)?;
    }
    Ok((variants, bases))
}

fn increment_count(count: &mut u32, api: &str) -> LuaResult<()> {
    *count = count.checked_add(1).ok_or_else(|| inventory_error(api))?;
    Ok(())
}

fn validate_counts(
    housing: &HousingState,
    variants: VariantCounts,
    bases: BaseCounts,
    api: &str,
) -> LuaResult<StorePlan> {
    let variants = variants
        .into_iter()
        .map(|(id, count)| {
            let record = housing
                .catalog
                .variants
                .get(&id)
                .ok_or_else(|| inventory_error(api))?;
            let delta = i32::try_from(count).map_err(|_| inventory_error(api))?;
            let stored = record
                .num_stored
                .checked_add(delta)
                .filter(|_| record.num_stored >= 0)
                .ok_or_else(|| inventory_error(api))?;
            Ok((id, stored))
        })
        .collect::<LuaResult<Vec<_>>>()?;
    let totals = bases
        .into_iter()
        .filter(|(id, _)| housing.catalog.entries.contains_key(id))
        .map(|(id, count)| {
            let base = &housing.catalog.entries[&id];
            let stored = adjust_known_total(base.total_num_stored, count, true, api)?;
            let placed = adjust_known_total(base.total_num_placed, count, false, api)?;
            Ok((id, stored, placed))
        })
        .collect::<LuaResult<Vec<_>>>()?;
    Ok(StorePlan { variants, totals })
}

fn adjust_known_total(
    total: Option<u32>,
    delta: u32,
    add: bool,
    api: &str,
) -> LuaResult<Option<u32>> {
    total
        .map(|value| {
            let changed = if add {
                value.checked_add(delta)
            } else {
                value.checked_sub(delta)
            };
            changed.ok_or_else(|| inventory_error(api))
        })
        .transpose()
}

fn commit_storage(housing: &mut HousingState, affected: &[String], plan: StorePlan) {
    for (id, stored) in plan.variants {
        housing
            .catalog
            .variants
            .get_mut(&id)
            .expect("validated existing variant")
            .num_stored = stored;
    }
    for (id, stored, placed) in plan.totals {
        let base = housing
            .catalog
            .entries
            .get_mut(&id)
            .expect("validated existing entry");
        base.total_num_stored = stored;
        base.total_num_placed = placed;
    }
    for id in affected {
        housing.exterior.decor.remove(id);
    }
}

fn inventory_error(api: &str) -> rilua::LuaError {
    runtime_error(format!(
        "{api}: Store requires valid existing inventory counts"
    ))
}
