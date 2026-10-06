//! Crafting inventory transactions and result production for C_TradeSkillUI.

use super::{crafting_input, crafting_plan, crafting_tables};
use crate::items;
use crate::lua_api::game_data::CastingState;
use crate::lua_api::globals::profession_data;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::BagItem;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::{BTreeSet, HashMap};

const CRAFTING_CAST_DURATION_SECONDS: f64 = 2.0;
const DEFAULT_CRAFTING_ICON: &str = "Interface/Icons/INV_Misc_QuestionMark";

/// Returns true iff the recipe exists in the catalogue AND all reagents are
/// available in `player.bag_items` for the requested `count`.
pub(crate) fn recipe_is_craftable(state: &mut LuaState, recipe_id: i32, count: i32) -> bool {
    let Ok(sim) = borrow_state(state) else {
        return false;
    };
    let Some(slots) = sim.crafting.reagents.recipe_slots.get(&recipe_id) else {
        return false;
    };
    let allocations = crafting_plan::default_allocations(slots);
    crafting_plan::plan_deltas(&sim.crafting.reagents, recipe_id, &allocations, count)
        .is_ok_and(|deltas| crafting_plan::has_resources(&sim, &deltas))
}

/// Consumes reagents from bags and adds output item if all reagents are available.
/// Returns false and changes nothing when reagents are insufficient.
pub(crate) fn craft_recipe(state: &mut LuaState, recipe_id: i32, count: i32) -> LuaResult<bool> {
    let Some(plan) = craft_plan(state, recipe_id, count)? else {
        return Ok(false);
    };

    let (affected_bags, output_guid) = {
        let mut sim = borrow_state_mut(state)?;
        let Some(bags) = commit_inventory(&mut sim, &plan) else {
            return Ok(false);
        };
        let (location, _) = sim
            .bag_items
            .iter()
            .find(|(_, item)| item.item_id == plan.output_item_id)
            .expect("crafted output inventory committed");
        let guid =
            super::item_spell::item_guid_for_bag_slot(location.0, location.1, plan.output_item_id);
        (bags, guid)
    };

    start_crafting_cast(state, &plan);

    for bag_id in &affected_bags {
        fire_named_event_state(state, "BAG_UPDATE", &[Val::Num(*bag_id as f64)]);
    }
    fire_named_event_state(state, "BAG_UPDATE_DELAYED", &[]);

    publish_crafting_result(state, &plan, &output_guid);
    Ok(true)
}

fn commit_inventory(
    sim: &mut crate::lua_api::state::SimState,
    plan: &CraftPlan,
) -> Option<BTreeSet<i32>> {
    let has_output_stack = sim
        .bag_items
        .values()
        .any(|slot| slot.item_id == plan.output_item_id);
    if !has_output_stack && free_bag0_slot(sim).is_none() {
        return None;
    }
    let mut affected_bags = BTreeSet::new();
    let capacity = sim.bag_num_slots(0);
    consume_reagents(&mut sim.bag_items, &plan.reagents.items, &mut affected_bags);
    for (id, quantity) in &plan.reagents.currencies {
        sim.currency_info
            .get_mut(&(*id as i32))
            .expect("currency preflighted")
            .quantity -= quantity;
    }
    let output_bag = add_output_item(
        &mut sim.bag_items,
        capacity,
        plan.output_item_id,
        plan.output_count,
    );
    affected_bags.insert(output_bag);
    Some(affected_bags)
}

struct CraftPlan {
    recipe_id: i32,
    cast_name: String,
    reagents: crafting_plan::ReagentDeltas,
    output_item_id: u32,
    output_count: i32,
    hyperlink: String,
}

fn craft_plan(state: &mut LuaState, recipe_id: i32, count: i32) -> LuaResult<Option<CraftPlan>> {
    let Some(recipe) = profession_data::get_recipe(recipe_id) else {
        return Ok(None);
    };
    let provided = stack_val(state, 3);
    let allocations = if matches!(provided, Val::Nil) {
        let sim = borrow_state(state)?;
        let slots = sim
            .crafting
            .reagents
            .recipe_slots
            .get(&recipe_id)
            .expect("catalog schematic installed");
        crafting_plan::default_allocations(slots)
    } else {
        crafting_input::read_allocations(state, provided)?
    };
    let sim = borrow_state(state)?;
    let reagents =
        crafting_plan::plan_deltas(&sim.crafting.reagents, recipe_id, &allocations, count)?;
    if !crafting_plan::has_resources(&sim, &reagents) {
        return Ok(None);
    }
    Ok(Some(CraftPlan {
        recipe_id,
        cast_name: crafted_item_name(recipe).into(),
        reagents,
        output_item_id: recipe.output_item_id,
        output_count: count,
        hyperlink: super::item_spell::item_link_for_id(recipe.output_item_id)
            .ok_or_else(|| rilua::runtime_error("crafted output item catalog data unavailable"))?,
    }))
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
        duration: CRAFTING_CAST_DURATION_SECONDS,
        cast_id,
        target: None,
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

fn publish_crafting_result(state: &mut LuaState, plan: &CraftPlan, output_guid: &str) {
    let table = crafting_tables::rooted_table(state);
    let returns = crafting_tables::regular_array(state, &plan.reagents.returns);
    crate::lua_api::methods::table_set(state, table, "resourcesReturned", returns);
    populate_result_fields(state, table, plan, output_guid);
    // INFERRED: result publication accompanies the existing immediate inventory commit;
    // native asynchronous crafting completion/proc probabilities are not simulated here.
    fire_named_event_state(state, "TRADE_SKILL_ITEM_CRAFTED_RESULT", &[table]);
}
fn populate_result_fields(state: &mut LuaState, table: Val, plan: &CraftPlan, output_guid: &str) {
    for (key, value) in [
        ("itemID", plan.output_item_id as i32),
        ("quantity", plan.output_count),
        ("qualityProgress", 0),
        ("critBonusSkill", 0),
        ("multicraft", 0),
        ("operationID", 0),
        ("concentrationCurrencyID", 0),
        ("concentrationSpent", 0),
        ("ingenuityRefund", 0),
    ] {
        crate::lua_api::methods::table_set(state, table, key, Val::Num(f64::from(value)));
    }
    for key in [
        "isCrit",
        "recraftable",
        "bonusCraft",
        "firstCraftReward",
        "isEnchant",
        "hasIngenuityProc",
    ] {
        crate::lua_api::methods::table_set(state, table, key, Val::Bool(false));
    }
    let guid = crate::lua_api::methods::create_string(state, output_guid);
    crate::lua_api::methods::table_set(state, table, "itemGUID", guid);
    let hyperlink = crate::lua_api::methods::create_string(state, &plan.hyperlink);
    crate::lua_api::methods::table_set(state, table, "hyperlink", hyperlink);
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

fn add_output_item(
    bag_items: &mut HashMap<(i32, i32), BagItem>,
    backpack_capacity: i32,
    item_id: u32,
    count: i32,
) -> i32 {
    if let Some((key, slot)) = bag_items
        .iter_mut()
        .find(|(_, slot)| slot.item_id == item_id)
    {
        slot.stack_count += count;
        return key.0;
    }

    let key = (1..=backpack_capacity)
        .map(|slot| (0, slot))
        .find(|slot| !bag_items.contains_key(slot))
        .expect("output slot preflighted before consuming reagents");
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

fn free_bag0_slot(sim: &crate::lua_api::state::SimState) -> Option<(i32, i32)> {
    (1..=sim.bag_num_slots(0))
        .map(|slot| (0, slot))
        .find(|slot| !sim.bag_items.contains_key(slot))
}
