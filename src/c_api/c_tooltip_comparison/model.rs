//! INFERRED simulator projection over the existing item and equipped-slot catalogs.
//! No initialized gear, equip event, native cache, or weapon/bag-selection policy is supplied.

use std::collections::BTreeMap;

use super::{ComparisonItem, ItemIdentity};
use crate::c_api::item_descriptor::{ItemDescriptor, parse_bounded_item_link};
use crate::c_api::item_spell::item_link_for_id;
use crate::c_api::item_tooltip::ItemTooltipStat;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::SimState;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

#[derive(Debug)]
pub(super) struct Selection {
    pub item: String,
    pub additional_items: Vec<String>,
}

struct ResolvedItem {
    id: u32,
    info: crate::items::ItemInfo,
    level: Option<u16>,
}

fn resolve(sim: &SimState, item: &ComparisonItem) -> Option<ResolvedItem> {
    let descriptor = match &item.identity {
        ItemIdentity::Guid { bag, slot, item_id } => {
            let instance = sim.bag_items.get(&(*bag, *slot))?;
            if instance.item_id != *item_id {
                return None;
            }
            if let Some(link) = &instance.hyperlink {
                let descriptor = parse_bounded_item_link(link)?.descriptor;
                if descriptor.item_id != *item_id {
                    return None;
                }
                descriptor
            } else {
                ItemDescriptor::catalog(*item_id)
            }
        }
        ItemIdentity::Hyperlink(link) => parse_bounded_item_link(link)?.descriptor,
    };
    let info = descriptor.resolve(sim.player.level).ok()??;
    let level = match item.override_level {
        Some(level) => u16::try_from(level).ok(),
        None => Some(info.item_level),
    };
    Some(ResolvedItem {
        id: descriptor.item_id,
        info,
        level,
    })
}

/// Supported worn armor/accessories, using simulator inventory slot IDs;
/// do not infer the special native two-hand or additional bag branches.
fn supported_slots(inventory_type: u8) -> &'static [i32] {
    match inventory_type {
        1 => &[1],       // head
        2 => &[2],       // neck
        3 => &[3],       // shoulders
        4 => &[4],       // shirt
        5 | 20 => &[5],  // chest/robe
        6 => &[6],       // waist
        7 => &[7],       // legs
        8 => &[8],       // feet
        9 => &[9],       // wrist
        10 => &[10],     // hands
        11 => &[11, 12], // rings
        12 => &[13, 14], // trinkets
        16 => &[15],     // back
        19 => &[19],     // tabard
        _ => &[],
    }
}

pub(super) fn select(
    state: &LuaState,
    comparison: &ComparisonItem,
) -> LuaResult<Option<Selection>> {
    let sim = borrow_state(state)?;
    let Some(source) = resolve(&sim, comparison) else {
        return Ok(None);
    };
    let slots = supported_slots(source.info.inventory_type);
    let mut links = slots.iter().filter_map(|slot| {
        let equipped = sim.player.equipped_items.get(slot)?;
        let info = crate::items::get_item(equipped.item_id)?;
        if !supported_slots(info.inventory_type).contains(slot) {
            return None;
        }
        // Equipped instances have no simulator publication GUID; use the
        // existing hyperlink exported for this item, not a fabricated ID.
        item_link_for_id(equipped.item_id)
    });
    let Some(first) = links.next() else {
        return Ok(None);
    };
    // R88n.6's normal group route replaces a first match with the second
    // and retains the first. Slot order here is simulator slot order.
    Ok(Some(match links.next() {
        Some(second) => Selection {
            item: second,
            additional_items: vec![first],
        },
        None => Selection {
            item: first,
            additional_items: vec![],
        },
    }))
}

fn stats<'a>(sim: &'a SimState, item: &ComparisonItem) -> Option<&'a [ItemTooltipStat]> {
    let resolved = resolve(sim, item)?;
    sim.item_tooltip_inputs
        .get(&(resolved.id, resolved.level?))
        .map(|input| input.stats.as_slice())
}

fn accumulate<'a>(delta: &mut BTreeMap<&'a str, i64>, rows: &'a [ItemTooltipStat], sign: i64) {
    for row in rows {
        *delta.entry(&row.label).or_default() += sign * i64::from(row.amount);
    }
}

pub(super) fn delta(
    state: &LuaState,
    candidate: &ComparisonItem,
    equipped: &ComparisonItem,
    paired: Option<&ComparisonItem>,
    add_paired: bool,
) -> LuaResult<Option<Vec<String>>> {
    let sim = borrow_state(state)?;
    let Some(candidate) = stats(&sim, candidate) else {
        return Ok(None);
    };
    let Some(equipped) = stats(&sim, equipped) else {
        return Ok(None);
    };
    let mut delta = BTreeMap::new();
    accumulate(&mut delta, candidate, 1);
    accumulate(&mut delta, equipped, -1);
    if let Some(paired) = paired {
        let Some(paired) = stats(&sim, paired) else {
            return Ok(None);
        };
        // Inferred from the original manager: an optional bag pair augments
        // the candidate side; replacing both equipped hands subtracts the pair.
        accumulate(&mut delta, paired, if add_paired { 1 } else { -1 });
    }
    // Host-supplied labels, deterministic order, no invented native colors or text.
    Ok(Some(
        delta
            .into_iter()
            .filter(|(_, amount)| *amount != 0)
            .map(|(label, amount)| format!("{amount:+} {label}"))
            .collect(),
    ))
}
