//! Retail/PTR/Forever `C_Container` item queries over the bag and equipment
//! models: ContainerDocumentation.lua `PlayerHasHearthstone`,
//! `GetContainerItemEquipmentSetInfo` and `SetBagPortraitTexture` (identical
//! in Retail 12.1.5 and Forever 1.60.1).

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Item 6948, Hearthstone. INFERRED R88g9 (native evidence pending): only this
/// carried item answers; toy hearthstones are not bag items.
const HEARTHSTONE_ITEM_ID: u32 = 6948;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Container")?;
    table_set_rust_fn_static(state, namespace, "PlayerHasHearthstone", has_hearthstone)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetContainerItemEquipmentSetInfo",
        equipment_set_info,
    )?;
    table_set_rust_fn_static(state, namespace, "SetBagPortraitTexture", bag_portrait)
}

/// `itemID` of a carried Hearthstone, or nil.
fn has_hearthstone(state: &mut LuaState) -> LuaResult<u32> {
    let carried = borrow_state(state)?
        .bag_items
        .values()
        .any(|item| item.item_id == HEARTHSTONE_ITEM_ID);
    state.push(if carried {
        Val::Num(f64::from(HEARTHSTONE_ITEM_ID))
    } else {
        Val::Nil
    });
    Ok(1)
}

/// Names of the saved equipment sets that include the item ID in the slot.
/// INFERRED R88g9 (native evidence pending): sets match by item ID, in set
/// order, joined with ", "; an empty slot answers `false, ""`.
fn set_names_for_slot(sim: &SimState, bag: i32, slot: i32) -> Vec<String> {
    let Some(item) = sim.bag_items.get(&(bag, slot)) else {
        return Vec::new();
    };
    sim.equipment_manager
        .sets
        .iter()
        .filter(|set| set.item_ids.values().any(|id| *id == item.item_id))
        .map(|set| set.name.clone())
        .collect()
}

fn equipment_set_info(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let names = set_names_for_slot(&*borrow_state(state)?, bag, slot);
    state.push(Val::Bool(!names.is_empty()));
    let list = create_string(state, &names.join(", "));
    state.push(list);
    Ok(2)
}

/// Icon of the bag item in the bag's equipment slot, else the container's
/// own item record. INFERRED R88g9 (native evidence pending): a bag without
/// an item clears the texture.
fn bag_icon(sim: &SimState, bag: i32) -> Val {
    let Some(info) = sim.bag_info.get(&bag) else {
        return Val::Nil;
    };
    info.inventory_slot
        .and_then(|slot| sim.player.equipped_items.get(&slot))
        .map(|item| item.item_id)
        .or(info.item_id)
        .and_then(crate::items::get_item)
        .map_or(Val::Nil, |item| Val::Num(f64::from(item.icon_file_data_id)))
}

fn bag_portrait(state: &mut LuaState) -> LuaResult<u32> {
    let texture = stack_val(state, 1);
    if !matches!(texture, Val::Table(_)) {
        return Err(runtime_error(
            "C_Container.SetBagPortraitTexture: expected a texture",
        ));
    }
    let bag = i32::from_stack(state, 2)?;
    let icon = bag_icon(&*borrow_state(state)?, bag);
    let key = state.gc.intern_string(b"SetTexture");
    let setter = state.gettable(texture, Val::Str(key))?;
    crate::lua_api::methods::call_function_state(state, setter, &[texture, icon])?;
    Ok(0)
}
