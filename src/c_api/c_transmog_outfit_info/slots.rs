//! Outfit-slot topology and per-outfit secondary shoulder selection.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

struct SlotDefinition {
    category: i32,
    name: &'static str,
    position: i32,
}

// INFERRED layout positions; enum slot/category and inventory identities follow
// the generated constants. Separate left shoulder appears only when enabled.
const SLOTS: [SlotDefinition; 15] = [
    SlotDefinition {
        category: 1,
        name: "HEADSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 2,
        name: "SHOULDERSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 2,
        name: "SHOULDERSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 3,
        name: "BACKSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 4,
        name: "CHESTSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 6,
        name: "TABARDSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 5,
        name: "SHIRTSLOT",
        position: 0,
    },
    SlotDefinition {
        category: 7,
        name: "WRISTSLOT",
        position: 1,
    },
    SlotDefinition {
        category: 8,
        name: "HANDSSLOT",
        position: 1,
    },
    SlotDefinition {
        category: 9,
        name: "WAISTSLOT",
        position: 1,
    },
    SlotDefinition {
        category: 10,
        name: "LEGSSLOT",
        position: 1,
    },
    SlotDefinition {
        category: 11,
        name: "FEETSLOT",
        position: 1,
    },
    SlotDefinition {
        category: 0,
        name: "MAINHANDSLOT",
        position: 2,
    },
    SlotDefinition {
        category: 0,
        name: "SECONDARYHANDSLOT",
        position: 2,
    },
    SlotDefinition {
        category: 0,
        name: "RANGEDSLOT",
        position: 2,
    },
];

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "IsSlotWeaponSlot",
            is_weapon as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("SlotHasSecondary", has_secondary),
        ("GetSecondarySlotState", get_secondary),
        ("SetSecondarySlotState", set_secondary),
        ("SetViewedWeaponOptionForSlot", set_weapon_option),
        ("GetWeaponOptionsForSlot", get_weapon_options),
        ("GetEquippedSlotOptionFromTransmogSlot", get_equipped_option),
        ("GetCollectionInfoForSlotAndOption", get_collection),
        ("GetSlotGroupInfo", get_groups),
        ("GetAllSlotLocationInfo", get_all_locations),
        ("GetLinkedSlotInfo", get_linked_slots),
        (
            "GetTransmogOutfitSlotFromInventorySlot",
            slot_from_inventory_slot,
        ),
        (
            "GetTransmogOutfitSlotForInventoryType",
            slot_for_inventory_type,
        ),
        (
            "GetItemModifiedAppearanceEffectiveCategory",
            effective_category,
        ),
        ("GetIllusionDefaultIMAIDForCollectionType", default_illusion),
        ("GetUnassignedAtlasForSlot", unassigned_atlas),
        ("GetUnassignedDisplayAtlasForSlot", unassigned_display_atlas),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn read_slot(state: &LuaState) -> LuaResult<i32> {
    let slot = i32::from_stack(state, 1)?;
    if !(0..=14).contains(&slot) {
        return Err(rilua::runtime_error("invalid outfit slot"));
    }
    Ok(slot)
}

fn is_weapon(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    state.push(Val::Bool(slot >= 12));
    Ok(1)
}

fn has_secondary(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    state.push(Val::Bool(matches!(slot, 1 | 2)));
    Ok(1)
}

fn get_secondary(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let enabled = {
        let sim = borrow_state(state)?;
        let saved = sim
            .viewed_transmog_outfit_id
            .and_then(|id| sim.transmog_outfits.saved.get(&id));
        saved
            .and_then(|outfit| outfit.secondary_slots.get(&slot).copied())
            .unwrap_or(false)
    };
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_secondary(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let enabled = bool::from_stack(state, 2)?;
    if !matches!(slot, 1 | 2) {
        return Err(rilua::runtime_error("slot has no secondary"));
    }
    {
        let mut sim = borrow_state_mut(state)?;
        let id = sim
            .viewed_transmog_outfit_id
            .ok_or_else(|| rilua::runtime_error("no viewed outfit"))?;
        // INFERRED: both shoulders share one immediate per-outfit split toggle.
        let slots = &mut sim
            .transmog_outfits
            .saved
            .entry(id)
            .or_default()
            .secondary_slots;
        slots.insert(1, enabled);
        slots.insert(2, enabled);
    }
    dispatch_event_now(state, "VIEWED_TRANSMOG_OUTFIT_SECONDARY_SLOTS_CHANGED", &[])?;
    Ok(0)
}

fn set_weapon_option(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let option = i32::from_stack(state, 2)?;
    if !(0..=11).contains(&option) {
        return Err(rilua::runtime_error("invalid weapon option"));
    }
    borrow_state_mut(state)?
        .transmog_outfits
        .viewed_weapon_options
        .insert(slot, option);
    dispatch_event_now(
        state,
        "VIEWED_TRANSMOG_OUTFIT_SLOT_WEAPON_OPTION_CHANGED",
        &[Val::Num(slot as f64), Val::Num(option as f64)],
    )?;
    Ok(0)
}

fn weapon_options(slot: i32) -> &'static [(i32, &'static str)] {
    // INFERRED option catalog, without artifact unlocks or spec restrictions.
    match slot {
        12 => &[(1, "One-handed weapon"), (2, "Two-handed weapon")],
        13 => &[(1, "One-handed weapon"), (4, "Off-hand"), (5, "Shield")],
        14 => &[(3, "Ranged weapon")],
        _ => &[(0, "None")],
    }
}

fn get_equipped_option(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    const INVENTORY_SLOTS: [i32; 15] = [1, 3, 3, 15, 5, 19, 4, 9, 10, 6, 7, 8, 16, 17, 18];
    let inventory_type = {
        let sim = borrow_state(state)?;
        sim.player
            .equipped_items
            .get(&INVENTORY_SLOTS[slot as usize])
            .and_then(|equipped| crate::items::get_item(equipped.item_id))
            .map(|item| item.inventory_type)
    };
    let Some(inventory_type) = inventory_type else {
        return Ok(0);
    };
    // INFERRED: inventory type selects the basic weapon option. Artifact/spec
    // options need explicit artifact state and are not fabricated.
    let option = match inventory_type {
        13 | 21 | 22 => 1,
        17 => 2,
        15 | 25 | 26 => 3,
        23 => 4,
        14 => 5,
        _ => 0,
    };
    state.push(Val::Num(option as f64));
    Ok(1)
}

fn get_weapon_options(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, (option, name)) in weapon_options(slot).iter().enumerate() {
        let row = create_table(state);
        let name = create_string(state, name);
        table_set(state, row, "weaponOption", Val::Num(*option as f64));
        table_set(state, row, "name", name);
        table_set(state, row, "enabled", Val::Bool(true));
        table_set_num(state, table, (index + 1) as f64, row);
    }
    state.push(array);
    state.push(Val::Nil);
    Ok(2)
}

fn category_matches(slot: i32, option: i32, category: i32) -> bool {
    if slot < 12 {
        return option == 0 && SLOTS[slot as usize].category == category;
    }
    match option {
        1 => matches!(category, 12..=17 | 28),
        2 | 7 => matches!(category, 20..=24),
        3 => matches!(category, 25..=27),
        4 => category == 19,
        5 => category == 18,
        8..=11 => category == 29,
        _ => false,
    }
}

fn get_collection(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let option = i32::from_stack(state, 2)?;
    let category = i32::from_stack(state, 3)?;
    if !category_matches(slot, option, category) {
        return Ok(0);
    }
    let table = create_table(state);
    let name = create_string(state, collection_name(category));
    table_set(state, table, "name", name);
    table_set(state, table, "isWeapon", Val::Bool(slot >= 12));
    table_set(
        state,
        table,
        "canHaveIllusions",
        Val::Bool(matches!(option, 1 | 2 | 7)),
    );
    state.push(table);
    Ok(1)
}

fn collection_name(category: i32) -> &'static str {
    const NAMES: [&str; 30] = [
        "None",
        "Head",
        "Shoulder",
        "Back",
        "Chest",
        "Shirt",
        "Tabard",
        "Wrist",
        "Hands",
        "Waist",
        "Legs",
        "Feet",
        "Wand",
        "One-handed axe",
        "One-handed sword",
        "One-handed mace",
        "Dagger",
        "Fist weapon",
        "Shield",
        "Off-hand",
        "Two-handed axe",
        "Two-handed sword",
        "Two-handed mace",
        "Staff",
        "Polearm",
        "Bow",
        "Gun",
        "Crossbow",
        "Warglaives",
        "Artifact",
    ];
    NAMES[category as usize]
}

fn get_groups(state: &mut LuaState) -> LuaResult<u32> {
    let (split, hunter) = {
        let sim = borrow_state(state)?;
        let saved = sim
            .viewed_transmog_outfit_id
            .and_then(|id| sim.transmog_outfits.saved.get(&id));
        let split = saved
            .and_then(|outfit| outfit.secondary_slots.get(&1))
            .copied()
            .unwrap_or(false);
        (split, sim.player.class_index == 3)
    };
    let array = create_table(state);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for position in 0..=2 {
        let group = build_group(state, position, split, hunter);
        table_set_num(state, table, (position + 1) as f64, group);
    }
    state.push(array);
    Ok(1)
}

fn build_group(state: &mut LuaState, position: i32, split: bool, hunter: bool) -> Val {
    let group = create_table(state);
    let appearances = create_table(state);
    let illusions = create_table(state);
    let Val::Table(appearance_table) = appearances else {
        unreachable!()
    };
    let Val::Table(illusion_table) = illusions else {
        unreachable!()
    };
    let mut appearance_index = 0;
    let mut illusion_index = 0;
    for (slot, definition) in SLOTS.iter().enumerate() {
        if definition.position != position || (slot == 2 && !split) || (slot == 14 && !hunter) {
            continue;
        }
        appearance_index += 1;
        let row = build_slot(state, slot, 0);
        table_set_num(state, appearance_table, appearance_index as f64, row);
        if matches!(slot, 12 | 13) {
            illusion_index += 1;
            let row = build_slot(state, slot, 1);
            table_set_num(state, illusion_table, illusion_index as f64, row);
        }
    }
    table_set(state, group, "position", Val::Num(position as f64));
    table_set(state, group, "appearanceSlotInfo", appearances);
    table_set(state, group, "illusionSlotInfo", illusions);
    group
}

fn build_slot(state: &mut LuaState, slot: usize, kind: i32) -> Val {
    let definition = &SLOTS[slot];
    let row = create_table(state);
    let name = create_string(state, definition.name);
    table_set(state, row, "slot", Val::Num(slot as f64));
    table_set(state, row, "type", Val::Num(kind as f64));
    table_set(
        state,
        row,
        "collectionType",
        Val::Num(definition.category as f64),
    );
    table_set(state, row, "slotName", name);
    table_set(state, row, "isSecondary", Val::Bool(slot == 2));
    row
}

fn get_all_locations(state: &mut LuaState) -> LuaResult<u32> {
    let appearances = create_table(state);
    let illusions = create_table(state);
    let Val::Table(appearance_table) = appearances else {
        unreachable!()
    };
    let Val::Table(illusion_table) = illusions else {
        unreachable!()
    };
    // Include secondary locations even when hidden, so TransmogUtil can cache
    // both sides before a later split toggle. Ranged availability is class-driven.
    let hunter = borrow_state(state)?.player.class_index == 3;
    let mut index = 0;
    for slot in 0..15 {
        if slot == 14 && !hunter {
            continue;
        }
        index += 1;
        let row = build_slot(state, slot, 0);
        table_set_num(state, appearance_table, index as f64, row);
    }
    for (index, slot) in [12, 13].into_iter().enumerate() {
        let row = build_slot(state, slot, 1);
        table_set_num(state, illusion_table, (index + 1) as f64, row);
    }
    state.push(appearances);
    state.push(illusions);
    Ok(2)
}

fn get_linked_slots(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    if !matches!(slot, 1 | 2) {
        return Ok(0);
    }
    let table = create_table(state);
    let primary = build_slot(state, 1, 0);
    let secondary = build_slot(state, 2, 0);
    table_set(state, table, "primarySlotInfo", primary);
    table_set(state, table, "secondarySlotInfo", secondary);
    state.push(table);
    Ok(1)
}

fn slot_from_inventory_slot(state: &mut LuaState) -> LuaResult<u32> {
    let inventory_slot = i32::from_stack(state, 1)?;
    let Some(slot) = outfit_slot_from_inventory(inventory_slot) else {
        return Ok(0);
    };
    state.push(Val::Num(slot as f64));
    Ok(1)
}

pub(super) fn outfit_slot_from_inventory(inventory_slot: i32) -> Option<i32> {
    Some(match inventory_slot {
        0 => 0,
        2 => 1,
        3 => 6,
        4 => 4,
        5 => 9,
        6 => 10,
        7 => 11,
        8 => 7,
        9 => 8,
        14 => 3,
        15 => 12,
        16 => 13,
        17 => 14,
        18 => 5,
        _ => return None,
    })
}

fn slot_for_inventory_type(state: &mut LuaState) -> LuaResult<u32> {
    let inventory_type = i32::from_stack(state, 1)?;
    let slot = match inventory_type {
        1 => 0,
        3 => 1,
        4 => 6,
        5 | 20 => 4,
        6 => 9,
        7 => 10,
        8 => 11,
        9 => 7,
        10 => 8,
        13 | 17 | 21 => 12,
        14 | 22 | 23 => 13,
        15 | 25 | 26 => 14,
        16 => 3,
        19 => 5,
        _ => return Ok(0),
    };
    state.push(Val::Num(slot as f64));
    Ok(1)
}

fn atlas_suffix(slot: i32) -> &'static str {
    const SUFFIXES: [&str; 15] = [
        "head",
        "shoulders",
        "shoulders",
        "back",
        "chest",
        "tabard",
        "shirt",
        "wrist",
        "hands",
        "waist",
        "legs",
        "feet",
        "mainhand",
        "offhand",
        "mainhand",
    ];
    SUFFIXES[slot as usize]
}

fn unassigned_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    // INFERRED slot-to-atlas assignment; atlas identities exist in data/atlas.rs.
    let atlas = create_string(
        state,
        &format!("transmog-gearslot-unassigned-{}", atlas_suffix(slot)),
    );
    state.push(atlas);
    Ok(1)
}

fn unassigned_display_atlas(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let atlas = create_string(
        state,
        &format!("transmog-appearance-unassigned-{}", atlas_suffix(slot)),
    );
    state.push(atlas);
    Ok(1)
}

fn effective_category(state: &mut LuaState) -> LuaResult<u32> {
    let id = i64::from_stack(state, 1)?;
    let category = borrow_state(state)?
        .transmog_appearance_sources
        .get(&id)
        .map(|row| row.category)
        .unwrap_or(0);
    state.push(Val::Num(category as f64));
    Ok(1)
}

fn default_illusion(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    // INFERRED: the first hide-visual illusion is the category's default.
    let id = borrow_state(state)?
        .transmog_illusions
        .iter()
        .find(|row| row.category as i32 == category && row.is_hide_visual)
        .map(|row| row.source_id);
    let Some(id) = id else { return Ok(0) };
    state.push(Val::Num(id as f64));
    Ok(1)
}
