//! Bounded local crafting orders. Host recipe/ability mapping owns catalog input.
use super::{crafting_input, crafting_reagents::*, crafting_tables};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let trade = super::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(state, trade, "GetItemSlotModifications", item_modifications)?;
    table_set_rust_fn_static(
        state,
        trade,
        "GetItemSlotModificationsForOrder",
        order_modifications,
    )?;
    let orders = super::ensure_namespace(state, "C_CraftingOrders")?;
    table_set_rust_fn_static(state, orders, "PlaceNewOrder", place_order)?;
    table_set_rust_fn_static(state, orders, "GetMyOrders", my_orders)
}
fn item_modifications(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let mods = borrow_state(state)?
        .crafting
        .reagents
        .item_modifications
        .get(&guid)
        .cloned()
        .unwrap_or_default();
    let table = crafting_tables::modifications(state, &mods);
    state.push(table);
    Ok(1)
}
fn order_modifications(state: &mut LuaState) -> LuaResult<u32> {
    let id = String::from_stack(state, 1)?
        .parse::<u32>()
        .map_err(|_| runtime_error("invalid orderID"))?;
    let sim = borrow_state(state)?;
    let mods = sim
        .crafting
        .reagents
        .orders
        .get(&id)
        .and_then(|order| order.request.recraft_item.as_ref())
        .and_then(|guid| sim.crafting.reagents.item_modifications.get(guid))
        .cloned()
        .unwrap_or_default();
    drop(sim);
    let table = crafting_tables::modifications(state, &mods);
    state.push(table);
    Ok(1)
}
fn map_order_reagents(
    slots: &[ReagentSlotSchematic],
    request: &NewCraftingOrderInfo,
) -> LuaResult<Vec<CraftingOrderReagentInfo>> {
    let mut allocations = request.crafting_reagent_items.clone();
    for entry in &request.reagent_infos {
        let slot = slots
            .iter()
            .find(|slot| slot.reagents.contains(&entry.reagent))
            .ok_or_else(|| runtime_error("order reagent not in recipe"))?;
        allocations.push(CraftingReagentInfo {
            reagent: entry.reagent,
            data_slot_index: slot.data_slot_index,
            quantity: entry.quantity,
        });
    }
    validate_allocations(slots, &allocations).map_err(runtime_error)?;
    Ok(allocations
        .into_iter()
        .map(|reagent_info| {
            let slot = slots
                .iter()
                .find(|slot| slot.data_slot_index == reagent_info.data_slot_index)
                .expect("validated slot");
            // Customer supplied entries are source Customer; Basic reagent type is enum value 1.
            CraftingOrderReagentInfo {
                reagent_info,
                slot_index: slot.slot_index,
                source: 1,
                is_basic_reagent: slot.reagent_type == 1,
            }
        })
        .collect())
}
fn map_request_to_recipe(
    inputs: &CraftingInputs,
    request: &NewCraftingOrderInfo,
) -> LuaResult<(i32, Vec<CraftingOrderReagentInfo>)> {
    let recipe_id = *inputs
        .order_recipes
        .get(&request.skill_line_ability_id)
        .ok_or_else(|| runtime_error("crafting order ability is not configured"))?;
    crate::lua_api::globals::profession_data::get_recipe(recipe_id)
        .ok_or_else(|| runtime_error("crafting order recipe is not in the catalog"))?;
    let slots = inputs
        .recipe_slots
        .get(&recipe_id)
        .ok_or_else(|| runtime_error("crafting order recipe is not configured"))?;
    let reagents = map_order_reagents(slots, request)?;
    Ok((recipe_id, reagents))
}
fn place_order(state: &mut LuaState) -> LuaResult<u32> {
    let request = crafting_input::read_order(state, stack_val(state, 1))?;
    // Generated ProfessionConstants: order types 0..3 and durations 0..2.
    if !(0..=3).contains(&request.order_type) || !(0..=2).contains(&request.order_duration) {
        return Err(runtime_error("unknown crafting order type or duration"));
    }
    let (recipe_id, reagents) = {
        let sim = borrow_state(state)?;
        map_request_to_recipe(&sim.crafting.reagents, &request)?
    };
    let mut sim = borrow_state_mut(state)?;
    let inputs = &mut sim.crafting.reagents;
    // INFERRED: simulator-local monotonic IDs; no server placement/escrow is claimed.
    let order_id = inputs.next_order_id;
    let next_order_id = order_id
        .checked_add(1)
        .ok_or_else(|| runtime_error("crafting order IDs exhausted"))?;
    let details = inputs
        .placement_results
        .pop_front()
        .ok_or_else(|| runtime_error("crafting order host placement result is not configured"))?;
    inputs.next_order_id = next_order_id;
    inputs.orders.insert(
        order_id,
        CraftingOrder {
            order_id,
            recipe_id,
            request,
            reagents,
            details,
        },
    );
    Ok(0)
}
fn my_orders(state: &mut LuaState) -> LuaResult<u32> {
    let records: Vec<_> = borrow_state(state)?
        .crafting
        .reagents
        .orders
        .values()
        .cloned()
        .collect();
    let table = crafting_tables::orders(state, &records);
    state.push(table);
    Ok(1)
}
