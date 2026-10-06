//! Serialization of profession DTOs. Builders keep every intermediate table rooted.
use super::crafting_reagents::*;
use crate::lua_api::methods::{create_string, create_table, table_get, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn rooted_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}
fn number(state: &mut LuaState, table: Val, key: &str, value: i32) {
    table_set(state, table, key, Val::Num(f64::from(value)));
}
fn text(state: &mut LuaState, table: Val, key: &str, value: &str) {
    let value = create_string(state, value);
    table_set(state, table, key, value);
}
fn array_entry(state: &mut LuaState, table: Val, index: usize, value: Val) {
    if let Val::Table(table) = table {
        crate::lua_api::methods::table_set_num(state, table, (index + 1) as f64, value);
    }
}

pub(crate) fn identity(state: &mut LuaState, reagent: CraftingReagent) -> Val {
    let table = rooted_table(state);
    let (key, id) = match reagent {
        CraftingReagent::Item(id) => ("itemID", id),
        CraftingReagent::Currency(id) => ("currencyID", id),
    };
    table_set(state, table, key, Val::Num(f64::from(id)));
    table
}
pub(crate) fn regular(state: &mut LuaState, info: &RegularReagentInfo) -> Val {
    let table = rooted_table(state);
    let reagent = identity(state, info.reagent);
    table_set(state, table, "reagent", reagent);
    number(state, table, "quantity", info.quantity);
    table
}
pub(crate) fn allocation(state: &mut LuaState, info: &CraftingReagentInfo) -> Val {
    let table = regular(
        state,
        &RegularReagentInfo {
            reagent: info.reagent,
            quantity: info.quantity,
        },
    );
    number(state, table, "dataSlotIndex", info.data_slot_index);
    table
}
pub(crate) fn modifications(state: &mut LuaState, mods: &[ItemSlotModification]) -> Val {
    let table = rooted_table(state);
    for (index, modification) in mods.iter().enumerate() {
        let value = rooted_table(state);
        number(state, value, "dataSlotIndex", modification.data_slot_index);
        let reagent = identity(state, modification.reagent);
        table_set(state, value, "reagent", reagent);
        array_entry(state, table, index, value);
    }
    table
}
pub(crate) fn regular_array(state: &mut LuaState, infos: &[RegularReagentInfo]) -> Val {
    let table = rooted_table(state);
    for (index, info) in infos.iter().enumerate() {
        let value = regular(state, info);
        array_entry(state, table, index, value);
    }
    table
}
fn slot_info(state: &mut LuaState, info: &ReagentSlotInfo) -> Val {
    let table = rooted_table(state);
    number(state, table, "mcrSlotID", info.mcr_slot_id);
    number(state, table, "requiredSkillRank", info.required_skill_rank);
    if let Some(value) = &info.slot_text {
        text(state, table, "slotText", value);
    }
    table
}
fn slot_schematic(state: &mut LuaState, slot: &ReagentSlotSchematic) -> Val {
    let table = rooted_table(state);
    let reagents = rooted_table(state);
    for (index, reagent) in slot.reagents.iter().enumerate() {
        let value = identity(state, *reagent);
        array_entry(state, reagents, index, value);
    }
    table_set(state, table, "reagents", reagents);
    let variable = regular_array(state, &slot.variable_quantities);
    table_set(state, table, "variableQuantities", variable);
    populate_slot_fields(state, table, slot);
    table
}
fn populate_slot_fields(state: &mut LuaState, table: Val, slot: &ReagentSlotSchematic) {
    for (key, value) in [
        ("reagentType", slot.reagent_type),
        ("quantityRequired", slot.quantity_required),
        ("dataSlotType", slot.data_slot_type),
        ("dataSlotIndex", slot.data_slot_index),
        ("slotIndex", slot.slot_index),
    ] {
        number(state, table, key, value);
    }
    table_set(state, table, "required", Val::Bool(slot.required));
    table_set(
        state,
        table,
        "hiddenInCraftingForm",
        Val::Bool(slot.hidden_in_crafting_form),
    );
    if let Some(source) = slot.order_source {
        number(state, table, "orderSource", source);
    }
    if let Some(info) = &slot.slot_info {
        let value = slot_info(state, info);
        table_set(state, table, "slotInfo", value);
    }
}
pub(crate) fn schematic_slots(state: &mut LuaState, slots: &[ReagentSlotSchematic]) -> Val {
    let table = rooted_table(state);
    for (index, slot) in slots.iter().enumerate() {
        let value = slot_schematic(state, slot);
        array_entry(state, table, index, value);
    }
    table
}
pub(crate) fn order(state: &mut LuaState, order: &CraftingOrder) -> Val {
    let table = rooted_table(state);
    populate_order_fields(state, table, order);
    let reagents = rooted_table(state);
    for (index, info) in order.reagents.iter().enumerate() {
        let value = rooted_table(state);
        let nested = allocation(state, &info.reagent_info);
        table_set(state, value, "reagentInfo", nested);
        number(state, value, "slotIndex", info.slot_index);
        number(state, value, "source", info.source);
        table_set(
            state,
            value,
            "isBasicReagent",
            Val::Bool(info.is_basic_reagent),
        );
        array_entry(state, reagents, index, value);
    }
    table_set(state, table, "reagents", reagents);
    let rewards = rooted_table(state);
    table_set(state, table, "npcOrderRewards", rewards);
    table
}
fn populate_order_fields(state: &mut LuaState, table: Val, order: &CraftingOrder) {
    let request = &order.request;
    // INFERRED: container lifecycle/economic fields below are local zero sentinels,
    // not server order state. Only the nested reagent contracts are proven here.
    text(state, table, "orderID", &order.order_id.to_string());
    for (key, value) in [
        ("spellID", order.recipe_id),
        ("skillLineAbilityID", request.skill_line_ability_id),
        ("orderType", request.order_type),
        ("orderState", 0),
        ("expirationTime", 0),
        ("claimEndTime", 0),
        ("minQuality", request.min_crafting_quality_id.unwrap_or(0)),
        ("consortiumCut", 0),
        ("reagentState", 0),
        ("npcCraftingOrderSetID", 0),
        ("npcTreasureID", 0),
    ] {
        number(state, table, key, value);
    }
    let item_id = crate::lua_api::globals::profession_data::get_recipe(order.recipe_id)
        .map_or(0, |recipe| recipe.output_item_id);
    table_set(state, table, "itemID", Val::Num(f64::from(item_id)));
    table_set(state, table, "tipAmount", Val::Num(request.tip_amount));
    table_set(
        state,
        table,
        "isRecraft",
        Val::Bool(request.recraft_item.is_some()),
    );
    table_set(state, table, "isFulfillable", Val::Bool(true));
    text(state, table, "customerNotes", &request.customer_notes);
    if let Some(name) = &request.order_target {
        text(state, table, "crafterName", name);
    }
}
pub(crate) fn orders(state: &mut LuaState, orders: &[CraftingOrder]) -> Val {
    let table = rooted_table(state);
    for (index, record) in orders.iter().enumerate() {
        let value = order(state, record);
        array_entry(state, table, index, value);
    }
    table
}

pub(crate) fn read_field(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Val> {
    let Val::Table(reference) = table else {
        return Err(rilua::runtime_error("crafting structure must be a table"));
    };
    rilua::table_security::check_table_access(state, reference, None)?;
    let value = table_get(state, table, key);
    rilua::table_security::unwrap_secret(state, value)
}
