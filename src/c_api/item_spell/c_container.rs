use crate::c_api::ensure_namespace;
use crate::c_api::item_spell::{helpers::global_table, item_link_for_id};
use crate::items;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num, table_set_static,
};
use crate::lua_api::state::BagItem;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(crate) fn register_c_item_upgrade(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_ItemUpgrade")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetItemUpgradeFromLocation",
        c_item_upgrade_set_location,
    )?;
    table_set_rust_fn_static(state, table_ref, "ClearItemUpgrade", c_item_upgrade_clear)?;
    Ok(())
}

fn c_item_upgrade_set_location(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    let storage = global_table(state, "__item_upgrade_state");
    table_set_static(state, storage, "location", location);
    Ok(0)
}

fn c_item_upgrade_clear(state: &mut LuaState) -> LuaResult<u32> {
    let storage = global_table(state, "__item_upgrade_state");
    table_set_static(state, storage, "location", Val::Nil);
    Ok(0)
}

pub(crate) fn register_c_container(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Container")?;
    register_container_query_methods(state, table_ref)?;
    crate::c_api::container_inventory::register(state)
}

type ContainerScriptFn = fn(&mut LuaState) -> LuaResult<u32>;
type ContainerMethod = (&'static str, ContainerScriptFn);

const BACKPACK_BAG_INDEX: i32 = 0;
const BAG_SLOT_FLAG_DISABLE_AUTO_SORT: i32 = 1;
const BAG_SLOT_FLAG_EXCLUDE_JUNK_SELL: i32 = 64;

fn register_container_query_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_container_methods(
        state,
        table_ref,
        &[
            ("GetContainerNumSlots", c_container_get_num_slots),
            ("GetContainerNumFreeSlots", c_container_get_num_free_slots),
            (
                "CalculateTotalNumberOfFreeBagSlots",
                c_container_calculate_total_number_of_free_bag_slots,
            ),
            ("GetContainerFreeSlots", c_container_get_free_slots),
            ("HasContainerItem", c_container_has_item),
            ("GetBagSlotFlag", c_container_get_bag_slot_flag),
            (
                "GetBackpackAutosortDisabled",
                c_container_get_backpack_autosort_disabled,
            ),
            (
                "GetBackpackSellJunkDisabled",
                c_container_get_backpack_sell_junk_disabled,
            ),
            ("GetContainerItemInfo", c_container_get_item_info),
            ("GetContainerItemCooldown", c_container_get_item_cooldown),
            ("GetItemCooldown", c_container_get_item_cooldown),
            ("GetContainerItemID", c_container_get_item_id),
            ("GetContainerItemLink", c_container_get_item_link),
            ("ContainerIDToInventoryID", c_container_id_to_inventory_id),
            ("GetBagName", c_container_get_bag_name),
            ("SetBagSlotFlag", c_container_set_bag_slot_flag),
        ],
    )
}

fn container_slot_count(bag: i32) -> i32 {
    match bag {
        -4 => 7,
        -1 => 28,
        0..=4 => 16,
        _ => 0,
    }
}

fn register_container_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    entries: &[ContainerMethod],
) -> LuaResult<()> {
    for &(name, func) in entries {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

pub(crate) fn c_container_get_num_slots(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    state.push(Val::Num(container_slot_count(bag) as f64));
    Ok(1)
}

fn c_container_get_num_free_slots(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let occupied = borrow_state(state)?.bag_occupied_slots(bag) as f64;
    let free = (container_slot_count(bag) as f64 - occupied).max(0.0);
    state.push(Val::Num(free));
    state.push(Val::Num(0.0));
    Ok(2)
}

fn c_container_calculate_total_number_of_free_bag_slots(state: &mut LuaState) -> LuaResult<u32> {
    let free_slots = {
        let sim = borrow_state(state)?;
        (0..=4)
            .map(|bag| {
                let occupied = sim.bag_occupied_slots(bag);
                (container_slot_count(bag) - occupied).max(0)
            })
            .sum::<i32>()
    };

    state.push(Val::Num(free_slots as f64));
    Ok(1)
}

fn c_container_get_free_slots(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let free_slots = {
        let sim = borrow_state(state)?;
        (1..=container_slot_count(bag))
            .filter(|slot| sim.get_bag_item(bag, *slot).is_none())
            .collect::<Vec<_>>()
    };

    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for (index, slot) in free_slots.iter().enumerate() {
            table_set_num(state, table_ref, (index + 1) as f64, Val::Num(*slot as f64));
        }
    }
    state.push(table);
    Ok(1)
}

fn c_container_has_item(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let has_item = borrow_state(state)?.get_bag_item(bag, slot).is_some();
    state.push(Val::Bool(has_item));
    Ok(1)
}

fn c_container_get_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let Some(bag_item) = container_bag_item(state, bag, slot)? else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let info = build_container_item_info(state, &bag_item);
    state.push(info);
    Ok(1)
}

fn container_bag_item(state: &LuaState, bag: i32, slot: i32) -> LuaResult<Option<BagItem>> {
    let bag_item = borrow_state(state)?.bag_items.get(&(bag, slot)).cloned();
    Ok(bag_item)
}

fn build_container_item_info(state: &mut LuaState, bag_item: &BagItem) -> Val {
    let item_id = bag_item.item_id;
    let item = items::get_item(item_id);
    let info = create_table(state);
    table_set_static(state, info, "itemID", Val::Num(item_id as f64));
    let icon_file_id = item
        .map(|item| item.icon_file_data_id)
        .filter(|icon| *icon != 0)
        .unwrap_or(134400);
    let quality = item.map(|item| item.quality).unwrap_or(0);
    set_container_item_stack_and_quality(state, info, bag_item, icon_file_id, quality);
    table_set_static(state, info, "isFiltered", Val::Bool(false));
    table_set_static(state, info, "isLocked", Val::Bool(false));
    table_set_static(state, info, "isBound", Val::Bool(false));
    table_set_static(state, info, "hasNoValue", Val::Bool(false));
    table_set_static(state, info, "isReadable", Val::Bool(false));
    table_set_static(state, info, "IsReadable", Val::Bool(false));
    set_container_item_hyperlink(state, info, bag_item);
    info
}

fn set_container_item_stack_and_quality(
    state: &mut LuaState,
    info: Val,
    bag_item: &BagItem,
    icon_file_id: u32,
    quality: u8,
) {
    table_set_static(
        state,
        info,
        "stackCount",
        Val::Num(bag_item.stack_count as f64),
    );
    table_set_static(state, info, "iconFileID", Val::Num(icon_file_id as f64));
    table_set_static(state, info, "quality", Val::Num(quality as f64));
}

fn set_container_item_hyperlink(state: &mut LuaState, info: Val, bag_item: &BagItem) {
    let hyperlink = bag_item
        .hyperlink
        .clone()
        .or_else(|| item_link_for_id(bag_item.item_id));

    match hyperlink {
        Some(link) => {
            let hyperlink = create_string(state, &link);
            table_set_static(state, info, "hyperlink", hyperlink);
        }
        None => table_set_static(state, info, "hyperlink", Val::Nil),
    }
}

fn c_container_get_item_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let _item_or_bag_id = i32::from_stack(state, 1)?;
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(1.0));
    Ok(3)
}

pub(crate) fn c_container_get_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = borrow_state(state)?
        .get_bag_item(bag, slot)
        .map(|(item_id, _)| item_id);
    match item_id {
        Some(item_id) => state.push(Val::Num(item_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(crate) fn c_container_get_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let link = borrow_state(state)?
        .get_bag_item(bag, slot)
        .and_then(|(item_id, _)| item_link_for_id(item_id));
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_container_id_to_inventory_id(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    state.push(Val::Num((20 + bag).max(0) as f64));
    Ok(1)
}

fn c_container_get_bag_name(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    if bag == 0 {
        let name = create_string(state, "Backpack");
        state.push(name);
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}

fn bag_slot_flags_storage(state: &mut LuaState) -> Val {
    global_table(state, "__bag_slot_flags")
}

fn bag_slot_flag_key(bag: i32, flag: i32) -> String {
    format!("{bag}:{flag}")
}

fn bag_slot_flag_is_set(state: &mut LuaState, bag: i32, flag: i32) -> bool {
    let storage = bag_slot_flags_storage(state);
    let key = bag_slot_flag_key(bag, flag);
    let value = crate::lua_api::methods::table_get(state, storage, &key);
    matches!(value, Val::Bool(true))
}

fn c_container_get_bag_slot_flag(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let flag = i32::from_stack(state, 2)?;
    let value = bag_slot_flag_is_set(state, bag, flag);
    state.push(Val::Bool(value));
    Ok(1)
}

fn c_container_get_backpack_autosort_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = bag_slot_flag_is_set(state, BACKPACK_BAG_INDEX, BAG_SLOT_FLAG_DISABLE_AUTO_SORT);
    state.push(Val::Bool(value));
    Ok(1)
}

fn c_container_get_backpack_sell_junk_disabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = bag_slot_flag_is_set(state, BACKPACK_BAG_INDEX, BAG_SLOT_FLAG_EXCLUDE_JUNK_SELL);
    state.push(Val::Bool(value));
    Ok(1)
}

fn c_container_set_bag_slot_flag(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let flag = i32::from_stack(state, 2)?;
    let value = matches!(stack_val(state, 3), Val::Bool(true));
    let storage = bag_slot_flags_storage(state);
    let key = bag_slot_flag_key(bag, flag);
    table_set(state, storage, &key, Val::Bool(value));
    Ok(0)
}
