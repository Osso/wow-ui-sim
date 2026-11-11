//! `C_AzeriteEmpoweredItem` — surface read by `Blizzard_AzeriteRespecUI`
//! (respec flow) and `Blizzard_AzeriteUI` (panel flow). Backed by
//! `state.azerite_empowered`.
//!
//! Respec-flow methods:
//! - `CloseAzeriteEmpoweredItemRespec(itemLocation?)` — fires
//!   `AZERITE_EMPOWERED_ITEM_RESPEC_CLOSE` so the panel can hide via
//!   `UIPanelWindows.showFailedFunc`.
//! - `GetAzeriteEmpoweredItemRespecCost() → number` — returns
//!   `state.azerite_empowered.respec_cost` (copper). Drives
//!   `AzeriteRespecMixin:RefreshCostFrame`.
//! - `IsAzeriteEmpoweredItem(itemLocation) → bool` — true when the
//!   resolved item id is in `state.azerite_empowered.empowered_items`.
//!   Accepts both the test-shape `{itemID = N}` and the canonical
//!   `ItemLocationMixin` shapes (`{bagID, slotIndex}` and
//!   `{equipmentSlotIndex}`).
//! - `ConfirmAzeriteEmpoweredItemRespec(itemLocation)` — records the
//!   location on `last_confirmed_respec` and clears the resolved
//!   item's `selections` entry, then fires
//!   `AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED` with the same location.
//!   No-op when the location is nil or not a table (mirrors the addon's
//!   nil-cursor case).
//!
//! Panel-flow methods:
//! - `GetPowerText(itemLocation, powerID, level) → {name, description}|nil`
//!   — looks up `state.azerite_empowered.power_text` keyed by
//!   `(itemID, powerID, level)`. Returns nil when not seeded; the
//!   addon's tooltip path only reaches it after seeding.
//! - `IsHeartOfAzerothEquipped() → bool` — returns
//!   `state.azerite_empowered.heart_equipped`. Drives
//!   `AzeriteEmpoweredItemPowerMixin:Update`.
//! - `IsPowerAvailableForSpec(powerID, specID) → bool` — looks up
//!   `state.azerite_empowered.spec_available`; defaults to true so
//!   panels show powers as available unless seeded false.
//! - `SelectPower(itemLocation, powerID) → bool` — appends `powerID`
//!   to `state.azerite_empowered.selections[itemLocation]` and fires
//!   `AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED`. Returns false when
//!   the location can't be resolved to an item id.

use crate::c_api::ensure_namespace;
use crate::c_api::item_spell::parse_item_id_key;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table_with_fields};
use crate::lua_api::state::{
    AzeriteEmpoweredPowerText, AzeriteEmpoweredSelectionKey, ItemLocationData,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

type AzeriteEmpoweredMethod = fn(&mut LuaState) -> LuaResult<u32>;

const AZERITE_EMPOWERED_METHODS: &[(&str, AzeriteEmpoweredMethod)] = &[
    (
        "CloseAzeriteEmpoweredItemRespec",
        close_azerite_empowered_item_respec,
    ),
    (
        "GetAzeriteEmpoweredItemRespecCost",
        get_azerite_empowered_item_respec_cost,
    ),
    ("IsAzeriteEmpoweredItem", is_azerite_empowered_item),
    (
        "ConfirmAzeriteEmpoweredItemRespec",
        confirm_azerite_empowered_item_respec,
    ),
    ("GetPowerText", get_power_text),
    ("IsHeartOfAzerothEquipped", is_heart_of_azeroth_equipped),
    ("IsPowerAvailableForSpec", is_power_available_for_spec),
    ("SelectPower", select_power),
];

pub(crate) fn register_c_azerite_empowered_item_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_AzeriteEmpoweredItem")?;
    for &(name, func) in AZERITE_EMPOWERED_METHODS {
        table_set_rust_fn_static(state, ns, name, func)?;
    }
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(
            state,
            ns,
            "IsAzeriteEmpoweredItemByID",
            is_azerite_empowered_item_by_id,
        )?;
    }
    Ok(())
}

fn close_azerite_empowered_item_respec(state: &mut LuaState) -> LuaResult<u32> {
    let location = read_optional_location(state, 1);
    {
        let mut sim = borrow_state_mut(state)?;
        sim.azerite_empowered.last_close_request = location;
    }
    dispatch_event_now(state, "AZERITE_EMPOWERED_ITEM_RESPEC_CLOSE", &[])?;
    Ok(0)
}

fn get_azerite_empowered_item_respec_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = borrow_state(state)?.azerite_empowered.respec_cost;
    state.push(Val::Num(cost as f64));
    Ok(1)
}

fn is_azerite_empowered_item(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = resolve_location_item_id(state, 1);
    let empowered = match item_id {
        Some(id) => {
            let sim = borrow_state(state)?;
            let empowered_items = &sim.azerite_empowered.empowered_items;
            std::collections::HashSet::contains(empowered_items, &id)
        }
        None => false,
    };
    state.push(Val::Bool(empowered));
    Ok(1)
}

fn is_azerite_empowered_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let value =
        rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, 1))?;
    if !matches!(value, Val::Num(_) | Val::Str(_)) {
        return Err(rilua::runtime_error(
            "C_AzeriteEmpoweredItem.IsAzeriteEmpoweredItemByID: itemInfo must be an item ID or link",
        ));
    }
    let item_id = parse_item_id_key(state, value).and_then(|id| i32::try_from(id).ok());
    let empowered = match item_id {
        Some(id) => borrow_state(state)?
            .azerite_empowered
            .empowered_items
            .contains(&id),
        None => false,
    };
    state.push(Val::Bool(empowered));
    Ok(1)
}

fn confirm_azerite_empowered_item_respec(state: &mut LuaState) -> LuaResult<u32> {
    let Some(location) = read_optional_location(state, 1) else {
        return Ok(0);
    };
    let location_arg = crate::lua_bridge::stack_val(state, 1);
    let selection_key = build_selection_key(state, 1);
    let mut sim = borrow_state_mut(state)?;
    sim.azerite_empowered.last_confirmed_respec = Some(location);
    if let Some(key) = selection_key {
        sim.azerite_empowered.selections.remove(&key);
    }
    drop(sim);
    dispatch_event_now(
        state,
        "AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED",
        &[location_arg],
    )?;
    Ok(0)
}

fn get_power_text(state: &mut LuaState) -> LuaResult<u32> {
    let Some(text) = lookup_power_text(state)? else {
        push_nil_power_text(state);
        return Ok(1);
    };
    push_power_text_table(state, &text);
    Ok(1)
}

fn push_nil_power_text(state: &mut LuaState) {
    state.push(Val::Nil);
}

fn push_power_text_table(state: &mut LuaState, text: &AzeriteEmpoweredPowerText) {
    let name_str = state.gc.intern_string(text.name.as_bytes());
    let desc_str = state.gc.intern_string(text.description.as_bytes());
    let table = create_table_with_fields(
        state,
        &[
            ("name", Val::Str(name_str)),
            ("description", Val::Str(desc_str)),
        ],
    );
    state.push(table);
}

fn lookup_power_text(state: &mut LuaState) -> LuaResult<Option<AzeriteEmpoweredPowerText>> {
    let Some(item_id) = resolve_location_item_id(state, 1) else {
        return Ok(None);
    };
    let Ok(power_id) = i32::from_stack(state, 2) else {
        return Ok(None);
    };
    let Ok(level) = i32::from_stack(state, 3) else {
        return Ok(None);
    };
    Ok(borrow_state(state)?
        .azerite_empowered
        .power_text
        .get(&(item_id, power_id, level))
        .cloned())
}

fn is_heart_of_azeroth_equipped(state: &mut LuaState) -> LuaResult<u32> {
    let equipped = borrow_state(state)?.azerite_empowered.heart_equipped;
    state.push(Val::Bool(equipped));
    Ok(1)
}

fn is_power_available_for_spec(state: &mut LuaState) -> LuaResult<u32> {
    let power_id = i32::from_stack(state, 1)?;
    let spec_id = i32::from_stack(state, 2)?;
    let available = borrow_state(state)?
        .azerite_empowered
        .spec_available
        .get(&(power_id, spec_id))
        .copied()
        .unwrap_or(true);
    state.push(Val::Bool(available));
    Ok(1)
}

fn select_power(state: &mut LuaState) -> LuaResult<u32> {
    let Some(key) = build_selection_key(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let power_id = match i32::from_stack(state, 2) {
        Ok(value) => value,
        Err(_) => {
            state.push(Val::Bool(false));
            return Ok(1);
        }
    };
    {
        let mut sim = borrow_state_mut(state)?;
        sim.azerite_empowered
            .selections
            .entry(key)
            .or_default()
            .push(power_id);
    }
    dispatch_event_now(state, "AZERITE_EMPOWERED_ITEM_SELECTION_UPDATED", &[])?;
    state.push(Val::Bool(true));
    Ok(1)
}

/// Build the composite selection key for `state.azerite_empowered.selections`
/// from an `itemLocation` arg. Returns None when the location can't be
/// resolved to an item id (no shape match, or the bag/equipment lookup
/// misses).
fn build_selection_key(state: &mut LuaState, slot: i32) -> Option<AzeriteEmpoweredSelectionKey> {
    let item_id = resolve_location_item_id(state, slot)?;
    let fields = read_location_fields(state, slot)?;
    Some(AzeriteEmpoweredSelectionKey {
        item_id,
        bag_id: fields.bag_id,
        slot_index: fields.slot_index,
        equipment_slot_index: fields.equipment_slot,
    })
}

/// Read the `{bagID, slotIndex, equipmentSlotIndex}` shape produced by
/// `ItemLocationMixin` (or its test-friendly cousin `{itemID}`) into the
/// simulator's `ItemLocationData`. Returns `None` when the slot is
/// missing/nil — `CloseAzeriteEmpoweredItemRespec` calls in via
/// `UIPanelWindows.showFailedFunc` with no argument.
fn read_optional_location(state: &mut LuaState, slot: i32) -> Option<ItemLocationData> {
    let Ok(Val::Table(table_ref)) = Val::from_stack(state, slot) else {
        return None;
    };
    let bag_id_key = state.gc.intern_string_static(b"bagID");
    let slot_index_key = state.gc.intern_string_static(b"slotIndex");
    let equipment_slot_key = state.gc.intern_string_static(b"equipmentSlotIndex");
    let table = state.gc.tables.get(table_ref)?;
    let arena = &state.gc.string_arena;
    Some(ItemLocationData {
        bag_id: read_optional_int(table, bag_id_key, arena),
        slot_index: read_optional_int(table, slot_index_key, arena),
        equipment_slot_index: read_optional_int(table, equipment_slot_key, arena),
    })
}

/// Three shapes the respec UI can hand us — the test-friendly
/// `{itemID}` short form, plus the canonical `ItemLocationMixin` bag and
/// equipment-slot variants.
struct LocationFields {
    item_id: Option<i32>,
    bag_id: Option<i32>,
    slot_index: Option<i32>,
    equipment_slot: Option<i32>,
}

/// Resolve an `itemLocation` arg to a concrete item id. Tries
/// `itemID` (test convenience shape) first, then `bagID`/`slotIndex`
/// against `state.bag_items`, then `equipmentSlotIndex` against
/// `player.equipped_items`. Returns `None` when no shape matches —
/// `IsAzeriteEmpoweredItem` then reports false, which mirrors how the
/// addon treats an unbound cursor.
fn resolve_location_item_id(state: &mut LuaState, slot: i32) -> Option<i32> {
    let fields = read_location_fields(state, slot)?;
    if let Some(id) = fields.item_id {
        return Some(id);
    }
    let sim = borrow_state(state).ok()?;
    if let (Some(bag), Some(idx)) = (fields.bag_id, fields.slot_index)
        && let Some((item_id, _)) = sim.get_bag_item(bag, idx)
    {
        return Some(item_id as i32);
    }
    let slot_idx = fields.equipment_slot?;
    sim.player
        .equipped_items
        .get(&slot_idx)
        .map(|item| item.item_id as i32)
}

fn read_location_fields(state: &mut LuaState, slot: i32) -> Option<LocationFields> {
    let Ok(Val::Table(table_ref)) = Val::from_stack(state, slot) else {
        return None;
    };
    let item_id_key = state.gc.intern_string_static(b"itemID");
    let bag_id_key = state.gc.intern_string_static(b"bagID");
    let slot_index_key = state.gc.intern_string_static(b"slotIndex");
    let equipment_slot_key = state.gc.intern_string_static(b"equipmentSlotIndex");
    let table = state.gc.tables.get(table_ref)?;
    let arena = &state.gc.string_arena;
    let item_id = match table.get_str(item_id_key, arena) {
        Val::Num(n) if n > 0.0 => Some(n as i32),
        _ => None,
    };
    Some(LocationFields {
        item_id,
        bag_id: read_optional_int(table, bag_id_key, arena),
        slot_index: read_optional_int(table, slot_index_key, arena),
        equipment_slot: read_optional_int(table, equipment_slot_key, arena),
    })
}

fn read_optional_int(
    table: &Table,
    key: rilua::vm::gc::arena::GcRef<rilua::vm::string::LuaString>,
    arena: &rilua::vm::gc::arena::Arena<rilua::vm::string::LuaString>,
) -> Option<i32> {
    match table.get_str(key, arena) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}
