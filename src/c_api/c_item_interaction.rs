//! Host-provided item-interaction frame data and pending-item lifecycle.
use super::helpers::ensure_namespace;
use crate::c_api::item_location::{ItemSlot, read_r88s_item_slot};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::state::{CursorInfo, CursorItemOrigin, ItemLocationData};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
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

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(builder);
    }
    builder.set_function("SetItemInteractionScenario", set_item_interaction_scenario)
}

fn set_item_interaction_scenario(state: &mut LuaState) -> LuaResult<u32> {
    let interaction_type = read_i32(stack_val(state, 1), "interactionType")?;
    let accepted_item_ids = read_u32_array(state, 2)?;
    let selection_id = read_i32(stack_val(state, 3), "selectionID")?;
    let record_id = read_u32(stack_val(state, 4), "recordID")?;
    let mut sim = borrow_state_mut(state)?;
    sim.item_interaction = Some(ItemInteractionInfo {
        interaction_type,
        ..ItemInteractionInfo::default()
    });
    sim.item_interaction_tooltip_input = Some(crate::c_api::ItemInteractionTooltipInput {
        pending_item_acceptance: accepted_item_ids
            .into_iter()
            .map(|item_id| ((interaction_type, item_id), true))
            .collect(),
        selection_id,
        record_id,
        ..crate::c_api::ItemInteractionTooltipInput::default()
    });
    Ok(0)
}

fn read_u32_array(state: &LuaState, position: i32) -> LuaResult<Vec<u32>> {
    let Val::Table(reference) = stack_val(state, position) else {
        return Err(rilua::runtime_error(
            "A_Admin.SetItemInteractionScenario: acceptedItemIDs must be a table",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| rilua::runtime_error("acceptedItemIDs table is unavailable"))?;
    table
        .array_slice()
        .iter()
        .map(|value| read_u32(*value, "acceptedItemIDs"))
        .collect()
}

fn read_u32(value: Val, field: &str) -> LuaResult<u32> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            Ok(value as u32)
        }
        _ => Err(rilua::runtime_error(format!(
            "A_Admin.SetItemInteractionScenario: {field} must contain unsigned integers"
        ))),
    }
}

fn read_i32(value: Val, field: &str) -> LuaResult<i32> {
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&value) =>
        {
            Ok(value as i32)
        }
        _ => Err(rilua::runtime_error(format!(
            "A_Admin.SetItemInteractionScenario: {field} must be a signed integer"
        ))),
    }
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ItemInteraction")?;
    table_set_rust_fn_static(state, namespace, "GetItemInteractionInfo", get_info)?;
    register_pending_item_lifecycle(state, namespace)
}

fn register_pending_item_lifecycle(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
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
    matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    )
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
    let candidate = find_or_assign_pending_item(state, location.clone())?;
    let success = update_pending_selection(state, candidate, location)?;
    state.push(Val::Bool(success));
    Ok(1)
}

fn find_or_assign_pending_item(
    state: &mut LuaState,
    location: Val,
) -> LuaResult<Option<PendingItem>> {
    if location.is_nil() {
        return Ok(None);
    }
    let slot = read_r88s_item_slot(state, location.clone())
        .map_err(|_| rilua::runtime_error(SET_PENDING_ITEM_USAGE))?;
    let Some((item_id, from_cursor)) = pending_item_at_location(state, slot)? else {
        return Err(rilua::runtime_error(SET_PENDING_ITEM_USAGE));
    };
    let identity = ensure_item_identity(state, slot, from_cursor)?
        .ok_or_else(|| rilua::runtime_error(SET_PENDING_ITEM_USAGE))?;
    Ok(Some(pending_item(item_id, slot, identity)))
}

fn pending_item_at_location(state: &LuaState, slot: ItemSlot) -> LuaResult<Option<(u32, bool)>> {
    if cursor_origin_matches(state, slot)? {
        let sim = borrow_state(state)?;
        let Some(CursorInfo::Item { item_id, .. }) = sim.cursor_item.as_ref() else {
            return Ok(None);
        };
        return Ok((*item_id != 0).then_some((*item_id, true)));
    }
    let sim = borrow_state(state)?;
    Ok(slot
        .item_id(&sim)
        .filter(|item_id| *item_id != 0)
        .map(|item_id| (item_id, false)))
}

fn ensure_item_identity(
    state: &mut LuaState,
    slot: ItemSlot,
    from_cursor: bool,
) -> LuaResult<Option<u64>> {
    let mut sim = borrow_state_mut(state)?;
    Ok(if from_cursor {
        crate::c_api::item_spell::item_equip::ensure_cursor_identity(&mut sim)
    } else {
        crate::c_api::item_spell::item_equip::ensure_identity(&mut sim, slot)
    })
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
    if !host_observes_item_acceptance(state, kind, candidate.item_id)? {
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

fn host_observes_item_acceptance(state: &LuaState, kind: i32, item_id: u32) -> LuaResult<bool> {
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
