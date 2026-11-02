//! Rilua A_Admin handlers — Crafting / profession seeding.
//!
//! Provides `LearnRecipe`, `UnlearnRecipe`, `ClearKnownRecipes`,
//! `SetSelectedProfession`, `UnlearnProfession`, `RelearnProfession`,
//! plus the higher-level reagent seeders `SetReagentCount` and
//! `SeedReagentsForRecipe` that bypass the per-`(bag, slot)` bookkeeping
//! the generic `A_Admin.AddBagItem` requires — the Professions crafting
//! page just needs "the player has N of item X", not a specific slot.

use crate::c_api::item_upgrade_eligibility::{ItemSlot, discard_location};
use crate::lua_api::globals::missing_surface::professions::{
    abandon_profession_impl, relearn_profession_impl,
};
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::{BagItem, SimState};
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::BTreeMap;

pub(super) fn learn_recipe(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .crafting
        .known_recipe_ids
        .insert(recipe_id);
    Ok(0)
}

pub(super) fn unlearn_recipe(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .crafting
        .known_recipe_ids
        .remove(&recipe_id);
    Ok(0)
}

pub(super) fn clear_known_recipes(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.crafting.known_recipe_ids.clear();
    Ok(0)
}

pub(super) fn set_selected_profession(state: &mut LuaState) -> LuaResult<u32> {
    let val = Val::from_stack(state, 1)?;
    let id = match val {
        Val::Nil => None,
        Val::Num(n) => Some(n as i32),
        _ => None,
    };
    borrow_state_mut(state)?.crafting.selected_profession_id = id;
    Ok(0)
}

pub(super) fn unlearn_profession(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    abandon_profession_impl(state, skill_line_id);
    fire_named_event_state(state, "SKILL_LINES_CHANGED", &[]);
    Ok(0)
}

pub(super) fn relearn_profession(state: &mut LuaState) -> LuaResult<u32> {
    let skill_line_id = i32::from_stack(state, 1)?;
    relearn_profession_impl(state, skill_line_id);
    fire_named_event_state(state, "SKILL_LINES_CHANGED", &[]);
    Ok(0)
}

/// `A_Admin.SetReagentCount(item_id, qty)` — set the player's total
/// owned count of `item_id` across all bag slots to exactly `qty`. If
/// `qty == 0`, every slot holding that item is removed. Otherwise the
/// first existing slot with that item is replaced; if none exists, a
/// fresh slot is appended to bag 0 (lowest free slot).
pub(super) fn set_reagent_count(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let qty = i32::from_stack(state, 2)?;
    apply_reagent_count(state, item_id, qty);
    Ok(0)
}

/// `A_Admin.SeedReagentsForRecipe(recipe_id, count)` — bulk-seed every
/// reagent in `recipe_id` at `required_count * count`, so the recipe
/// becomes craftable exactly `count` times. Dependent reagents declared
/// on each slot (e.g. a Spark of Omens required alongside a Crest) are
/// seeded too — without them, `HasMissingDependentReagents` would still
/// block crafting. Returns `true` on success; `false` (no-op) when
/// `recipe_id` is unknown to `profession_data`.
pub(super) fn seed_reagents_for_recipe(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let count = i32::from_stack(state, 2)?.max(0);
    let recipe = profession_data::get_recipe(recipe_id);
    let Some(recipe) = recipe else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let mut plan = BTreeMap::new();
    for r in recipe.reagents {
        seed_reagent_plan(&mut plan, r, count);
    }
    for (item_id, qty) in plan {
        apply_reagent_count(state, item_id, qty);
    }
    state.push(Val::Bool(true));
    Ok(1)
}

fn seed_reagent_plan(
    plan: &mut BTreeMap<u32, i32>,
    reagent: &profession_data::ReagentSlot,
    count: i32,
) {
    *plan.entry(reagent.item_id).or_insert(0) += reagent.quantity * count;
    for dep in reagent.dependent_reagents {
        seed_reagent_plan(plan, dep, count);
    }
}

fn apply_reagent_count(state: &mut LuaState, item_id: u32, qty: i32) {
    let existing_slots = collect_slots_for_item(state, item_id);
    let mut sim = match borrow_state_mut(state) {
        Ok(sim) => sim,
        Err(_) => return,
    };
    if qty <= 0 {
        for key in existing_slots {
            remove_reagent_slot(&mut sim, key);
        }
        return;
    }
    if let Some(first_key) = existing_slots.first() {
        // Collapse extras, then set the primary stack to the new total.
        for key in existing_slots.iter().skip(1) {
            remove_reagent_slot(&mut sim, *key);
        }
        if let Some(slot) = sim.bag_items.get_mut(first_key) {
            slot.stack_count = qty;
        }
        return;
    }
    insert_reagent_stack(&mut sim, item_id, qty);
}

fn remove_reagent_slot(sim: &mut SimState, key: (i32, i32)) {
    discard_location(sim, ItemSlot::Bag(key.0, key.1));
    sim.bag_items.remove(&key);
}

fn insert_reagent_stack(sim: &mut SimState, item_id: u32, qty: i32) {
    let next_slot = next_free_slot_in_bag(sim, 0);
    let key = (0, next_slot);
    discard_location(sim, ItemSlot::Bag(key.0, key.1));
    sim.bag_items.insert(
        key,
        BagItem {
            item_id,
            stack_count: qty,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
}

fn collect_slots_for_item(state: &LuaState, item_id: u32) -> Vec<(i32, i32)> {
    let Ok(sim) = borrow_state(state) else {
        return Vec::new();
    };
    let mut keys: Vec<(i32, i32)> = sim
        .bag_items
        .iter()
        .filter_map(|(key, item)| (item.item_id == item_id).then_some(*key))
        .collect();
    keys.sort();
    keys
}

fn next_free_slot_in_bag(sim: &crate::lua_api::state::SimState, bag: i32) -> i32 {
    let mut taken: Vec<i32> = sim
        .bag_items
        .keys()
        .filter_map(|(b, slot)| (*b == bag).then_some(*slot))
        .collect();
    taken.sort();
    let mut next = 1;
    for slot in taken {
        if slot == next {
            next += 1;
        } else if slot > next {
            break;
        }
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_reagent_plan_recurses_and_aggregates() {
        static GRANDCHILD: &[profession_data::ReagentSlot] = &[profession_data::ReagentSlot {
            item_id: 3,
            quantity: 4,
            dependent_reagents: &[],
        }];
        static CHILD_A: &[profession_data::ReagentSlot] = &[profession_data::ReagentSlot {
            item_id: 2,
            quantity: 3,
            dependent_reagents: GRANDCHILD,
        }];
        static CHILD_B: &[profession_data::ReagentSlot] = &[profession_data::ReagentSlot {
            item_id: 2,
            quantity: 1,
            dependent_reagents: &[],
        }];
        let reagent = profession_data::ReagentSlot {
            item_id: 1,
            quantity: 2,
            dependent_reagents: CHILD_A,
        };
        let mut plan = BTreeMap::new();
        seed_reagent_plan(&mut plan, &reagent, 5);
        seed_reagent_plan(
            &mut plan,
            &profession_data::ReagentSlot {
                item_id: 4,
                quantity: 1,
                dependent_reagents: CHILD_B,
            },
            5,
        );
        assert_eq!(plan.get(&1), Some(&10));
        assert_eq!(plan.get(&2), Some(&20));
        assert_eq!(plan.get(&3), Some(&20));
        assert_eq!(plan.get(&4), Some(&5));
    }
}
