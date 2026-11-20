//! Item/order recraft slots from host crafting state.
use super::{array_entry, identity, number, rooted_table};
use crate::c_api::crafting_reagents::ItemSlotModification;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // VERIFIED shape: Forever 70707 TradeSkillUIDocumentation.lua:394,409;
    // INFERRED: the host state producer matches Retail, not native server state.
    if !(cfg!(feature = "retail-12-0-0") || ACTIVE == ClientProfile::WowForever) {
        return Ok(());
    }
    let trade = crate::c_api::ensure_namespace(state, "C_TradeSkillUI")?;
    table_set_rust_fn_static(state, trade, "GetItemSlotModifications", item_modifications)?;
    table_set_rust_fn_static(
        state,
        trade,
        "GetItemSlotModificationsForOrder",
        order_modifications,
    )
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
    let table = modifications(state, &mods);
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
    let table = modifications(state, &mods);
    state.push(table);
    Ok(1)
}

fn modifications(state: &mut LuaState, mods: &[ItemSlotModification]) -> Val {
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
