//! Host-provided item-interaction frame data and pending-item lifecycle.
use super::helpers::ensure_namespace;
use crate::c_api::item_upgrade_eligibility::ItemSlot;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin, ItemLocationData};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const SET_PENDING_ITEM_USAGE: &str =
    "Usage: local success = C_ItemInteraction.SetPendingItem([item])";
const ITEM_INTERACTION_CLOSE_SELECTOR: i32 = 44;
// INFERRED (simulator representation): this dedicated bit is not a native reason value.
const ITEM_INTERACTION_SELECTION_LOCK_REASON: u32 = 2;

#[derive(Clone, Debug, Default)]
pub struct ItemInteractionInfo {
    pub texture_kit: String,
    pub open_sound_kit_id: i32,
    pub close_sound_kit_id: i32,
    pub title_text: String,
    pub tutorial_text: String,
    pub button_text: String,
    pub interaction_type: i32,
    pub flags: i32,
    pub description: Option<String>,
    pub button_tooltip: Option<String>,
    pub confirmation_description: Option<String>,
    pub slot_tooltip: Option<String>,
    pub cost: Option<f64>,
    pub currency_type_id: Option<i32>,
    pub drop_in_slot_sound_kit_id: Option<i32>,
}

#[derive(Clone, Debug)]
struct PendingItem {
    item_id: u32,
    slot: ItemSlot,
    identity: u64,
    guid: Option<String>,
    location: ItemLocationData,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ItemInteraction")?;
    table_set_rust_fn_static(state, namespace, "GetItemInteractionInfo", get_info)?;
    register_pending_item_lifecycle(state, namespace)
}

fn register_pending_item_lifecycle(state: &mut LuaState, namespace: Val) -> LuaResult<()> {
    if !pending_item_lifecycle_available() {
        return Ok(());
    }
    // INFERRED publication pattern: R88o.2 q4 does not state registration flags.
    table_set_rust_fn_static(state, namespace, "SetPendingItem", set_pending_item)?;
    table_set_rust_fn_static(state, namespace, "ClearPendingItem", clear_pending_item)?;
    table_set_rust_fn_static(state, namespace, "InitializeFrame", initialize_frame)?;
    table_set_rust_fn_static(state, namespace, "Reset", reset)?;
    table_set_rust_fn_static(state, namespace, "CloseUI", close_ui)
}

fn pending_item_lifecycle_available() -> bool {
    ACTIVE == ClientProfile::Retail
}

fn get_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(info) = borrow_state(state)?.item_interaction.clone() else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let table = create_table(state);
    state.push(table);
    publish_text(state, table, &info);
    publish_numbers(state, table, &info);
    Ok(1)
}

fn publish_text(state: &mut LuaState, table: Val, info: &ItemInteractionInfo) {
    for (key, value) in [
        ("textureKit", Some(&info.texture_kit)),
        ("titleText", Some(&info.title_text)),
        ("tutorialText", Some(&info.tutorial_text)),
        ("buttonText", Some(&info.button_text)),
        ("description", info.description.as_ref()),
        ("buttonTooltip", info.button_tooltip.as_ref()),
        (
            "confirmationDescription",
            info.confirmation_description.as_ref(),
        ),
        ("slotTooltip", info.slot_tooltip.as_ref()),
    ] {
        if let Some(value) = value {
            let value = create_string(state, value);
            table_set(state, table, key, value);
        }
    }
}

fn publish_numbers(state: &mut LuaState, table: Val, info: &ItemInteractionInfo) {
    for (key, value) in [
        ("openSoundKitID", Some(f64::from(info.open_sound_kit_id))),
        ("closeSoundKitID", Some(f64::from(info.close_sound_kit_id))),
        ("interactionType", Some(f64::from(info.interaction_type))),
        ("flags", Some(f64::from(info.flags))),
        ("cost", info.cost),
        ("currencyTypeId", info.currency_type_id.map(f64::from)),
        (
            "dropInSlotSoundKitId",
            info.drop_in_slot_sound_kit_id.map(f64::from),
        ),
    ] {
        if let Some(value) = value {
            table_set(state, table, key, Val::Num(value));
        }
    }
}

fn set_pending_item(state: &mut LuaState) -> LuaResult<u32> {
    let location = crate::c_api::tooltip_getter_args::allowed_when_untainted(
        state,
        1,
        "C_ItemInteraction.SetPendingItem",
    )?;
    let candidate = parse_pending_item(state, location.clone())?;
    let success = update_pending_selection(state, candidate, location)?;
    state.push(Val::Bool(success));
    Ok(1)
}

fn parse_pending_item(state: &mut LuaState, location: Val) -> LuaResult<Option<PendingItem>> {
    if location.is_nil() {
        return Ok(None);
    }
    let slot = crate::c_api::item_upgrade_eligibility::read_item_slot(state, location.clone())
        .ok_or_else(|| rilua::runtime_error(SET_PENDING_ITEM_USAGE))?;
    let Some((item_id, _)) = crate::c_api::item_spell::location_item(state, location) else {
        return selected_item_at_location(state, slot);
    };
    let identity = ensure_item_identity(state, slot)?;
    Ok(identity.map(|identity| pending_item(item_id, slot, identity)))
}

fn ensure_item_identity(state: &mut LuaState, slot: ItemSlot) -> LuaResult<Option<u64>> {
    if cursor_origin_matches(state, slot)? {
        let mut sim = borrow_state_mut(state)?;
        Ok(crate::c_api::item_spell::item_equip::ensure_cursor_identity(&mut sim))
    } else {
        let mut sim = borrow_state_mut(state)?;
        Ok(crate::c_api::item_spell::item_equip::ensure_identity(
            &mut sim, slot,
        ))
    }
}

fn cursor_origin_matches(state: &LuaState, slot: ItemSlot) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    let Some(CursorInfo::Item { origin, .. }) = sim.cursor_item.as_ref() else {
        return Ok(false);
    };
    Ok(matches!(
        (slot, origin),
        (ItemSlot::Bag(bag, index), CursorItemOrigin::Bag { bag: cursor_bag, slot: cursor_slot })
            if bag == *cursor_bag && index == *cursor_slot
    ) || matches!(
        (slot, origin),
        (ItemSlot::Equipment(index), CursorItemOrigin::Equipped { slot: cursor_slot })
            if index == *cursor_slot
    ))
}

fn selected_item_at_location(state: &LuaState, slot: ItemSlot) -> LuaResult<Option<PendingItem>> {
    let sim = borrow_state(state)?;
    let Some(input) = sim.item_interaction_tooltip_input.as_ref() else {
        return Ok(None);
    };
    let (Some(item_id), Some(identity), Some(location)) = (
        input.selected_item_id,
        input.selected_item_identity,
        input.selected_item_location.as_ref(),
    ) else {
        return Ok(None);
    };
    if item_slot_from_location(location) != Some(slot) {
        return Ok(None);
    }
    Ok(Some(PendingItem {
        item_id,
        slot,
        identity,
        guid: input.selected_item_guid.clone(),
        location: location.clone(),
    }))
}

fn pending_item(item_id: u32, slot: ItemSlot, identity: u64) -> PendingItem {
    let guid = match slot {
        ItemSlot::Bag(bag, index) => Some(crate::c_api::item_spell::item_guid_for_bag_slot(
            bag, index, item_id,
        )),
        ItemSlot::Equipment(_) => None,
    };
    PendingItem {
        item_id,
        slot,
        identity,
        guid,
        location: item_location_data(slot),
    }
}

fn item_location_data(slot: ItemSlot) -> ItemLocationData {
    match slot {
        ItemSlot::Bag(bag, index) => ItemLocationData {
            bag_id: Some(bag),
            slot_index: Some(index),
            equipment_slot_index: None,
        },
        ItemSlot::Equipment(index) => ItemLocationData {
            bag_id: None,
            slot_index: None,
            equipment_slot_index: Some(index),
        },
    }
}

fn item_slot_from_location(location: &ItemLocationData) -> Option<ItemSlot> {
    match (
        location.bag_id,
        location.slot_index,
        location.equipment_slot_index,
    ) {
        (Some(bag), Some(index), None) => Some(ItemSlot::Bag(bag, index)),
        (None, None, Some(index)) => Some(ItemSlot::Equipment(index)),
        _ => None,
    }
}

fn update_pending_selection(
    state: &mut LuaState,
    candidate: Option<PendingItem>,
    event_location: Val,
) -> LuaResult<bool> {
    if let Some(candidate) = candidate.as_ref()
        && is_selected_item(state, candidate)?
    {
        return Ok(true);
    }
    let previous = selected_pending_item(state)?;
    if let Some(previous) = previous.as_ref() {
        set_selection_lock(state, previous, false)?;
    }
    clear_selected_object(state)?;
    let Some(candidate) = candidate else {
        fire_selection_updated(state, Val::Nil)?;
        return Ok(false);
    };
    let kind = current_interaction_kind(state)?;
    if !host_accepts_item(state, kind, candidate.item_id)? {
        fire_selection_updated(state, Val::Nil)?;
        return Ok(false);
    }
    store_selected_object(state, kind, &candidate)?;
    set_selection_lock(state, &candidate, true)?;
    fire_selection_updated(state, event_location)?;
    crate::lua_api::globals::inventory_verbs::clear_cursor_payload(state)?;
    Ok(true)
}

fn is_selected_item(state: &LuaState, item: &PendingItem) -> LuaResult<bool> {
    Ok(borrow_state(state)?
        .item_interaction_tooltip_input
        .as_ref()
        .and_then(|input| input.selected_item_identity)
        == Some(item.identity))
}

fn selected_pending_item(state: &LuaState) -> LuaResult<Option<PendingItem>> {
    let Some(input) = borrow_state(state)?.item_interaction_tooltip_input.clone() else {
        return Ok(None);
    };
    let (Some(item_id), Some(identity), Some(location)) = (
        input.selected_item_id,
        input.selected_item_identity,
        input.selected_item_location,
    ) else {
        return Ok(None);
    };
    let Some(slot) = item_slot_from_location(&location) else {
        return Ok(None);
    };
    Ok(Some(PendingItem {
        item_id,
        slot,
        identity,
        guid: input.selected_item_guid,
        location,
    }))
}

fn set_selection_lock(state: &mut LuaState, item: &PendingItem, locked: bool) -> LuaResult<()> {
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        crate::c_api::item_spell::item_equip::set_lock_reason(
            &mut sim,
            item.slot,
            item.identity,
            ITEM_INTERACTION_SELECTION_LOCK_REASON,
            locked,
        )
    };
    if changed {
        crate::c_api::item_spell::item_equip::dispatch_lock_transition(state, item.slot, locked)?;
    }
    Ok(())
}

fn clear_selected_object(state: &mut LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if let Some(input) = sim.item_interaction_tooltip_input.as_mut() {
        clear_selected_object_fields(input);
    }
    Ok(())
}

fn clear_selected_object_fields(input: &mut crate::c_api::ItemInteractionTooltipInput) {
    input.object_kind = None;
    input.selected_item_guid = None;
    input.selected_item_id = None;
    input.selected_item_location = None;
    input.selected_item_identity = None;
}

fn store_selected_object(state: &mut LuaState, kind: i32, item: &PendingItem) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let input = sim
        .item_interaction_tooltip_input
        .get_or_insert_with(Default::default);
    input.object_kind = Some(kind);
    input.selected_item_guid.clone_from(&item.guid);
    input.selected_item_id = Some(item.item_id);
    input.selected_item_location = Some(item.location.clone());
    input.selected_item_identity = Some(item.identity);
    Ok(())
}

fn current_interaction_kind(state: &LuaState) -> LuaResult<i32> {
    Ok(borrow_state(state)?
        .item_interaction
        .as_ref()
        .map_or(0, |info| info.interaction_type))
}

fn host_accepts_item(state: &LuaState, kind: i32, item_id: u32) -> LuaResult<bool> {
    if !matches!(kind, 2 | 3 | 4) {
        return Ok(false);
    }
    // UNKNOWN: the ItemModified, RelatedKey and ItemConversion predicate tables
    // are not modeled; absent host observations reject without fabricated rows.
    Ok(borrow_state(state)?
        .item_interaction_tooltip_input
        .as_ref()
        .and_then(|input| input.pending_item_acceptance.get(&(kind, item_id)))
        .copied()
        .unwrap_or(false))
}

fn fire_selection_updated(state: &mut LuaState, object: Val) -> LuaResult<()> {
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "ITEM_INTERACTION_ITEM_SELECTION_UPDATED",
        &[object],
    )
}

fn clear_pending_item(state: &mut LuaState) -> LuaResult<u32> {
    update_pending_selection(state, None, Val::Nil)?;
    Ok(0)
}

fn initialize_frame(state: &mut LuaState) -> LuaResult<u32> {
    clear_selected_object(state)?;
    Ok(0)
}

fn reset(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    let input = sim
        .item_interaction_tooltip_input
        .get_or_insert_with(Default::default);
    clear_selected_object_fields(input);
    input.selection_id = -1;
    input.record_id = 0;
    Ok(0)
}

fn close_ui(state: &mut LuaState) -> LuaResult<u32> {
    crate::c_api::c_player_interaction_manager::clear_interaction_type(
        state,
        ITEM_INTERACTION_CLOSE_SELECTOR,
    )?;
    Ok(0)
}
