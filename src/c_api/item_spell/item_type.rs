//! Retail equipped-item type matching over supplied equipment and generated DB2 names.

use crate::lua_api::methods::{borrow_state, table_get, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use sim_data::item_categories::{CLASSES, ITEMS, MASKS, SUBCLASSES};

// WM29: ordered native tokens; a localized value at an earlier index wins.
const TOKENS: [&str; 35] = [
    "INVTYPE_NON_EQUIP_IGNORE",
    "INVTYPE_HEAD",
    "INVTYPE_NECK",
    "INVTYPE_SHOULDER",
    "INVTYPE_BODY",
    "INVTYPE_CHEST",
    "INVTYPE_WAIST",
    "INVTYPE_LEGS",
    "INVTYPE_FEET",
    "INVTYPE_WRIST",
    "INVTYPE_HAND",
    "INVTYPE_FINGER",
    "INVTYPE_TRINKET",
    "INVTYPE_WEAPON",
    "INVTYPE_SHIELD",
    "INVTYPE_RANGED",
    "INVTYPE_CLOAK",
    "INVTYPE_2HWEAPON",
    "INVTYPE_BAG",
    "INVTYPE_TABARD",
    "INVTYPE_ROBE",
    "INVTYPE_WEAPONMAINHAND",
    "INVTYPE_WEAPONOFFHAND",
    "INVTYPE_HOLDABLE",
    "INVTYPE_AMMO",
    "INVTYPE_THROWN",
    "INVTYPE_RANGEDRIGHT",
    "INVTYPE_QUIVER",
    "INVTYPE_RELIC",
    "INVTYPE_PROFESSION_TOOL",
    "INVTYPE_PROFESSION_GEAR",
    "INDEX_EQUIPABLESPELL_OFFENSIVE_TYPE",
    "INDEX_EQUIPABLESPELL_UTILITY_TYPE",
    "INDEX_EQUIPABLESPELL_DEFENSIVE_TYPE",
    "INDEX_EQUIPABLESPELL_WEAPON_TYPE",
];

fn equal_name(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

fn find_inventory_type(state: &mut LuaState, query: &str) -> Option<u8> {
    for (index, token) in TOKENS.iter().enumerate() {
        if equal_name(query, token) {
            return Some(index as u8);
        }
        let value = table_get(state, Val::Table(state.global), token);
        if val_to_string(state, value).is_some_and(|localized| equal_name(query, &localized)) {
            return Some(index as u8);
        }
    }
    None
}

fn matches_category(query: &str, class: u8, subclass: u8) -> bool {
    CLASSES
        .iter()
        .any(|(id, name)| *id == class && equal_name(query, name))
        || SUBCLASSES.iter().any(|(c, s, name, verbose)| {
            *c == class && *s == subclass && (equal_name(query, name) || equal_name(query, verbose))
        })
        || MASKS.iter().any(|(c, mask, name)| {
            *c == class
                && 1u32
                    .checked_shl(u32::from(subclass))
                    .is_some_and(|bit| mask & bit != 0)
                && equal_name(query, name)
        })
}

pub(super) fn is_equipped_item_type(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let query = match value {
        Val::Str(_) => {
            val_to_string(state, value).ok_or_else(|| runtime_error("invalid item type string"))?
        }
        Val::Num(number) => number.to_string(),
        _ => return Err(runtime_error("C_Item.IsEquippedItemType: expected string")),
    };
    let inventory_type = find_inventory_type(state, &query).filter(|index| *index > 0);
    let found = {
        let sim = borrow_state(state)?;
        sim.player.present
            && sim.player.equipped_items.iter().any(|(slot, item)| {
                if !(0..=18).contains(slot) {
                    return false;
                }
                let Ok(index) = ITEMS.binary_search_by_key(&item.item_id, |entry| entry.0) else {
                    return false;
                };
                let (_, class, subclass, inventory) = ITEMS[index];
                match inventory_type {
                    Some(expected) => inventory == expected,
                    None => matches_category(&query, class, subclass),
                }
            })
    };
    state.push(Val::Bool(found));
    Ok(1)
}
