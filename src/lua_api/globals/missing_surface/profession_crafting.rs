//! Crafting helpers for `professions.rs`.
//!
//! Extracted to keep `professions.rs` under 750 lines.
//! Provides `recipe_is_craftable` and `craft_recipe`, called by the
//! `C_TradeSkillUI.IsRecipeCraftable` and `C_TradeSkillUI.CraftRecipe`
//! dispatchers in `professions.rs`.

use crate::items;
use crate::lua_api::game_data::CastingState;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::BagItem;
use rilua::Val;
use rilua::vm::state::LuaState;
use std::collections::{BTreeSet, HashMap};

const CRAFTING_CAST_DURATION_SECONDS: f64 = 2.0;
const DEFAULT_CRAFTING_ICON: &str = "Interface/Icons/INV_Misc_QuestionMark";

/// Returns true iff the recipe exists in the catalogue AND all reagents are
/// available in `player.bag_items` for the requested `count`.
pub(super) fn recipe_is_craftable(state: &mut LuaState, recipe_id: i32, count: i32) -> bool {
    let Some(recipe) = profession_data::get_recipe(recipe_id) else {
        return false;
    };
    let Ok(sim) = borrow_state(state) else {
        return false;
    };
    for reagent in recipe.reagents {
        let needed = reagent.quantity * count;
        let have: i32 = sim
            .bag_items
            .values()
            .filter(|b| b.item_id == reagent.item_id)
            .map(|b| b.stack_count)
            .sum();
        if have < needed {
            return false;
        }
    }
    true
}

/// Consumes reagents from bags and adds output item if all reagents are available.
/// Returns false and changes nothing when reagents are insufficient.
pub(super) fn craft_recipe(state: &mut LuaState, recipe_id: i32, count: i32) -> bool {
    let Some(plan) = craft_plan(state, recipe_id, count) else {
        return false;
    };

    let mut affected_bags = BTreeSet::new();
    {
        let Ok(mut sim) = borrow_state_mut(state) else {
            return false;
        };

        consume_reagents(&mut sim.bag_items, &plan.reagent_deltas, &mut affected_bags);
        let output_bag =
            add_output_item(&mut sim.bag_items, plan.output_item_id, plan.output_count);
        affected_bags.insert(output_bag);
    }

    start_crafting_cast(state, &plan);

    for bag_id in &affected_bags {
        fire_named_event_state(state, "BAG_UPDATE", &[Val::Num(*bag_id as f64)]);
    }
    fire_named_event_state(state, "BAG_UPDATE_DELAYED", &[]);

    true
}

struct CraftPlan {
    recipe_id: i32,
    cast_name: String,
    reagent_deltas: Vec<(u32, i32)>,
    output_item_id: u32,
    output_count: i32,
}

fn craft_plan(state: &mut LuaState, recipe_id: i32, count: i32) -> Option<CraftPlan> {
    if !recipe_is_craftable(state, recipe_id, count) {
        return None;
    }

    let recipe = profession_data::get_recipe(recipe_id)?;
    Some(CraftPlan {
        recipe_id: recipe.recipe_id,
        cast_name: crafted_item_name(recipe).to_string(),
        reagent_deltas: reagent_deltas(recipe, count),
        output_item_id: recipe.output_item_id,
        output_count: count,
    })
}

fn start_crafting_cast(state: &mut LuaState, plan: &CraftPlan) {
    let Ok(mut sim) = borrow_state_mut(state) else {
        return;
    };
    let now = sim.start_time.elapsed().as_secs_f64();
    let cast_id = sim.next_cast_id;
    sim.next_cast_id = sim.next_cast_id.wrapping_add(1);
    #[cfg(feature = "retail-12-1-0")]
    crate::lua_api::spellcast_events::clear_replaced_specialization(&mut sim);
    sim.casting = Some(CastingState {
        spell_id: plan.recipe_id as u32,
        spell_name: plan.cast_name.clone(),
        icon_path: DEFAULT_CRAFTING_ICON.to_string(),
        start_time: now,
        end_time: now + CRAFTING_CAST_DURATION_SECONDS,
        cast_id,
        empower: None,
        delay_time: 0.0,
    });
    drop(sim);

    crate::lua_api::spellcast_events::fire_player_cast_start(state, cast_id, plan.recipe_id as u32);
}

fn crafted_item_name(recipe: &profession_data::RecipeEntry) -> &'static str {
    items::get_item(recipe.output_item_id)
        .map(|item| item.name)
        .unwrap_or(recipe.name)
}

fn reagent_deltas(recipe: &profession_data::RecipeEntry, count: i32) -> Vec<(u32, i32)> {
    recipe
        .reagents
        .iter()
        .map(|reagent| (reagent.item_id, reagent.quantity * count))
        .collect()
}

fn consume_reagents(
    bag_items: &mut HashMap<(i32, i32), BagItem>,
    reagent_deltas: &[(u32, i32)],
    affected_bags: &mut BTreeSet<i32>,
) {
    for &(item_id, needed) in reagent_deltas {
        consume_item_stacks(bag_items, item_id, needed, affected_bags);
    }
    bag_items.retain(|_, item| item.stack_count > 0);
}

fn consume_item_stacks(
    bag_items: &mut HashMap<(i32, i32), BagItem>,
    item_id: u32,
    mut needed: i32,
    affected_bags: &mut BTreeSet<i32>,
) {
    for ((bag_id, _), slot) in bag_items.iter_mut() {
        if slot.item_id != item_id || needed == 0 {
            continue;
        }
        let taken = needed.min(slot.stack_count);
        slot.stack_count -= taken;
        needed -= taken;
        affected_bags.insert(*bag_id);
    }
}

fn add_output_item(bag_items: &mut HashMap<(i32, i32), BagItem>, item_id: u32, count: i32) -> i32 {
    if let Some((key, slot)) = bag_items
        .iter_mut()
        .find(|(_, slot)| slot.item_id == item_id)
    {
        slot.stack_count += count;
        return key.0;
    }

    let key = free_bag0_slot(bag_items);
    bag_items.insert(
        key,
        BagItem {
            item_id,
            stack_count: count,
            hyperlink: None,
        },
    );
    key.0
}

/// Find a free slot in bag 0 (slots 1–16). Falls back to negative slots if
/// all 16 are occupied (tests rarely exceed that).
fn free_bag0_slot(bag_items: &HashMap<(i32, i32), BagItem>) -> (i32, i32) {
    for slot in 1..=16_i32 {
        let k = (0, slot);
        if !bag_items.contains_key(&k) {
            return k;
        }
    }
    // overflow safety: negative slots are never valid WoW slots
    for slot in (-1000..=-1_i32).rev() {
        let k = (0, slot);
        if !bag_items.contains_key(&k) {
            return k;
        }
    }
    unreachable!("bag 0 exhausted")
}
