//! `C_EquipmentSet` namespace — the saved-loadout API consumed by
//! `Blizzard_UIPanels_Game/Mainline/PaperDollFrame.lua`'s
//! `EquipmentManagerPane`. The pane's flow is:
//!
//!   1. `CreateEquipmentSet(name, icon)` from the icon-picker dialog
//!   2. `EQUIPMENT_SETS_CHANGED` fires → `PaperDollEquipmentManagerPane_Update(true)`
//!   3. The pane reads `GetEquipmentSetIDs()` + `GetEquipmentSetInfo(id)`
//!      to populate its scroll box.
//!
//! Without (1) creating real state, (3) returns nothing and the new
//! set is invisible. Without (2) firing, the pane never refreshes
//! even if state changes.

use std::collections::{HashMap, HashSet};

use crate::c_api::ensure_namespace;
use crate::event::Event;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table, frame_ref,
    table_set_num, val_to_string,
};
use crate::lua_api::script_helpers::{get_dispatch_script, get_event_listeners};
use crate::lua_api::state_types::{CharacterStats, EquipmentSet, EquippedItem};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const EQUIPMENT_SETS_CHANGED: &str = "EQUIPMENT_SETS_CHANGED";
const EQUIPMENT_SWAP_PENDING: &str = "EQUIPMENT_SWAP_PENDING";
const EQUIPMENT_SWAP_FINISHED: &str = "EQUIPMENT_SWAP_FINISHED";

const C_EQUIPMENT_SET_METHODS: &[(&str, rilua::RustFn)] = &[
    ("CanUseEquipmentSets", c_can_use_equipment_sets),
    ("CreateEquipmentSet", c_create_equipment_set),
    ("DeleteEquipmentSet", c_delete_equipment_set),
    ("ModifyEquipmentSet", c_modify_equipment_set),
    ("SaveEquipmentSet", c_save_equipment_set),
    ("UseEquipmentSet", c_use_equipment_set),
    ("PickupEquipmentSet", c_pickup_equipment_set),
    ("GetEquipmentSetID", c_get_equipment_set_id),
    ("GetEquipmentSetIDs", c_get_equipment_set_ids),
    ("GetNumEquipmentSets", c_get_num_equipment_sets),
    ("GetEquipmentSetInfo", c_get_equipment_set_info),
    (
        "GetEquipmentSetAssignedSpec",
        c_get_equipment_set_assigned_spec,
    ),
    ("GetEquipmentSetForSpec", c_get_equipment_set_for_spec),
    ("AssignSpecToEquipmentSet", c_assign_spec_to_equipment_set),
    ("UnassignEquipmentSetSpec", c_unassign_equipment_set_spec),
    ("GetIgnoredSlots", c_get_ignored_slots),
    ("GetItemIDs", c_get_item_ids),
    ("GetItemLocations", c_get_item_locations),
    ("IgnoreSlotForSave", c_ignore_slot_for_save),
    ("UnignoreSlotForSave", c_unignore_slot_for_save),
    ("IsSlotIgnoredForSave", c_is_slot_ignored_for_save),
    ("ClearIgnoredSlotsForSave", c_clear_ignored_slots_for_save),
];

pub(super) fn register_c_equipment_set(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_EquipmentSet")?;
    for &(name, func) in C_EQUIPMENT_SET_METHODS {
        table_set_rust_fn_static(state, table_ref, name, func)?;
    }
    Ok(())
}

// ── helpers ─────────────────────────────────────────────────────────

fn val_to_optional_string(state: &LuaState, val: Val) -> Option<String> {
    match val {
        Val::Nil => None,
        Val::Str(_) => val_to_string(state, val),
        Val::Num(n) => Some(format!("{}", n as i64)),
        _ => None,
    }
}

fn fire_event(state: &mut LuaState, event_name: &'static str, args: Vec<Val>) -> LuaResult<()> {
    if let Ok(mut sim) = borrow_state_mut(state) {
        sim.events.push(Event {
            name: event_name.to_string(),
            args: Vec::new(),
        });
    }
    for widget_id in get_event_listeners(state, event_name) {
        let Some(handler) = get_dispatch_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_val = create_string(state, event_name);
        let mut call_args = Vec::with_capacity(2 + args.len());
        call_args.push(frame);
        call_args.push(event_val);
        call_args.extend(args.iter().cloned());
        let _ = call_function_state(state, handler, &call_args);
    }
    Ok(())
}

fn snapshot_equipped_items(state: &LuaState) -> (HashMap<i32, u32>, HashMap<i32, i64>) {
    let mut item_ids = HashMap::new();
    let mut locations = HashMap::new();
    let Ok(sim) = borrow_state(state) else {
        return (item_ids, locations);
    };
    for (slot, item) in &sim.player.equipped_items {
        item_ids.insert(*slot, item.item_id);
        // Pack location: slot id stored verbatim — the real client uses
        // `EquipmentManager_PackLocation` but the pane only consumes
        // this opaquely via `GetItemLocations` (loop body iterates and
        // passes each value back into `EquipmentManager_*` helpers).
        locations.insert(*slot, *slot as i64);
    }
    (item_ids, locations)
}

fn apply_equipment_set(state: &mut LuaState, set_id: i32) -> LuaResult<bool> {
    let set = {
        let sim = borrow_state(state)?;
        let Some(set) = sim.equipment_manager.sets.iter().find(|s| s.id == set_id) else {
            return Ok(false);
        };
        set.clone()
    };

    let mut sim = borrow_state_mut(state)?;
    let slots_to_update: HashSet<i32> = sim
        .player
        .equipped_items
        .keys()
        .chain(set.item_ids.keys())
        .copied()
        .filter(|slot| !set.ignored_slots.contains(slot))
        .collect();

    for slot in slots_to_update {
        match set.item_ids.get(&slot) {
            Some(item_id) => {
                sim.player.equipped_items.insert(
                    slot,
                    EquippedItem {
                        item_id: *item_id,
                        enchant_id: 0,
                        gem_ids: [0; 3],
                    },
                );
            }
            None => {
                sim.player.equipped_items.remove(&slot);
            }
        }
    }

    sim.player.stats = CharacterStats::compute(&sim.player.equipped_items, sim.player.class_index);
    sim.equipment_manager.last_used_set_id = Some(set_id);
    Ok(true)
}

// ── mutation surface ────────────────────────────────────────────────

fn c_create_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let name = match val_to_string(state, stack_val(state, 1)) {
        Some(n) if !n.is_empty() => n,
        _ => return Ok(0),
    };
    let icon = val_to_optional_string(state, stack_val(state, 2)).unwrap_or_default();

    let (item_ids, item_locations) = snapshot_equipped_items(state);

    {
        let mut sim = borrow_state_mut(state)?;
        // Reject duplicate names — matches retail, where the icon-picker
        // dialog refuses to call CreateEquipmentSet twice with the same
        // name (see `IconSelectorPopupFrame_OkayButton_OnClick`).
        if sim.equipment_manager.sets.iter().any(|s| s.name == name) {
            return Ok(0);
        }
        let id = sim.equipment_manager.next_id;
        sim.equipment_manager.next_id += 1;
        sim.equipment_manager.sets.push(EquipmentSet {
            id,
            name,
            icon,
            spec_index: None,
            ignored_slots: HashSet::new(),
            item_locations,
            item_ids,
        });
    }
    fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    Ok(0)
}

fn c_modify_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let new_name = match val_to_string(state, stack_val(state, 2)) {
        Some(n) if !n.is_empty() => n,
        _ => return Ok(0),
    };
    let new_icon = val_to_optional_string(state, stack_val(state, 3));
    let mut changed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(set) = sim
            .equipment_manager
            .sets
            .iter_mut()
            .find(|s| s.id == set_id)
        {
            set.name = new_name;
            if let Some(icon) = new_icon {
                set.icon = icon;
            }
            changed = true;
        }
    }
    if changed {
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(0)
}

fn c_delete_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let mut removed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        let before = sim.equipment_manager.sets.len();
        sim.equipment_manager.sets.retain(|s| s.id != set_id);
        if sim.equipment_manager.sets.len() != before {
            removed = true;
            if sim.equipment_manager.last_used_set_id == Some(set_id) {
                sim.equipment_manager.last_used_set_id = None;
            }
        }
    }
    if removed {
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(0)
}

fn c_save_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let icon = val_to_optional_string(state, stack_val(state, 2));
    let (item_ids, item_locations) = snapshot_equipped_items(state);
    let mut changed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        let pending_ignored = sim.equipment_manager.ignored_slots_pending_save.clone();
        if let Some(set) = sim
            .equipment_manager
            .sets
            .iter_mut()
            .find(|s| s.id == set_id)
        {
            set.item_ids = item_ids;
            set.item_locations = item_locations;
            set.ignored_slots = pending_ignored;
            if let Some(icon) = icon {
                set.icon = icon;
            }
            changed = true;
        }
    }
    if changed {
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(0)
}

fn c_use_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let exists = use_equipment_set_by_id(state, set_id)?;
    state.push(Val::Bool(exists));
    Ok(1)
}

pub(crate) fn use_equipment_set_by_id(state: &mut LuaState, set_id: i32) -> LuaResult<bool> {
    let exists = apply_equipment_set(state, set_id)?;
    if exists {
        fire_event(state, EQUIPMENT_SWAP_PENDING, Vec::new())?;
        fire_event(
            state,
            EQUIPMENT_SWAP_FINISHED,
            vec![Val::Bool(true), Val::Num(set_id as f64)],
        )?;
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(exists)
}

fn c_pickup_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    // Cursor pickup of equipment sets is a hot-bar drag interaction
    // we don't simulate; the call must succeed silently so the
    // SecureHandlers attribute path doesn't error.
    let _ = i32::from_stack(state, 1)?;
    Ok(0)
}

// ── read surface ────────────────────────────────────────────────────

fn c_can_use_equipment_sets(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_get_equipment_set_id(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = val_to_string(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    let id = borrow_state(state)?
        .equipment_manager
        .sets
        .iter()
        .find(|s| s.name == name)
        .map(|s| s.id);
    match id {
        Some(id) => {
            state.push(Val::Num(id as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

fn c_get_equipment_set_ids(state: &mut LuaState) -> LuaResult<u32> {
    let ids: Vec<i32> = borrow_state(state)?
        .equipment_manager
        .sets
        .iter()
        .map(|s| s.id)
        .collect();
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        state.push(table);
        return Ok(1);
    };
    for (i, id) in ids.iter().enumerate() {
        table_set_num(state, table_ref, (i + 1) as f64, Val::Num(*id as f64));
    }
    state.push(table);
    Ok(1)
}

fn c_get_num_equipment_sets(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.equipment_manager.sets.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

struct EquipmentSetInfo {
    name: String,
    icon: String,
    id: i32,
    is_equipped: bool,
    num_items: i32,
    num_ignored: i32,
}

fn c_get_equipment_set_info(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let Some(info) = find_equipment_set_info(state, set_id)? else {
        return Ok(0);
    };
    push_equipment_set_info(state, &info);
    Ok(9)
}

fn find_equipment_set_info(state: &LuaState, set_id: i32) -> LuaResult<Option<EquipmentSetInfo>> {
    let sim = borrow_state(state)?;
    let Some(set) = sim.equipment_manager.sets.iter().find(|s| s.id == set_id) else {
        return Ok(None);
    };

    Ok(Some(EquipmentSetInfo {
        name: set.name.clone(),
        icon: set.icon.clone(),
        id: set.id,
        is_equipped: sim.equipment_manager.last_used_set_id == Some(set_id),
        num_items: set.item_ids.len() as i32,
        num_ignored: set.ignored_slots.len() as i32,
    }))
}

fn push_equipment_set_info(state: &mut LuaState, info: &EquipmentSetInfo) {
    let name_val = create_string(state, &info.name);
    state.push(name_val);
    if let Ok(parsed) = info.icon.parse::<i64>() {
        state.push(Val::Num(parsed as f64));
    } else if info.icon.is_empty() {
        state.push(Val::Num(0.0));
    } else {
        let icon_val = create_string(state, &info.icon);
        state.push(icon_val);
    }
    state.push(Val::Num(info.id as f64));
    state.push(Val::Bool(info.is_equipped));
    state.push(Val::Num(info.num_items as f64));
    // numEquipped — we don't model per-item equip diffs, so report
    // every captured item as equipped if `isEquipped`, else 0.
    let num_equipped = if info.is_equipped { info.num_items } else { 0 };
    state.push(Val::Num(num_equipped as f64));
    // numInInventory — unlocked items currently in bags. Match
    // numItems for the simulated case (no bag inventory model for
    // saved sets).
    state.push(Val::Num(info.num_items as f64));
    // numLost — items the player no longer has. Always 0 since we
    // capture the live state at save time.
    state.push(Val::Num(0.0));
    state.push(Val::Num(info.num_ignored as f64));
}

fn c_get_equipment_set_assigned_spec(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let spec = borrow_state(state)?
        .equipment_manager
        .sets
        .iter()
        .find(|s| s.id == set_id)
        .and_then(|s| s.spec_index);
    match spec {
        Some(idx) => {
            state.push(Val::Num(idx as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

fn c_get_equipment_set_for_spec(state: &mut LuaState) -> LuaResult<u32> {
    let spec_index = i32::from_stack(state, 1)?;
    let id = borrow_state(state)?
        .equipment_manager
        .sets
        .iter()
        .find(|s| s.spec_index == Some(spec_index))
        .map(|s| s.id);
    match id {
        Some(id) => {
            state.push(Val::Num(id as f64));
            Ok(1)
        }
        None => Ok(0),
    }
}

fn c_assign_spec_to_equipment_set(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let spec_index = i32::from_stack(state, 2)?;
    let mut changed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        // Each spec can only own one set — clear conflicts first.
        for s in sim.equipment_manager.sets.iter_mut() {
            if s.id != set_id && s.spec_index == Some(spec_index) {
                s.spec_index = None;
                changed = true;
            }
        }
        if let Some(set) = sim
            .equipment_manager
            .sets
            .iter_mut()
            .find(|s| s.id == set_id)
            && set.spec_index != Some(spec_index)
        {
            set.spec_index = Some(spec_index);
            changed = true;
        }
    }
    if changed {
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(0)
}

fn c_unassign_equipment_set_spec(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let mut changed = false;
    {
        let mut sim = borrow_state_mut(state)?;
        if let Some(set) = sim
            .equipment_manager
            .sets
            .iter_mut()
            .find(|s| s.id == set_id)
            && set.spec_index.take().is_some()
        {
            changed = true;
        }
    }
    if changed {
        fire_event(state, EQUIPMENT_SETS_CHANGED, Vec::new())?;
    }
    Ok(0)
}

fn c_get_ignored_slots(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let slots: Option<HashSet<i32>> = {
        let sim = borrow_state(state)?;
        sim.equipment_manager
            .sets
            .iter()
            .find(|s| s.id == set_id)
            .map(|s| s.ignored_slots.clone())
    };
    let Some(slots) = slots else {
        return Ok(0);
    };
    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for slot in &slots {
            table_set_num(state, table_ref, *slot as f64, Val::Bool(true));
        }
    }
    state.push(table);
    Ok(1)
}

fn c_get_item_ids(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let ids: Option<HashMap<i32, u32>> = {
        let sim = borrow_state(state)?;
        sim.equipment_manager
            .sets
            .iter()
            .find(|s| s.id == set_id)
            .map(|s| s.item_ids.clone())
    };
    let Some(ids) = ids else {
        return Ok(0);
    };
    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for (slot, item_id) in &ids {
            table_set_num(state, table_ref, *slot as f64, Val::Num(*item_id as f64));
        }
    }
    state.push(table);
    Ok(1)
}

fn c_get_item_locations(state: &mut LuaState) -> LuaResult<u32> {
    let set_id = i32::from_stack(state, 1)?;
    let locs: Option<HashMap<i32, i64>> = {
        let sim = borrow_state(state)?;
        sim.equipment_manager
            .sets
            .iter()
            .find(|s| s.id == set_id)
            .map(|s| s.item_locations.clone())
    };
    let Some(locs) = locs else {
        return Ok(0);
    };
    let table = create_table(state);
    if let Val::Table(table_ref) = table {
        for (slot, loc) in &locs {
            table_set_num(state, table_ref, *slot as f64, Val::Num(*loc as f64));
        }
    }
    state.push(table);
    Ok(1)
}

fn c_ignore_slot_for_save(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .equipment_manager
        .ignored_slots_pending_save
        .insert(slot);
    Ok(0)
}

fn c_unignore_slot_for_save(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .equipment_manager
        .ignored_slots_pending_save
        .remove(&slot);
    Ok(0)
}

fn c_is_slot_ignored_for_save(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let ignored = is_slot_pending_save_ignored(state, slot)?;
    state.push(Val::Bool(ignored));
    Ok(1)
}

fn is_slot_pending_save_ignored(state: &LuaState, slot: i32) -> LuaResult<bool> {
    Ok(borrow_state(state)?
        .equipment_manager
        .ignored_slots_pending_save
        .contains(&slot))
}

fn c_clear_ignored_slots_for_save(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .equipment_manager
        .ignored_slots_pending_save
        .clear();
    Ok(0)
}
