//! Owned-item state for Retail equip requests (WM44-B, WM57).

use std::collections::HashMap;

use crate::c_api::item_upgrade_eligibility::ItemSlot;
use crate::lua_api::state::{CursorInfo, SimState};

/// Native flags and locally retained lock reasons for a single owned item.
#[derive(Debug, Clone, Default)]
pub struct OwnedItemEquipInputs {
    pub native_flags: u32,
    pub lock_reasons: u32,
    pub bound_marker: bool,
    pub binding_enchantment: bool,
    pub species_flags: u32,
    /// Native refund timer value before the 7200-second comparison.
    pub refund_timer_delta: Option<i64>,
    pub refund_disabled: bool,
    /// Native trade timer value before the 14400-second comparison.
    pub trade_timer_delta: Option<i64>,
    pub trade_window: bool,
    pub confirmation_eligible: Option<bool>,
    pub account_conversion: bool,
}

/// A locally submitted equip request; no server success is implied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipRequest {
    pub identity: u64,
    pub item_id: u32,
    pub destination: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSource {
    Bag(i32, i32),
    Equipment(i32),
}

impl From<ItemSlot> for EquipSource {
    fn from(slot: ItemSlot) -> Self {
        match slot {
            ItemSlot::Bag(bag, index) => Self::Bag(bag, index),
            ItemSlot::Equipment(index) => Self::Equipment(index),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipConfirmation {
    Refundable,
    Tradeable,
    Bind,
    AccountConversion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingEquip {
    pub identity: u64,
    pub item_id: u32,
    pub source: EquipSource,
    pub destination: Option<i32>,
    pub confirmation: EquipConfirmation,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StagedEquip {
    pub identity: u64,
    pub item_id: u32,
    pub source: ItemSlot,
}

#[derive(Debug)]
pub struct ItemEquipState {
    next_identity: u64,
    pub inputs: HashMap<u64, OwnedItemEquipInputs>,
    pub requests: Vec<EquipRequest>,
    pub pending_confirmation: Option<PendingEquip>,
    pub(crate) staged: Option<StagedEquip>,
    /// UI initialization enables pickup control (WM57 Q7).
    pub pickup_control_enabled: bool,
}

impl Default for ItemEquipState {
    fn default() -> Self {
        Self {
            next_identity: 0,
            inputs: HashMap::new(),
            requests: Vec::new(),
            pending_confirmation: None,
            staged: None,
            pickup_control_enabled: true,
        }
    }
}

const NATIVE_LOCK_FLAG: u32 = 2048;
pub(crate) const EQUIP_REQUEST_REASON: u32 = 1;

impl ItemEquipState {
    pub fn is_locked(&self, identity: Option<u64>) -> bool {
        identity
            .and_then(|id| self.inputs.get(&id))
            .is_some_and(|item| item.native_flags & NATIVE_LOCK_FLAG != 0 || item.lock_reasons != 0)
    }
}

pub(crate) fn identity_at(sim: &SimState, slot: ItemSlot) -> Option<u64> {
    match slot {
        ItemSlot::Bag(bag, index) => sim.bag_items.get(&(bag, index))?.owned_identity,
        ItemSlot::Equipment(index) => sim
            .player
            .equipped_items
            .get(&index)
            .and_then(|item| item.owned_identity)
            .or_else(|| {
                sim.bag_info
                    .values()
                    .find(|bag| bag.inventory_slot == Some(index) && bag.item_id.is_some())
                    .and_then(|bag| bag.owned_identity)
            }),
    }
}

/// Assign a session-local identity without reusing an explicitly supplied one.
pub(crate) fn ensure_identity(sim: &mut SimState, slot: ItemSlot) -> Option<u64> {
    if let Some(id) = identity_at(sim, slot) {
        return Some(id);
    }
    let cursor_id = match sim.cursor_item.as_ref() {
        Some(CursorInfo::Item { owned_identity, .. }) => *owned_identity,
        _ => None,
    };
    let maximum = sim
        .bag_items
        .values()
        .filter_map(|item| item.owned_identity)
        .chain(
            sim.player
                .equipped_items
                .values()
                .filter_map(|item| item.owned_identity),
        )
        .chain(sim.bag_info.values().filter_map(|bag| bag.owned_identity))
        .chain(sim.item_equip.inputs.keys().copied())
        .chain(cursor_id)
        .chain([sim.item_equip.next_identity])
        .max()?;
    let id = maximum.checked_add(1)?;
    match slot {
        ItemSlot::Bag(bag, index) => {
            sim.bag_items.get_mut(&(bag, index))?.owned_identity = Some(id)
        }
        ItemSlot::Equipment(index) => {
            if let Some(item) = sim.player.equipped_items.get_mut(&index) {
                item.owned_identity = Some(id);
            } else {
                let bag = sim
                    .bag_info
                    .values_mut()
                    .find(|bag| bag.inventory_slot == Some(index) && bag.item_id.is_some())?;
                bag.owned_identity = Some(id);
            }
        }
    }
    sim.item_equip.next_identity = id;
    Some(id)
}

/// Resolve the current location of a simulator-owned identity.
pub(crate) fn find_identity_slot(sim: &SimState, identity: u64) -> Option<ItemSlot> {
    sim.bag_items
        .iter()
        .find_map(|(&(bag, slot), item)| {
            (item.owned_identity == Some(identity)).then_some(ItemSlot::Bag(bag, slot))
        })
        .or_else(|| {
            sim.player.equipped_items.iter().find_map(|(&slot, item)| {
                (item.owned_identity == Some(identity)).then_some(ItemSlot::Equipment(slot))
            })
        })
        .or_else(|| {
            sim.bag_info.values().find_map(|bag| {
                (bag.owned_identity == Some(identity))
                    .then_some(bag.inventory_slot)
                    .flatten()
                    .map(ItemSlot::Equipment)
            })
        })
}

/// Publish an effective lock transition for a resolved item location.
pub(crate) fn dispatch_lock_transition(
    state: &mut rilua::vm::state::LuaState,
    slot: ItemSlot,
    locked: bool,
) -> rilua::LuaResult<()> {
    use rilua::Val;
    let args = match slot {
        ItemSlot::Bag(bag, index) => [Val::Num(f64::from(bag)), Val::Num(f64::from(index))],
        ItemSlot::Equipment(index) => [Val::Num(f64::from(index)), Val::Nil],
    };
    let dispatch = crate::lua_api::globals::state_backed_queries::dispatch_event_now;
    dispatch(state, "ITEM_LOCK_CHANGED", &args)?;
    dispatch(
        state,
        if locked {
            "ITEM_LOCKED"
        } else {
            "ITEM_UNLOCKED"
        },
        &args,
    )
}

/// Remove the equip reason when a staged cursor is restored.
pub(crate) fn restore_staged_lock(sim: &mut SimState) -> Option<ItemSlot> {
    let staged = sim.item_equip.staged.take()?;
    let slot = find_identity_slot(sim, staged.identity)?;
    let was_locked = is_locked_at(sim, slot);
    if let Some(item) = sim.item_equip.inputs.get_mut(&staged.identity) {
        item.lock_reasons &= !EQUIP_REQUEST_REASON;
    }
    (was_locked && !is_locked_at(sim, slot)).then_some(slot)
}

/// Effective item lock from identity-owned reasons and the keystone model.
pub(crate) fn is_locked_at(sim: &SimState, slot: ItemSlot) -> bool {
    if slot.item_id(sim).is_none() {
        return false;
    }
    let keystone_slot = match slot {
        ItemSlot::Bag(bag, index) => {
            crate::c_api::c_challenge_mode::KeystoneLocation::Bag(bag, index)
        }
        ItemSlot::Equipment(index) => {
            crate::c_api::c_challenge_mode::KeystoneLocation::Equipment(index)
        }
    };
    sim.item_equip.is_locked(identity_at(sim, slot)) || sim.keystone.is_locked(keystone_slot)
}

pub(super) fn is_locked(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let location = crate::lua_bridge::stack_val(state, 1);
    let location = rilua::table_security::unwrap_secret(state, location)?;
    let slot = crate::c_api::item_upgrade_eligibility::read_item_slot(state, location)
        .ok_or_else(|| rilua::runtime_error("C_Item.IsLocked: expected item location"))?;
    let locked = is_locked_at(&*crate::lua_api::methods::borrow_state(state)?, slot);
    state.push(rilua::Val::Bool(locked));
    Ok(1)
}

/// Selector 14 searches the backpack and carried bags first. Selector 0 then
/// admits player root slots 0–29 (WM57). Each bag precedes its contents.
fn find_owned_item(sim: &SimState, item_id: u32) -> Option<ItemSlot> {
    for bag in 0..=5 {
        if bag != 0 {
            if let Some(info) = sim.bag_info.get(&bag) {
                if info.item_id == Some(item_id) {
                    if let Some(slot) = info.inventory_slot {
                        return Some(ItemSlot::Equipment(slot));
                    }
                }
            }
        }
        let found = sim
            .bag_items
            .iter()
            .filter_map(|(&(bag_id, slot), item)| {
                (bag_id == bag
                    && slot > 0
                    && slot <= sim.bag_num_slots(bag)
                    && item.item_id == item_id
                    && item.stack_count > 0)
                    .then_some(slot)
            })
            .min();
        if let Some(slot) = found {
            return Some(ItemSlot::Bag(bag, slot));
        }
    }
    sim.player
        .equipped_items
        .iter()
        .filter_map(|(&slot, item)| {
            ((1..=30).contains(&slot) && item.item_id == item_id).then_some(slot)
        })
        .min()
        .map(ItemSlot::Equipment)
}

fn read_destination(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<Option<i32>> {
    use rilua::Val;
    let value =
        rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, 2))?;
    match value {
        Val::Nil => Ok(None),
        Val::Num(number) if number.is_finite() => {
            let slot = (number - 1.0).trunc();
            if (0.0..=18.0).contains(&slot) {
                Ok(Some(slot as i32 + 1))
            } else {
                Err(rilua::runtime_error(
                    "C_Item.EquipItemByName: invalid destination slot",
                ))
            }
        }
        _ => Err(rilua::runtime_error(
            "C_Item.EquipItemByName: expected destination slot",
        )),
    }
}

fn cursor_for_item(sim: &SimState, staged: StagedEquip) -> Option<CursorInfo> {
    let (stack_count, applied_transmog) = match staged.source {
        ItemSlot::Bag(bag, slot) => {
            let item = sim.bag_items.get(&(bag, slot))?;
            (item.stack_count, item.applied_transmog)
        }
        ItemSlot::Equipment(slot) => (
            1,
            sim.player
                .equipped_items
                .get(&slot)
                .and_then(|item| item.applied_transmog),
        ),
    };
    Some(CursorInfo::Item {
        owned_identity: Some(staged.identity),
        item_id: staged.item_id,
        stack_count,
        origin: staged.source.cursor_origin(),
        applied_transmog,
    })
}

fn stage_item(state: &mut rilua::vm::state::LuaState, staged: StagedEquip) -> rilua::LuaResult<()> {
    use crate::lua_api::globals::real::cursor_model;
    use crate::lua_api::methods::borrow_state;
    cursor_model::clear(state)?;
    let cursor = {
        let sim = borrow_state(state)?;
        if staged.source.item_id(&sim) != Some(staged.item_id)
            || identity_at(&sim, staged.source) != Some(staged.identity)
        {
            return Ok(());
        }
        cursor_for_item(&sim, staged)
    };
    if let Some(cursor) = cursor {
        cursor_model::place_staged_after_clear(state, cursor, staged)?;
    }
    Ok(())
}

fn lock_request_item(
    state: &mut rilua::vm::state::LuaState,
    identity: u64,
) -> rilua::LuaResult<bool> {
    let transition = {
        let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
        let Some(slot) = find_identity_slot(&sim, identity) else {
            return Ok(false);
        };
        let was_locked = is_locked_at(&sim, slot);
        sim.item_equip
            .inputs
            .entry(identity)
            .or_default()
            .lock_reasons |= EQUIP_REQUEST_REASON;
        (!was_locked && is_locked_at(&sim, slot)).then_some(slot)
    };
    if let Some(slot) = transition {
        dispatch_lock_transition(state, slot, true)?;
    }
    Ok(true)
}

fn submit_request(
    state: &mut rilua::vm::state::LuaState,
    staged: StagedEquip,
    destination: Option<i32>,
) -> rilua::LuaResult<()> {
    if !lock_request_item(state, staged.identity)? {
        return Ok(());
    }
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    sim.item_equip.requests.push(EquipRequest {
        identity: staged.identity,
        item_id: staged.item_id,
        destination,
    });
    Ok(())
}

fn confirmation_for_item(
    inputs: &OwnedItemEquipInputs,
    bonding: u8,
) -> rilua::LuaResult<Option<EquipConfirmation>> {
    const BINDING_ENCHANT_EXCLUDED: u32 = 8192;
    const SPECIES_BOUND: u32 = 16;
    let binding_enchantment =
        inputs.binding_enchantment && inputs.native_flags & BINDING_ENCHANT_EXCLUDED == 0;
    let bound =
        inputs.bound_marker || binding_enchantment || inputs.species_flags & SPECIES_BOUND != 0;
    let refundable = !inputs.refund_disabled
        && inputs
            .refund_timer_delta
            .is_some_and(|delta| delta.saturating_add(7200) > 0);
    if refundable {
        return Ok(Some(EquipConfirmation::Refundable));
    }
    let tradeable = bound
        && inputs.trade_window
        && !binding_enchantment
        && inputs
            .trade_timer_delta
            .is_some_and(|delta| delta.saturating_add(14400) > 0);
    if bonding != 2 && !tradeable {
        return Ok(None);
    }
    let eligible = inputs.confirmation_eligible.ok_or_else(|| {
        rilua::runtime_error("C_Item.EquipItemByName: confirmation eligibility not supplied")
    })?;
    Ok(eligible.then_some(if tradeable {
        EquipConfirmation::Tradeable
    } else {
        EquipConfirmation::Bind
    }))
}

fn confirmation_for_staged(
    sim: &SimState,
    staged: StagedEquip,
) -> rilua::LuaResult<Option<EquipConfirmation>> {
    let Some(bonding) = super::item_spec_derive::item_bonding(staged.item_id) else {
        return Ok(None);
    };
    let default_inputs = OwnedItemEquipInputs::default();
    let inputs = sim
        .item_equip
        .inputs
        .get(&staged.identity)
        .unwrap_or(&default_inputs);
    confirmation_for_item(inputs, bonding)
}

fn reject_unimplemented_confirmation(confirmation: EquipConfirmation) -> rilua::LuaResult<()> {
    Err(rilua::runtime_error(format!(
        "C_Item.EquipItemByName: {confirmation:?} confirmation payload is not established"
    )))
}

fn process_automatic(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<()> {
    let staged = crate::lua_api::methods::borrow_state(state)?
        .item_equip
        .staged;
    let Some(staged) = staged else {
        return Ok(());
    };
    if matches!(staged.source, ItemSlot::Equipment(20..=35)) {
        return crate::lua_api::globals::real::cursor_model::clear(state);
    }
    {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        if find_identity_slot(&sim, staged.identity).is_none() {
            return Ok(());
        }
    }
    let Some(class) = super::item_spec_derive::item_class(staged.item_id) else {
        return Ok(());
    };
    let (confirmation, conversion) = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        let confirmation = confirmation_for_staged(&sim, staged)?;
        let conversion = sim
            .item_equip
            .inputs
            .get(&staged.identity)
            .is_some_and(|inputs| inputs.account_conversion);
        (confirmation, conversion)
    };
    if let Some(confirmation) = confirmation {
        return reject_unimplemented_confirmation(confirmation);
    }
    if conversion {
        if lock_request_item(state, staged.identity)? {
            let pending = PendingEquip {
                identity: staged.identity,
                item_id: staged.item_id,
                source: staged.source.into(),
                destination: None,
                confirmation: EquipConfirmation::AccountConversion,
            };
            crate::lua_api::methods::borrow_state_mut(state)?
                .item_equip
                .pending_confirmation = Some(pending);
            crate::lua_api::globals::state_backed_queries::dispatch_event_now(
                state,
                "CONVERT_TO_BIND_TO_ACCOUNT_CONFIRM",
                &[],
            )?;
        }
        return Ok(());
    }
    if class == 6 {
        return crate::lua_api::globals::real::cursor_model::clear(state);
    }
    submit_request(state, staged, None)?;
    crate::lua_api::globals::real::cursor_model::clear_retaining_equip_lock(state)
}

/// The submitted request records local state only; no server response is synthesized.
pub(super) fn equip_item_by_name(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    let item = rilua::table_security::unwrap_secret(state, crate::lua_bridge::stack_val(state, 1))?;
    let item_id = super::parse_item_id_key(state, item)
        .ok_or_else(|| rilua::runtime_error("C_Item.EquipItemByName: expected itemInfo"))?;
    let destination = read_destination(state)?;
    let staged = {
        let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
        if !sim.player.present {
            return Ok(0);
        }
        let Some(source) = find_owned_item(&sim, item_id) else {
            return Ok(0);
        };
        if destination.is_none() && matches!(source, ItemSlot::Equipment(1..=19)) {
            return Ok(0);
        }
        if destination.is_some_and(|slot| source == ItemSlot::Equipment(slot)) {
            return Ok(0);
        }
        if is_locked_at(&sim, source) {
            return Ok(0);
        }
        let Some(identity) = ensure_identity(&mut sim, source) else {
            return Ok(0);
        };
        StagedEquip {
            identity,
            item_id,
            source,
        }
    };
    if let Some(destination) = destination {
        if super::item_spec_derive::item_class(item_id).is_some() {
            let confirmation = {
                let sim = crate::lua_api::methods::borrow_state(state)?;
                confirmation_for_staged(&sim, staged)?
            };
            if let Some(confirmation) = confirmation {
                reject_unimplemented_confirmation(confirmation)?;
            }
            submit_request(state, staged, Some(destination))?;
        }
        return Ok(0);
    }
    let can_stage = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        sim.item_equip.pickup_control_enabled || sim.player.on_taxi
    };
    if can_stage {
        stage_item(state, staged)?;
    }
    process_automatic(state)?;
    Ok(0)
}
