//! R88p.p3 reagent inventory selection. No bank fetch or update-event claims.
use crate::lua_api::SimState;
use std::collections::{HashMap, HashSet};

/// Explicit metadata and nested-container relationships, empty until supplied.
/// UNKNOWN: native property decoding, item-eligibility details and update writers.
#[derive(Debug, Default)]
pub struct SpellReagentInventory {
    pub metadata_masks: HashMap<i32, u32>,
    /// Child bag ID -> the parent bag/slot containing that container item.
    pub container_parents: HashMap<i32, (i32, i32)>,
    /// Empty-default candidates; native property identity and decoding are UNKNOWN.
    pub property_candidates: HashMap<i32, Vec<SpellReagentPropertyCandidate>>,
}

/// One supplied item candidate without a native property ID or schema claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpellReagentPropertyCandidate {
    pub signed_value: i64,
    pub available_count: i64,
}

pub(crate) fn matching_count(sim: &SimState, reagent_id: i32) -> i64 {
    let mask = sim
        .spell_reagent_inventory
        .metadata_masks
        .get(&reagent_id)
        .copied()
        .unwrap_or(0);
    let bags = sim
        .bag_items
        .iter()
        .filter(|((bag, _), item)| {
            item.item_id as i64 == i64::from(reagent_id) && admitted_container(sim, *bag, mask)
        })
        .map(|(_, item)| i64::from(item.stack_count))
        .sum::<i64>();
    bags + matching_roots(sim, reagent_id, mask)
}

fn admitted_root(bag: i32, mask: u32) -> bool {
    use crate::client_profile::{ACTIVE, ClientProfile};
    // INFERRED applicability; Forever's existing Enum.BagIndex bank tabs end at23.
    let last_bank = if ACTIVE == ClientProfile::WowForever {
        23
    } else {
        16
    };
    (0..=5).contains(&bag) || (mask & 32 != 0 && (6..=last_bank).contains(&bag))
}

fn admitted_container(sim: &SimState, mut bag: i32, mask: u32) -> bool {
    let mut visited = HashSet::new();
    while let Some(&(parent, slot)) = sim.spell_reagent_inventory.container_parents.get(&bag) {
        if !visited.insert(bag) || !sim.bag_items.contains_key(&(parent, slot)) {
            return false;
        }
        bag = parent;
    }
    admitted_root(bag, mask)
}

fn matching_roots(sim: &SimState, reagent_id: i32, mask: u32) -> i64 {
    let equipped = sim
        .player
        .equipped_items
        .iter()
        .filter(|(slot, item)| {
            equipped_root_allowed(sim, **slot, item.item_id, mask)
                && i64::from(item.item_id) == i64::from(reagent_id)
        })
        .count() as i64;
    equipped + matching_container_roots(sim, reagent_id, mask)
}

fn matching_container_roots(sim: &SimState, reagent_id: i32, mask: u32) -> i64 {
    sim.bag_info
        .iter()
        .filter(|(bag, info)| {
            **bag != 0
                && container_is_root(sim, **bag, mask)
                && info
                    .item_id
                    .is_some_and(|id| i64::from(id) == i64::from(reagent_id))
                && !info
                    .inventory_slot
                    .is_some_and(|slot| sim.player.equipped_items.contains_key(&slot))
        })
        .count() as i64
}

fn container_is_root(sim: &SimState, bag: i32, mask: u32) -> bool {
    !sim.spell_reagent_inventory
        .container_parents
        .contains_key(&bag)
        && admitted_root(bag, mask)
}

/// Mask 2 only enables a property scan, not alternate arithmetic by itself.
/// UNKNOWN property backing is absent, so no qualifying alternate helper exists.
pub(crate) fn alternate_limit(_sim: &SimState, _reagent_id: i32, _quantity: i16) -> Option<i64> {
    None
}

fn equipped_root_allowed(sim: &SimState, slot: i32, item_id: u32, mask: u32) -> bool {
    let container = sim
        .bag_info
        .iter()
        .find(|(_, info)| info.item_id == Some(item_id) && info.inventory_slot == Some(slot));
    container.map_or((1..=30).contains(&slot), |(bag, _)| {
        !sim.spell_reagent_inventory
            .container_parents
            .contains_key(bag)
            && admitted_root(*bag, mask)
    })
}
