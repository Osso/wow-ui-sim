//! WM94 §2: `GetItemKey` composes the shared catalog item with the item-key
//! displayed-level override and required-level control.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:484` declare the getter.
//!
//! Modelled from WM94:
//! - a nonzero item level adds top-level `overrideItemLevel` and drives the
//!   producer's displayed level, whose working value is bounded to 1..=1300;
//! - a nonzero required level emits the simple requirement line
//!   (`ITEM_MIN_LEVEL`, numeric line type 5, integer `level`), combined with the
//!   item requirement by taking the greater value.
//!
//! UNKNOWN and therefore not published: the
//! `AUCTION_HOUSE_BUCKET_VARIATION_EQUIPMENT_TOOLTIP` eligibility branch, and any
//! item-name suffix record (no suffix table is modelled, so a supplied suffix
//! leaves the base name in place). The item requirement term is EMULATED from
//! the catalog `RequiredLevel`; the native separate requirement calculation is
//! not established.

use crate::c_api::tooltip_getter_args::{optional_unsigned, signed, unsigned};
use crate::items::{self, ItemInfo};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, format_global_numeric, tooltip_for_item_context,
};
use crate::lua_api::methods::table_set;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

/// WM94 §2: the simple requirement line uses numeric line type 5.
const LINE_TYPE_REQUIREMENT: f64 = 5.0;
/// WM94 §2: the working item level calculation is bounded to 1..=1300.
const WORKING_LEVEL_MIN: u32 = 1;
const WORKING_LEVEL_MAX: u32 = 1300;

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetItemKey", get_item_key)
}

fn get_item_key(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = unsigned(state, 1)?;
    let level = unsigned(state, 2)?;
    // The supplied suffix overrides the title formatter's suffix selection, but
    // no suffix record set is modelled (see the module note).
    let _suffix = signed(state, 3)?;
    let required_level = optional_unsigned(state, 4)?.unwrap_or(0);
    let Some(catalog_item) = items::get_item(item_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let item = item_with_working_level(catalog_item, level);
    let tooltip = tooltip_for_item_context(state, item_id, Some(&item));
    if level != 0 {
        table_set(
            state,
            tooltip,
            "overrideItemLevel",
            Val::Num(f64::from(level)),
        );
    }
    publish_requirement(state, tooltip, catalog_item, required_level)?;
    state.push(tooltip);
    Ok(1)
}

/// WM94 §2: the published override keeps the supplied value; only the working
/// level used for line composition is clamped.
fn item_with_working_level(catalog_item: &ItemInfo, level: u32) -> ItemInfo {
    let mut item = catalog_item.clone();
    if level != 0 {
        item.item_level = level.clamp(WORKING_LEVEL_MIN, WORKING_LEVEL_MAX) as u16;
    }
    item
}

/// WM94 §2: zero skips the line; otherwise the greater of the supplied and the
/// item requirement is published, and the line only appears above level 1.
fn publish_requirement(
    state: &mut LuaState,
    tooltip: Val,
    catalog_item: &ItemInfo,
    required_level: u32,
) -> LuaResult<()> {
    let requirement = required_level.max(u32::from(catalog_item.required_level));
    if required_level == 0 || requirement <= 1 {
        return Ok(());
    }
    let text = format_global_numeric(state, "ITEM_MIN_LEVEL", f64::from(requirement))?;
    let line = append_line(state, tooltip, LINE_TYPE_REQUIREMENT, &text, None);
    table_set(state, line, "level", Val::Num(f64::from(requirement)));
    Ok(())
}
