use crate::lua_api::SimState;
use crate::lua_api::methods::{create_table, table_get};
use rilua::Val;
use rilua::vm::state::LuaState;

const ITEM_CLASS_NAMES: &[(i32, &str)] = &[
    (0, "Consumable"),
    (1, "Container"),
    (2, "Weapon"),
    (3, "Gem"),
    (4, "Armor"),
    (5, "Reagent"),
    (6, "Projectile"),
    (7, "Tradeskill"),
    (8, "Item Enhancement"),
    (9, "Recipe"),
    (10, "Money"),
    (11, "Quiver"),
    (12, "Quest"),
    (13, "Key"),
    (14, "Permanent"),
    (15, "Miscellaneous"),
    (16, "Glyph"),
    (17, "Battle Pets"),
    (18, "WoW Token"),
    (19, "Profession"),
    (20, "Housing"),
];

const ITEM_SUBCLASS_NAMES: &[(i32, i32, &str)] = &[
    (0, 0, "Explosives and Devices"),
    (0, 1, "Potions"),
    (0, 2, "Elixirs"),
    (0, 3, "Flasks & Phials"),
    (0, 5, "Food & Drink"),
    (0, 7, "Bandages"),
    (0, 8, "Other"),
    (0, 9, "Vantus Runes"),
    (0, 10, "Utility Curio"),
    (0, 11, "Combat Curio"),
    (2, 0, "One-Handed Axes"),
    (2, 1, "Two-Handed Axes"),
    (2, 2, "Bows"),
    (2, 3, "Guns"),
    (2, 4, "One-Handed Maces"),
    (2, 5, "Two-Handed Maces"),
    (2, 6, "Polearms"),
    (2, 7, "One-Handed Swords"),
    (2, 8, "Two-Handed Swords"),
    (2, 9, "Warglaives"),
    (2, 10, "Staves"),
    (2, 11, "Bear Claws"),
    (2, 12, "CatClaws"),
    (2, 13, "Fist Weapons"),
    (2, 14, "Generic"),
    (2, 15, "Daggers"),
    (2, 16, "Thrown"),
    (2, 17, "Spears"),
    (2, 18, "Crossbows"),
    (2, 19, "Wands"),
    (2, 20, "Fishing Poles"),
    (4, 1, "Cloth"),
    (4, 2, "Leather"),
    (4, 3, "Mail"),
    (4, 4, "Plate"),
    (4, 6, "Shields"),
    (7, 1, "Parts"),
    (7, 4, "Jewelcrafting"),
    (7, 5, "Cloth"),
    (7, 6, "Leather"),
    (7, 7, "Metal & Stone"),
    (7, 8, "Cooking"),
    (7, 9, "Herb"),
    (7, 10, "Elemental"),
    (7, 12, "Enchanting"),
    (7, 16, "Inscription"),
    (7, 18, "Optional Reagents"),
    (7, 19, "Finishing Reagents"),
    (9, 1, "Leatherworking"),
    (9, 2, "Tailoring"),
    (9, 3, "Engineering"),
    (9, 4, "Blacksmithing"),
    (9, 5, "Cooking"),
    (9, 6, "Alchemy"),
    (9, 7, "First Aid"),
    (9, 8, "Enchanting"),
    (9, 9, "Fishing"),
    (9, 10, "Jewelcrafting"),
    (9, 11, "Inscription"),
    (15, 5, "Mount"),
    (16, 1, "Warrior"),
    (16, 2, "Paladin"),
    (16, 3, "Hunter"),
    (16, 4, "Rogue"),
    (16, 5, "Priest"),
    (16, 6, "Death Knight"),
    (16, 7, "Shaman"),
    (16, 8, "Mage"),
    (16, 9, "Warlock"),
    (16, 10, "Monk"),
    (16, 11, "Druid"),
    (16, 12, "Demon Hunter"),
    (17, 0, "Humanoid"),
    (17, 1, "Dragonkin"),
    (17, 2, "Flying"),
    (17, 3, "Undead"),
    (17, 4, "Critter"),
    (17, 5, "Magic"),
    (17, 6, "Elemental"),
    (17, 7, "Beast"),
    (17, 8, "Aquatic"),
    // ItemSubClass 12.1.0.69497 rows 329..342; docs/item-profession-subclass-data.md.
    (19, 0, "Blacksmithing"),
    (19, 1, "Leatherworking"),
    (19, 2, "Alchemy"),
    (19, 3, "Herbalism"),
    (19, 4, "Cooking"),
    (19, 5, "Mining"),
    (19, 6, "Tailoring"),
    (19, 7, "Engineering"),
    (19, 8, "Enchanting"),
    (19, 9, "Fishing"),
    (19, 10, "Skinning"),
    (19, 11, "Jewelcrafting"),
    (19, 12, "Inscription"),
    (19, 13, "Archaeology"),
    (20, 0, "Decor"),
    (20, 1, "Housing Dye"),
    (20, 2, "Room"),
    (20, 3, "Room Customization"),
    (20, 4, "Exterior Customization"),
    (20, 5, "Service Item"),
];

const INV_TYPE_EQUIP_LOCS: &[(u8, &str)] = &[
    (1, "INVTYPE_HEAD"),
    (2, "INVTYPE_NECK"),
    (3, "INVTYPE_SHOULDER"),
    (4, "INVTYPE_BODY"),
    (5, "INVTYPE_CHEST"),
    (6, "INVTYPE_WAIST"),
    (7, "INVTYPE_LEGS"),
    (8, "INVTYPE_FEET"),
    (9, "INVTYPE_WRIST"),
    (10, "INVTYPE_HAND"),
    (11, "INVTYPE_FINGER"),
    (12, "INVTYPE_TRINKET"),
    (13, "INVTYPE_WEAPON"),
    (14, "INVTYPE_SHIELD"),
    (15, "INVTYPE_RANGED"),
    (16, "INVTYPE_CLOAK"),
    (17, "INVTYPE_2HWEAPON"),
    (20, "INVTYPE_ROBE"),
    (21, "INVTYPE_WEAPONMAINHAND"),
    (22, "INVTYPE_WEAPONOFFHAND"),
    (23, "INVTYPE_HOLDABLE"),
];

pub(crate) fn item_class_from_inv_type(inv_type: u8) -> &'static str {
    match inv_type {
        13 | 15 | 17 | 21 | 22 | 25 | 26 => "Weapon",
        1..=12 | 14 | 16 | 23 => "Armor",
        _ => "Miscellaneous",
    }
}

pub(crate) fn inv_type_to_class_id(inv_type: u8) -> i32 {
    match inv_type {
        13 | 15 | 17 | 21 | 22 | 25 | 26 => 2,
        1..=12 | 14 | 16 | 23 => 4,
        _ => 15,
    }
}

pub(crate) fn item_class_name(class_id: i32) -> &'static str {
    if let Some(name) = mists_item_class_name(class_id) {
        return name;
    }

    ITEM_CLASS_NAMES
        .iter()
        .find_map(|(id, name)| (*id == class_id).then_some(*name))
        .unwrap_or("Unknown")
}

pub(super) fn item_subclass_name(class_id: i32, subclass_id: i32) -> &'static str {
    if let Some(name) = mists_item_subclass_name(class_id, subclass_id) {
        return name;
    }

    if let Some(name) = ITEM_SUBCLASS_NAMES
        .iter()
        .find_map(|(class, subclass, name)| {
            (*class == class_id && *subclass == subclass_id).then_some(*name)
        })
    {
        return name;
    }

    if is_known_item_subclass_id(class_id, subclass_id) {
        return "Other";
    }

    "Unknown"
}

#[cfg(feature = "client-mists")]
fn mists_item_class_name(class_id: i32) -> Option<&'static str> {
    match class_id {
        7 => Some("Trade Goods"),
        _ => None,
    }
}

#[cfg(not(feature = "client-mists"))]
fn mists_item_class_name(_class_id: i32) -> Option<&'static str> {
    None
}

#[cfg(feature = "client-mists")]
fn mists_item_subclass_name(class_id: i32, subclass_id: i32) -> Option<&'static str> {
    match (class_id, subclass_id) {
        (0, 1) => Some("Potion"),
        (0, 2) => Some("Elixir"),
        (0, 3) => Some("Flask"),
        (0, 4) => Some("Scroll"),
        (0, 6) => Some("Item Enhancement"),
        (0, 7) => Some("Bandage"),
        (2, 11) => Some("One-Handed Exotics"),
        (2, 12) => Some("Two-Handed Exotics"),
        (7, 2) => Some("Explosives"),
        (7, 3) => Some("Devices"),
        _ => None,
    }
}

#[cfg(not(feature = "client-mists"))]
fn mists_item_subclass_name(_class_id: i32, _subclass_id: i32) -> Option<&'static str> {
    None
}

fn is_known_item_subclass_id(class_id: i32, subclass_id: i32) -> bool {
    subclass_id >= 0 && subclass_id < standard_item_subclass_count(class_id)
}

fn standard_item_subclass_count(class_id: i32) -> i32 {
    match class_id {
        0 => 13,  // Consumable
        1 => 8,   // Container
        2 => 21,  // Weapon
        3 => 12,  // Gem
        4 => 12,  // Armor
        5 => 3,   // Reagent
        6 => 6,   // Projectile
        7 => 21,  // Tradegoods
        9 => 12,  // Recipe
        12 => 1,  // Quest
        13 => 1,  // Key
        15 => 7,  // Miscellaneous
        16 => 13, // Glyph
        17 => 9,  // Battle Pet
        18 => 1,  // WoW Token
        19 => 14, // Profession
        20 => 6,  // Housing
        _ => 0,
    }
}

pub(crate) fn inv_type_to_subclass(inv_type: u8) -> &'static str {
    match inv_type {
        1 => "Head",
        2 => "Neck",
        3 => "Shoulder",
        4 => "Shirt",
        5 => "Chest",
        6 => "Waist",
        7 => "Legs",
        8 => "Feet",
        9 => "Wrist",
        10 => "Hands",
        11 => "Finger",
        12 => "Trinket",
        14 => "Shield",
        16 => "Back",
        _ => "Junk",
    }
}

pub(crate) fn inv_type_to_equip_loc(inv_type: u8) -> &'static str {
    INV_TYPE_EQUIP_LOCS
        .iter()
        .find_map(|(id, equip_loc)| (*id == inv_type).then_some(*equip_loc))
        .unwrap_or("")
}

pub(crate) fn global_table(state: &mut LuaState, name: &str) -> Val {
    let key_ref = state.gc.intern_string(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    if matches!(current, Val::Table(_)) {
        return current;
    }
    let table = create_table(state);
    let global = state.global;
    if let Some(globals) = state.gc.tables.get_mut(global) {
        let _ = globals.raw_set(Val::Str(key_ref), table, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table
}

pub(crate) fn unit_is_reachable(state: &SimState, unit: &str) -> bool {
    match unit {
        "" => false,
        "player" | "pet" | "vehicle" => true,
        "target" => state.current_target.is_some(),
        "focus" => state.current_focus.is_some(),
        other => crate::lua_api::globals::unit_api::parse_party_index(other)
            .is_some_and(|index| state.party_group_active && index < state.party_members.len()),
    }
}

pub(crate) fn current_item_upgrade_location(state: &mut LuaState) -> Option<(i32, i32)> {
    let storage = global_table(state, "__item_upgrade_state");
    let location = table_get(state, storage, "location");
    let Val::Table(_) = location else { return None };
    let bag = match table_get(state, location, "bagID") {
        Val::Num(value) => value as i32,
        _ => return None,
    };
    let slot = match table_get(state, location, "slotIndex") {
        Val::Num(value) => value as i32,
        _ => return None,
    };
    Some((bag, slot))
}
