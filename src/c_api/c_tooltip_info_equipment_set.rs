//! WM94 §6: `GetEquipmentSet` summarises a saved equipment set.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:271` declare the getter.
//!
//! Modelled from WM94: a header double line with the set name and the formatted
//! `ITEMS_VARIABLE_QUANTITY` count, then only nonzero `ITEMS_EQUIPPED`
//! (normal color), `ITEMS_IN_INVENTORY` (highlight color) and
//! `ITEM_SLOTS_IGNORED` (disabled color) counts, then one `ITEM_MISSING` line
//! per unfilled non-ignored inventory slot in reverse slot order.
//!
//! Native line types and the header's own color are UNKNOWN; lines are
//! published as plain lines. The equipped/inventory counting rules are EMULATED
//! from the saved set's snapshot against current equipment and bag contents.

use std::collections::{BTreeSet, HashMap};

use crate::c_api::tooltip_getter_args::signed;
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_double_line, append_line, compose_tooltip, format_global_numeric, format_global_text,
    global_text, named_color,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

/// `Enum.TooltipDataType.EquipmentSet` (TooltipInfoSharedDocumentation).
const TOOLTIP_TYPE_EQUIPMENT_SET: f64 = 14.0;
const LINE_TYPE_PLAIN: f64 = 0.0;

/// Saved-set slot labels in inventory-slot order (1-based slots 1..=19).
const SLOT_LABELS: [&str; 19] = [
    "HEADSLOT",
    "NECKSLOT",
    "SHOULDERSLOT",
    "SHIRTSLOT",
    "CHESTSLOT",
    "WAISTSLOT",
    "LEGSSLOT",
    "FEETSLOT",
    "WRISTSLOT",
    "HANDSSLOT",
    "FINGER0SLOT",
    "FINGER1SLOT",
    "TRINKET0SLOT",
    "TRINKET1SLOT",
    "BACKSLOT",
    "MAINHANDSLOT",
    "SECONDARYHANDSLOT",
    "RANGEDSLOT",
    "TABARDSLOT",
];

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetEquipmentSet", get_equipment_set)
}

struct SetSummary {
    id: i32,
    name: String,
    items: u32,
    equipped: u32,
    in_bags: u32,
    ignored: u32,
    missing_slots: BTreeSet<usize>,
}

fn get_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = signed(state, 1)?;
    let summary = {
        let sim = borrow_state(state)?;
        summarize_set(&sim, set_id)
    };
    let Some(summary) = summary else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    publish_summary(state, &summary)?;
    Ok(1)
}

fn summarize_set(sim: &SimState, set_id: i32) -> Option<SetSummary> {
    let set = sim
        .equipment_manager
        .sets
        .iter()
        .find(|set| set.id == set_id)?;
    let saved: HashMap<i32, u32> = set
        .item_ids
        .iter()
        .filter(|(_, item_id)| **item_id != 0)
        .map(|(slot, item_id)| (*slot, *item_id))
        .collect();
    let ignored_slots: BTreeSet<i32> = set.ignored_slots.iter().copied().collect();
    Some(SetSummary {
        id: set.id,
        name: set.name.clone(),
        items: saved.len() as u32,
        equipped: count_equipped(sim, &saved),
        in_bags: count_in_bags(sim, &saved),
        ignored: ignored_slots.len() as u32,
        missing_slots: unlisted_slots(&saved, &ignored_slots),
    })
}

fn count_equipped(sim: &SimState, saved: &HashMap<i32, u32>) -> u32 {
    saved
        .iter()
        .filter(|(slot, item_id)| {
            sim.player
                .equipped_items
                .get(slot)
                .is_some_and(|equipped| equipped.item_id == **item_id)
        })
        .count() as u32
}

fn count_in_bags(sim: &SimState, saved: &HashMap<i32, u32>) -> u32 {
    saved
        .values()
        .filter(|item_id| {
            sim.bag_items
                .values()
                .any(|bag_item| bag_item.item_id == **item_id)
        })
        .count() as u32
}

fn unlisted_slots(saved: &HashMap<i32, u32>, ignored_slots: &BTreeSet<i32>) -> BTreeSet<usize> {
    SLOT_LABELS
        .iter()
        .enumerate()
        .filter(|(index, _)| {
            let slot = *index as i32 + 1;
            !saved.contains_key(&slot) && !ignored_slots.contains(&slot)
        })
        .map(|(index, _)| index)
        .collect()
}

fn publish_summary(state: &mut LuaState, summary: &SetSummary) -> LuaResult<()> {
    let tooltip = compose_tooltip(
        state,
        TOOLTIP_TYPE_EQUIPMENT_SET,
        Some(f64::from(summary.id)),
    );
    publish_header(state, tooltip, summary)?;
    publish_count(
        state,
        tooltip,
        "ITEMS_EQUIPPED",
        summary.equipped,
        "NORMAL_FONT_COLOR",
    )?;
    publish_count(
        state,
        tooltip,
        "ITEMS_IN_INVENTORY",
        summary.in_bags,
        "HIGHLIGHT_FONT_COLOR",
    )?;
    publish_count(
        state,
        tooltip,
        "ITEM_SLOTS_IGNORED",
        summary.ignored,
        "DISABLED_FONT_COLOR",
    )?;
    publish_missing_slots(state, tooltip, &summary.missing_slots)?;
    state.push(tooltip);
    Ok(())
}

fn publish_header(state: &mut LuaState, tooltip: Val, summary: &SetSummary) -> LuaResult<()> {
    let items = format_global_numeric(state, "ITEMS_VARIABLE_QUANTITY", f64::from(summary.items))?;
    append_double_line(
        state,
        tooltip,
        LINE_TYPE_PLAIN,
        &summary.name,
        None,
        &items,
        None,
    );
    Ok(())
}

/// WM94 §6: only nonzero counts are published.
fn publish_count(
    state: &mut LuaState,
    tooltip: Val,
    key: &str,
    count: u32,
    color_key: &str,
) -> LuaResult<()> {
    if count == 0 {
        return Ok(());
    }
    let text = format_global_numeric(state, key, f64::from(count))?;
    append_line(
        state,
        tooltip,
        LINE_TYPE_PLAIN,
        &text,
        named_color(state, color_key),
    );
    Ok(())
}

/// WM94 §6: reverse missing-slot order; labels come from the pinned slot strings.
fn publish_missing_slots(
    state: &mut LuaState,
    tooltip: Val,
    missing: &BTreeSet<usize>,
) -> LuaResult<()> {
    let color = named_color(state, "ERROR_COLOR");
    for index in missing.iter().rev() {
        let key = SLOT_LABELS[*index];
        let label = global_text(state, key)
            .ok_or_else(|| rilua::runtime_error(format!("missing slot label {key}")))?;
        let text = format_global_text(state, "ITEM_MISSING", &label)?;
        append_line(state, tooltip, LINE_TYPE_PLAIN, &text, color);
    }
    Ok(())
}
