use super::helpers::{
    inv_type_to_class_id, inv_type_to_equip_loc, inv_type_to_subclass, item_class_from_inv_type,
    item_class_name, item_subclass_name, unit_is_reachable,
};
use crate::c_api::ensure_namespace;
use crate::items;
use crate::lua_api::SimState;
use crate::lua_api::methods::{borrow_state, create_string, table_get, val_to_string};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaError, LuaResult, RuntimeError, Val};
use std::collections::HashSet;

type CItemMethod = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

/// Parse the id from a hyperlink (`|Hitem:123:...|h`) or a bare link string
/// (`item:123:...`); WoW APIs accept both forms.
pub(crate) fn parse_prefixed_id(value: &str, prefix: &str) -> Option<u32> {
    let bare = format!("{prefix}:");
    let hyperlink = format!("|H{bare}");
    let fields = match value.find(&hyperlink) {
        Some(start) => &value[start + hyperlink.len()..],
        None => value.strip_prefix(&bare)?,
    };
    let digits: String = fields
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

pub(crate) fn parse_item_id_from_val(state: &LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(number) if number > 0.0 => Some(number as u32),
        Val::Str(_) => val_to_string(state, value)
            .and_then(|text| parse_prefixed_id(&text, "item").or_else(|| text.parse().ok())),
        _ => None,
    }
}

pub(crate) fn item_link_for_id(item_id: u32) -> Option<String> {
    let item = items::get_item(item_id)?;
    Some(format!(
        "|cnIQ{}:|Hitem:{}::::::::80:70:::::::::|h[{}]|h|r",
        item.quality, item_id, item.name
    ))
}

pub(crate) fn spell_link_for_id(spell_id: u32) -> Option<String> {
    use crate::spells;
    let spell = spells::get_spell(spell_id)?;
    Some(format!(
        "|cff71d5ff|Hspell:{}|h[{}]|h|r",
        spell_id, spell.name
    ))
}

fn item_guid_for_bag_slot(bag: i32, slot: i32, item_id: u32) -> String {
    format!("Item-{bag}-{slot}-{item_id}")
}

pub(crate) fn parse_item_guid(guid: &str) -> Option<(i32, i32, u32)> {
    let mut parts = guid.split('-');
    if parts.next()? != "Item" {
        return None;
    }
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

pub(super) fn register_c_item(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Item")?;
    register_c_item_existence_queries(state, table_ref)?;
    register_c_item_metadata_queries(state, table_ref)?;
    register_c_item_inventory_queries(state, table_ref)?;
    Ok(())
}

fn register_c_item_existence_queries(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_c_item_methods(
        state,
        table_ref,
        &[
            ("DoesItemExist", c_item_does_item_exist),
            ("DoesItemExistByID", c_item_does_item_exist_by_id),
            #[cfg(feature = "retail-12-1-0")]
            (
                "DoesItemMatchSpellItemCondition",
                c_item_does_item_match_spell_item_condition,
            ),
            ("GetItemID", c_item_get_item_id),
            ("IsItemDataCached", c_item_is_item_data_cached),
            ("IsItemDataCachedByID", c_item_is_item_data_cached_by_id),
            ("GetItemIDForItemInfo", c_item_get_item_id_for_item_info),
        ],
    )
}

fn register_c_item_metadata_queries(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_c_item_methods(
        state,
        table_ref,
        &[
            ("GetItemIcon", c_item_get_item_icon),
            ("GetItemName", c_item_get_item_name),
            ("GetItemIconByID", c_item_get_item_icon_by_id),
            ("GetItemNameByID", c_item_get_item_name_by_id),
            ("GetItemQualityByID", c_item_get_item_quality_by_id),
            ("GetItemQuality", c_item_get_item_quality),
            (
                "GetItemMaxStackSizeByID",
                c_item_get_item_max_stack_size_by_id,
            ),
            ("GetItemMaxStackSize", c_item_get_item_max_stack_size),
            ("GetItemInfoInstant", c_item_get_item_info_instant),
            ("GetItemInfo", c_item_get_item_info),
            (
                "GetDetailedItemLevelInfo",
                c_item_get_detailed_item_level_info,
            ),
            ("GetCurrentItemLevel", c_item_get_current_item_level),
            ("GetItemSubClassInfo", c_item_get_item_sub_class_info),
            ("GetItemClassInfo", c_item_get_item_class_info),
        ],
    )
}

fn register_c_item_inventory_queries(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_c_item_methods(
        state,
        table_ref,
        &[
            ("GetItemCount", c_item_get_item_count),
            ("GetStackCount", c_item_get_stack_count),
            ("GetItemLink", c_item_get_item_link),
            ("GetItemCooldown", c_item_get_item_cooldown),
            ("GetItemGUID", c_item_get_item_guid),
            ("IsBound", c_item_is_bound),
            ("IsBoundToAccountUntilEquip", c_item_is_bound),
            ("IsConsumableItem", c_item_is_consumable_item),
            ("IsEquippableItem", c_item_is_equippable_item),
            ("IsItemInRange", c_item_is_item_in_range),
            (
                "GetItemInventorySlotInfo",
                c_item_get_item_inventory_slot_info,
            ),
        ],
    )
}

fn register_c_item_methods(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
    methods: &[CItemMethod],
) -> LuaResult<()> {
    for &(name, func) in methods {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

pub(crate) fn c_item_is_equippable_item(state: &mut LuaState) -> LuaResult<u32> {
    push_item_classification(state, |sim| &sim.equippable_items)
}

pub(crate) fn c_item_is_consumable_item(state: &mut LuaState) -> LuaResult<u32> {
    push_item_classification(state, |sim| &sim.consumable_items)
}

fn push_item_classification(
    state: &mut LuaState,
    item_ids: fn(&SimState) -> &HashSet<u32>,
) -> LuaResult<u32> {
    let item_id = match stack_val(state, 1) {
        Val::Num(number) if number >= 0.0 => number as u32,
        _ => {
            state.push(Val::Bool(false));
            return Ok(1);
        }
    };
    let classified = {
        let sim = borrow_state(state)?;
        item_ids(&sim).contains(&item_id)
    };
    state.push(Val::Bool(classified));
    Ok(1)
}

pub(crate) fn c_item_is_item_in_range(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let in_range = {
        let sim = borrow_state(state)?;
        unit_is_reachable(&sim, &unit)
    };
    state.push(Val::Bool(in_range));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn c_item_does_item_match_spell_item_condition(state: &mut LuaState) -> LuaResult<u32> {
    // Spell-item condition metadata is not modeled; default to no match.
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_item_does_item_exist(state: &mut LuaState) -> LuaResult<u32> {
    let exists = item_may_exist(state, stack_val(state, 1))?;
    state.push(Val::Bool(exists));
    Ok(1)
}

fn c_item_does_item_exist_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let exists = parse_item_id_from_val(state, stack_val(state, 1))
        .map(item_id_may_exist)
        .unwrap_or(false);
    state.push(Val::Bool(exists));
    Ok(1)
}

pub(crate) fn location_item(state: &mut LuaState, location: Val) -> Option<(u32, Option<String>)> {
    if let Val::Num(slot) = table_get(state, location, "equipmentSlotIndex") {
        let sim = borrow_state(state).ok()?;
        return resolve_equipped_location_item(&sim, slot as i32);
    }
    let bag = match table_get(state, location, "bagID") {
        Val::Num(number) => number as i32,
        _ => 0,
    };
    let slot = match table_get(state, location, "slotIndex") {
        Val::Num(number) => number as i32,
        _ => 0,
    };
    borrow_state(state)
        .ok()?
        .bag_items
        .get(&(bag, slot))
        .map(|item| (item.item_id, item.hyperlink.clone()))
}

fn resolve_equipped_location_item(sim: &SimState, slot: i32) -> Option<(u32, Option<String>)> {
    let bag = sim
        .bag_info
        .values()
        .find(|bag| bag.inventory_slot == Some(slot) && bag.item_id.is_some());
    sim.player
        .equipped_items
        .get(&slot)
        .map(|item| {
            let link = bag
                .filter(|bag| bag.item_id == Some(item.item_id))
                .and_then(|bag| bag.hyperlink.clone());
            (item.item_id, link)
        })
        .or_else(|| bag.and_then(|bag| bag.item_id.map(|id| (id, bag.hyperlink.clone()))))
}

fn item_id_from_location_or_item_info(state: &mut LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Table(_) => location_item(state, value).map(|(item_id, _)| item_id),
        _ => parse_item_id_from_val(state, value),
    }
}

pub(crate) fn c_item_get_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    match item_id_from_location_or_item_info(state, value) {
        Some(item_id) => state.push(Val::Num(item_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_get_item_icon(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    match item_id_from_location_or_item_info(state, value) {
        Some(item_id) => {
            let icon = items::get_item(item_id)
                .map(|item| {
                    if item.icon_file_data_id == 0 {
                        134400.0
                    } else {
                        item.icon_file_data_id as f64
                    }
                })
                .unwrap_or(134400.0);
            state.push(Val::Num(icon));
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_get_item_name(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    match item_id_from_location_or_item_info(state, value) {
        Some(item_id) => {
            let name = items::get_item(item_id)
                .map(|item| item.name)
                .unwrap_or("Unknown");
            let name = create_string(state, name);
            state.push(name);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_is_item_data_cached(state: &mut LuaState) -> LuaResult<u32> {
    let cached = item_may_exist(state, stack_val(state, 1))?;
    state.push(Val::Bool(cached));
    Ok(1)
}

fn item_may_exist(state: &mut LuaState, value: Val) -> LuaResult<bool> {
    Ok(match value {
        Val::Table(_) => location_item(state, value).is_some(),
        _ => parse_item_id_from_val(state, value)
            .map(item_id_may_exist)
            .unwrap_or(false),
    })
}

fn item_id_may_exist(item_id: u32) -> bool {
    item_id > 0
}

fn c_item_is_item_data_cached_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let cached = parse_item_id_from_val(state, stack_val(state, 1))
        .map(item_id_has_synthetic_data)
        .unwrap_or(false);
    state.push(Val::Bool(cached));
    Ok(1)
}

fn item_id_has_synthetic_data(item_id: u32) -> bool {
    item_id_may_exist(item_id)
}

fn c_item_get_item_icon_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    let icon = items::get_item(item_id)
        .map(|item| {
            if item.icon_file_data_id == 0 {
                134400.0
            } else {
                item.icon_file_data_id as f64
            }
        })
        .unwrap_or(134400.0);
    state.push(Val::Num(icon));
    Ok(1)
}

fn c_item_get_item_name_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    let name = items::get_item(item_id)
        .map(|item| item.name)
        .unwrap_or("Unknown");
    let name = create_string(state, name);
    state.push(name);
    Ok(1)
}

fn c_item_get_item_quality_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    state.push(Val::Num(item_quality_for_id(item_id)));
    Ok(1)
}

fn c_item_get_item_quality(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = item_id_from_location_or_item_info(state, stack_val(state, 1)).unwrap_or(0);
    state.push(Val::Num(item_quality_for_id(item_id)));
    Ok(1)
}

fn item_quality_for_id(item_id: u32) -> f64 {
    items::get_item(item_id)
        .map(|item| item.quality as f64)
        .unwrap_or_else(|| if item_id == 1 { 1.0 } else { 0.0 })
}

fn c_item_get_item_max_stack_size_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    state.push(Val::Num(item_max_stack_size_for_id(item_id)));
    Ok(1)
}

fn c_item_get_item_max_stack_size(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item_id) = item_id_from_location_or_item_info(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    state.push(Val::Num(item_max_stack_size_for_id(item_id)));
    Ok(1)
}

fn item_max_stack_size_for_id(item_id: u32) -> f64 {
    items::get_item(item_id)
        .map(|item| item.stackable.max(1) as f64)
        .unwrap_or_else(|| {
            if item_id_has_synthetic_data(item_id) {
                1.0
            } else {
                0.0
            }
        })
}

fn c_item_get_item_info_instant(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item_id) = parse_item_id_from_val(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    let Some(item) = items::get_item(item_id) else {
        let class_name = create_string(state, "Miscellaneous");
        let subclass_name = create_string(state, "Junk");
        let empty = create_string(state, "");
        state.push(Val::Num(item_id as f64));
        state.push(class_name);
        state.push(subclass_name);
        state.push(empty);
        state.push(Val::Num(134400.0));
        state.push(Val::Num(15.0));
        state.push(Val::Num(0.0));
        return Ok(7);
    };
    let class_name = create_string(state, item_class_from_inv_type(item.inventory_type));
    let subclass_name = create_string(state, inv_type_to_subclass(item.inventory_type));
    let empty = create_string(state, "");
    state.push(Val::Num(item_id as f64));
    state.push(class_name);
    state.push(subclass_name);
    state.push(empty);
    state.push(Val::Num(if item.icon_file_data_id == 0 {
        134400.0
    } else {
        item.icon_file_data_id as f64
    }));
    state.push(Val::Num(inv_type_to_class_id(item.inventory_type) as f64));
    state.push(Val::Num(0.0));
    Ok(7)
}

pub(crate) fn push_item_info(state: &mut LuaState, item_id: u32) -> LuaResult<u32> {
    let Some(item) = items::get_item(item_id) else {
        return push_unknown_item_info(state, item_id);
    };
    let item_name = create_string(state, item.name);
    let item_link = create_string(state, &item_link_for_id(item_id).unwrap_or_default());
    let class_name = create_string(state, item_class_from_inv_type(item.inventory_type));
    let subclass_name = create_string(state, inv_type_to_subclass(item.inventory_type));
    let equip_loc = create_string(state, inv_type_to_equip_loc(item.inventory_type));
    state.push(item_name);
    state.push(item_link);
    state.push(Val::Num(item.quality as f64));
    state.push(Val::Num(item.item_level as f64));
    state.push(Val::Num(item.required_level as f64));
    state.push(class_name);
    state.push(subclass_name);
    state.push(Val::Num(item.stackable as f64));
    state.push(equip_loc);
    state.push(Val::Num(if item.icon_file_data_id == 0 {
        134400.0
    } else {
        item.icon_file_data_id as f64
    }));
    state.push(Val::Num(item.sell_price as f64));
    state.push(Val::Num(inv_type_to_class_id(item.inventory_type) as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(item.bonding as f64));
    state.push(Val::Num(item.expansion_id as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    Ok(17)
}

fn push_unknown_item_info(state: &mut LuaState, item_id: u32) -> LuaResult<u32> {
    let item_name = create_string(state, "Unknown");
    let item_link = create_string(
        state,
        &format!("|cnIQ1:|Hitem:{item_id}::::::::80:70:::::::::|h[Unknown]|h|r"),
    );
    let class_name = create_string(state, "Miscellaneous");
    let subclass_name = create_string(state, "Junk");
    let equip_loc = create_string(state, "");
    push_item_info_values(
        state,
        item_name,
        item_link,
        class_name,
        subclass_name,
        equip_loc,
    );
    Ok(17)
}

fn push_item_info_values(
    state: &mut LuaState,
    item_name: Val,
    item_link: Val,
    class_name: Val,
    subclass_name: Val,
    equip_loc: Val,
) {
    state.push(item_name);
    state.push(item_link);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(class_name);
    state.push(subclass_name);
    state.push(Val::Num(1.0));
    state.push(equip_loc);
    state.push(Val::Num(134400.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(15.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
}

fn c_item_get_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item_id) = parse_item_id_from_val(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    push_item_info(state, item_id)
}

fn c_item_get_detailed_item_level_info(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    let level = item_level_for_id(item_id);
    state.push(Val::Num(level));
    state.push(Val::Num(0.0));
    state.push(Val::Num(level));
    Ok(3)
}

fn c_item_get_current_item_level(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = item_id_from_location_or_item_info(state, stack_val(state, 1)).unwrap_or(0);
    state.push(Val::Num(item_level_for_id(item_id)));
    Ok(1)
}

fn item_level_for_id(item_id: u32) -> f64 {
    items::get_item(item_id)
        .map(|item| item.item_level as f64)
        .unwrap_or(0.0)
}

fn c_item_get_item_sub_class_info(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = i32::from_stack(state, 1)?;
    let subclass_id = i32::from_stack(state, 2)?;
    let name = item_subclass_name(class_id, subclass_id);
    if name == "Unknown" {
        state.push(Val::Nil);
        return Ok(1);
    }

    let name = create_string(state, name);
    state.push(name);
    state.push(Val::Bool(class_id == 2 || class_id == 4));
    Ok(2)
}

fn c_item_get_item_class_info(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = i32::from_stack(state, 1)?;
    let name = create_string(state, item_class_name(class_id));
    state.push(name);
    Ok(1)
}

fn c_item_get_item_id_for_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1));
    match item_id {
        Some(item_id) => state.push(Val::Num(item_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_get_item_count(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = parse_item_id_from_val(state, stack_val(state, 1)).unwrap_or(0);
    let count = borrow_state(state)
        .map(|sim| {
            let bag_count = sim
                .bag_items
                .values()
                .filter(|item| item.item_id == item_id)
                .map(|item| item.stack_count)
                .sum::<i32>();
            let equipped_count = sim
                .player
                .equipped_items
                .values()
                .filter(|item| item.item_id == item_id)
                .count() as i32;
            bag_count + equipped_count
        })
        .unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn c_item_get_stack_count(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    if !matches!(value, Val::Table(_)) {
        return Err(LuaError::Runtime(RuntimeError {
            message: format!(
                "C_Item.GetStackCount: expected ItemLocation table at argument 1, got {}",
                value.type_name()
            ),
            level: 1,
            traceback: vec![],
        }));
    }
    let count = match table_get(state, value, "equipmentSlotIndex") {
        Val::Num(slot) => borrow_state(state)?
            .player
            .equipped_items
            .contains_key(&(slot as i32)) as i32,
        _ => bag_stack_count(state, value),
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn bag_stack_count(state: &mut LuaState, location: Val) -> i32 {
    let bag = match table_get(state, location, "bagID") {
        Val::Num(number) => number as i32,
        _ => 0,
    };
    let slot = match table_get(state, location, "slotIndex") {
        Val::Num(number) => number as i32,
        _ => 0,
    };
    borrow_state(state)
        .ok()
        .and_then(|sim| sim.get_bag_item(bag, slot).map(|(_, count)| count))
        .unwrap_or(0)
}

fn c_item_is_bound(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_item_get_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let item = match value {
        Val::Table(_) => location_item(state, value),
        _ => parse_item_id_from_val(state, value).map(|item_id| (item_id, None)),
    };
    let link = item.and_then(|(item_id, captured)| captured.or_else(|| item_link_for_id(item_id)));
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_get_item_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let _item_info = stack_val(state, 1);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(true));
    Ok(3)
}

fn c_item_get_item_guid(state: &mut LuaState) -> LuaResult<u32> {
    let location = stack_val(state, 1);
    let bag = match table_get(state, location, "bagID") {
        Val::Num(value) => value as i32,
        _ => 0,
    };
    let slot = match table_get(state, location, "slotIndex") {
        Val::Num(value) => value as i32,
        _ => 0,
    };
    let item_id = borrow_state(state)?
        .get_bag_item(bag, slot)
        .map(|(item_id, _)| item_id);
    match item_id {
        Some(item_id) => {
            let guid = create_string(state, &item_guid_for_bag_slot(bag, slot, item_id));
            state.push(guid);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_item_get_item_inventory_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let inv_type = i32::from_stack(state, 1)?;
    let label = create_string(state, inv_type_to_subclass(inv_type.max(0) as u8));
    state.push(label);
    Ok(1)
}
