//! Validated carried-bag snapshots. Missing data is not an empty inventory.

use crate::c_api::bag_info::BagInfo;
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{table_get, val_to_string};
use crate::lua_api::state::{BagItem, EquippedItem};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, Val};
use std::collections::{BTreeSet, HashMap};

const LAST_CARRIED_BAG: i32 = 5;
const FIRST_BAG_INVENTORY_SLOT: i64 = 20;

pub(super) struct BagSnapshot {
    metadata: HashMap<i32, BagInfo>,
    items: HashMap<(i32, i32), BagItem>,
}

pub(super) fn read(env: &WowLuaEnv) -> crate::Result<Option<BagSnapshot>> {
    let value: Val = env.eval(&super::snapshot_chunk(
        "return type(snapshot) == 'table' and snapshot.bags or nil",
    ))?;
    if value.is_nil() {
        return Ok(None);
    }
    let mut lua = env.rilua_mut();
    parse(lua.state_mut(), value).map(Some)
}

fn invalid(path: &str, message: &str) -> crate::Error {
    crate::Error::Other(format!("ServerSnapshot {path}: {message}"))
}

fn require_table(value: Val, path: &str) -> crate::Result<GcRef<Table>> {
    match value {
        Val::Table(table) => Ok(table),
        _ => Err(invalid(path, "expected a table")),
    }
}

fn integer(value: Val, min: i64, max: i64, path: &str) -> crate::Result<i64> {
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= min as f64
                && number <= max as f64 =>
        {
            Ok(number as i64)
        }
        _ => Err(invalid(path, &format!("expected integer in {min}..={max}"))),
    }
}

fn field_integer(
    state: &mut LuaState,
    row: Val,
    key: &'static str,
    min: i64,
    path: &str,
) -> crate::Result<i32> {
    integer(
        table_get(state, row, key),
        min,
        i32::MAX as i64,
        &format!("{path}.{key}"),
    )
    .map(|value| value as i32)
}

fn optional_text(
    state: &mut LuaState,
    row: Val,
    key: &'static str,
    path: &str,
) -> crate::Result<Option<String>> {
    let value = table_get(state, row, key);
    if value.is_nil() {
        return Ok(None);
    }
    val_to_string(state, value)
        .map(Some)
        .ok_or_else(|| invalid(&format!("{path}.{key}"), "expected string or nil"))
}

fn optional_item_id(state: &mut LuaState, row: Val, path: &str) -> crate::Result<Option<u32>> {
    let value = table_get(state, row, "itemID");
    if value.is_nil() {
        return Ok(None);
    }
    integer(value, 1, u32::MAX as i64, &format!("{path}.itemID")).map(|value| Some(value as u32))
}

fn optional_inventory_slot(
    state: &mut LuaState,
    row: Val,
    path: &str,
) -> crate::Result<Option<i32>> {
    let value = table_get(state, row, "inventorySlot");
    if value.is_nil() {
        return Ok(None);
    }
    integer(
        value,
        FIRST_BAG_INVENTORY_SLOT,
        i32::MAX as i64,
        &format!("{path}.inventorySlot"),
    )
    .map(|value| Some(value as i32))
}

fn entries(
    state: &LuaState,
    reference: GcRef<Table>,
    path: &str,
) -> crate::Result<Vec<(Val, Val)>> {
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| invalid(path, "table is no longer available"))?;
    let mut entries: Vec<_> = table
        .array_slice()
        .iter()
        .enumerate()
        .filter_map(|(index, value)| {
            (!value.is_nil()).then_some((Val::Num((index + 1) as f64), *value))
        })
        .collect();
    entries.extend(table.hash_entries());
    Ok(entries)
}

fn read_containers(state: &mut LuaState, value: Val) -> crate::Result<(i32, GcRef<Table>)> {
    require_table(value, "bags")?;
    let max_bag = integer(
        table_get(state, value, "maxBagID"),
        0,
        LAST_CARRIED_BAG as i64,
        "bags.maxBagID",
    )? as i32;
    let containers = table_get(state, value, "containers");
    let table = require_table(containers, "bags.containers")?;
    for (key, _) in entries(state, table, "bags.containers")? {
        integer(key, 0, max_bag as i64, "bags.containers key")?;
    }
    Ok((max_bag, table))
}

fn parse(state: &mut LuaState, value: Val) -> crate::Result<BagSnapshot> {
    let (max_bag, table) = read_containers(state, value)?;
    let mut snapshot = BagSnapshot {
        metadata: HashMap::new(),
        items: HashMap::new(),
    };
    let mut inventory_slots = BTreeSet::new();
    for bag in 0..=max_bag {
        let row = state
            .gc
            .tables
            .get(table)
            .map(|table| table.get_int(bag as i64))
            .ok_or_else(|| invalid("bags.containers", "table is no longer available"))?;
        let path = format!("bags.containers[{bag}]");
        require_table(row, &path)?;
        let info = parse_metadata(state, row, &path)?;
        validate_inventory_slot(bag, info.inventory_slot, &mut inventory_slots, &path)?;
        let items = parse_items(state, row, info.num_slots, &path)?;
        snapshot
            .items
            .extend(items.into_iter().map(|(slot, item)| ((bag, slot), item)));
        snapshot.metadata.insert(bag, info);
    }
    Ok(snapshot)
}

fn parse_metadata(state: &mut LuaState, row: Val, path: &str) -> crate::Result<BagInfo> {
    Ok(BagInfo {
        num_slots: field_integer(state, row, "numSlots", 0, path)?,
        family: field_integer(state, row, "family", 0, path)?,
        name: optional_text(state, row, "name", path)?,
        inventory_slot: optional_inventory_slot(state, row, path)?,
        item_id: optional_item_id(state, row, path)?,
        hyperlink: optional_text(state, row, "hyperlink", path)?,
    })
}

fn validate_inventory_slot(
    bag: i32,
    slot: Option<i32>,
    seen: &mut BTreeSet<i32>,
    path: &str,
) -> crate::Result<()> {
    if let Some(slot) = slot {
        if bag == 0 {
            return Err(invalid(
                path,
                "backpack cannot claim an equipped inventory slot",
            ));
        }
        if !seen.insert(slot) {
            return Err(invalid(path, "duplicate equipped bag inventory slot"));
        }
    }
    Ok(())
}

fn parse_items(
    state: &mut LuaState,
    row: Val,
    num_slots: i32,
    path: &str,
) -> crate::Result<Vec<(i32, BagItem)>> {
    let value = table_get(state, row, "items");
    let table = require_table(value, &format!("{path}.items"))?;
    entries(state, table, path)?
        .into_iter()
        .map(|(key, row)| {
            let slot = integer(key, 1, num_slots as i64, &format!("{path}.items key"))? as i32;
            let item = parse_item(state, row, &format!("{path}.items[{slot}]"))?;
            Ok((slot, item))
        })
        .collect()
}

fn parse_item(state: &mut LuaState, row: Val, path: &str) -> crate::Result<BagItem> {
    require_table(row, path)?;
    let item_id = integer(
        table_get(state, row, "itemID"),
        1,
        u32::MAX as i64,
        &format!("{path}.itemID"),
    )? as u32;
    Ok(BagItem {
        item_id,
        stack_count: field_integer(state, row, "stackCount", 1, path)?,
        hyperlink: optional_text(state, row, "hyperlink", path)?,
    })
}

pub(super) fn apply(env: &WowLuaEnv, snapshot: BagSnapshot) -> Vec<i32> {
    let container_count = snapshot.metadata.len();
    let item_count = snapshot.items.len();
    let mut sim = env.state().borrow_mut();
    let changed: BTreeSet<_> = sim
        .bag_info
        .keys()
        .chain(snapshot.metadata.keys())
        .copied()
        .filter(|bag| (0..=LAST_CARRIED_BAG).contains(bag))
        .collect();
    clear_carried_bags(&mut sim);
    for (bag, info) in snapshot.metadata {
        apply_equipped_bag(&mut sim, &info);
        sim.bag_info.insert(bag, info);
    }
    sim.bag_items.extend(snapshot.items);
    crate::logging::eprintln_elapsed(&format!(
        "[ServerSnapshot] Imported {container_count} carried containers and {item_count} occupied slots"
    ));
    changed.into_iter().collect()
}

fn clear_carried_bags(sim: &mut crate::lua_api::state::SimState) {
    let old_equipment: Vec<_> = sim
        .bag_info
        .iter()
        .filter(|(bag, _)| (0..=LAST_CARRIED_BAG).contains(*bag))
        .filter_map(|(_, info)| info.inventory_slot)
        .collect();
    for slot in old_equipment {
        sim.player.equipped_items.remove(&slot);
    }
    sim.bag_info
        .retain(|bag, _| !(0..=LAST_CARRIED_BAG).contains(bag));
    sim.bag_items
        .retain(|(bag, _), _| !(0..=LAST_CARRIED_BAG).contains(bag));
}

fn apply_equipped_bag(sim: &mut crate::lua_api::state::SimState, info: &BagInfo) {
    if let Some(slot) = info.inventory_slot {
        sim.player.equipped_items.remove(&slot);
        if let Some(item_id) = info.item_id {
            sim.player.equipped_items.insert(
                slot,
                EquippedItem {
                    item_id,
                    enchant_id: 0,
                    gem_ids: [0; 3],
                },
            );
        }
    }
}

pub(super) fn notify(env: &WowLuaEnv, bags: &[i32]) -> crate::Result<()> {
    for bag in bags {
        let args = [Val::Num(*bag as f64)];
        env.fire_event_with_args("BAG_UPDATE", &args)?;
        env.fire_event_with_args("BAG_CONTAINER_UPDATE", &args)?;
    }
    env.fire_event("BAG_UPDATE_DELAYED")
}
