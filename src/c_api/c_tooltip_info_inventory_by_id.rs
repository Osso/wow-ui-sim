//! Round 6 §GetInventoryItemByID and WM94 §1: the getter selects an owned
//! instance and composes the ordinary item tooltip from it.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:407` declare the getter.
//! Native evidence is Retail 12.1.5 (70688); Forever 1.60.1 (70707) shows the
//! same shape.
//!
//! VERIFIED: the argument is read by the plain signed-integer reader, and the
//! getter applies no semantic item-ID bound, so zero and negative ids pass the
//! reader and then simply fail the inventory search. Selection is one forward
//! pass that keeps the first acceptable candidate: the backpack slot range
//! precedes the equipment range, and a candidate slot that does not match has
//! its child container searched before the pass moves on. The request is built
//! from the found item's ID and its own GUID twice; the shared item producer
//! publishes that GUID as the top-level `guid` field (INFERRED for the field).
//! The found instance's gem/socket projection is copied before the item enters
//! the builder.
//!
//! The simulator has no flat native inventory-slot array: it stores the
//! backpack, the four carried bags and the equipment slots separately. The
//! native category order is therefore reproduced over those stores as backpack,
//! equipment range, carried bags. The names of the native ranges listed between
//! the backpack range and the equipment range are UNKNOWN, and the bank is not
//! part of the modelled carried inventory, so neither is searched.
//!
//! Explicit host-supplied bag-instance socket rows are copied into the shared
//! producer. Pinned TooltipInfoSharedDocumentation declares GemSocket as line
//! type 3 and the original TooltipDataRules consumer renders `gemIcon` or
//! `socketType`. Native row selection remains UNKNOWN. An equipped copy also has
//! no simulator GUID string, because `C_Item.GetItemGUID` only models bag
//! locations, so only a bag copy publishes `guid`.

use crate::c_api::item_spell::item_guid_for_bag_slot;
use crate::c_api::item_tooltip::ItemSocketProjectionRow;
use crate::c_api::tooltip_data::unknown_getter_failure;
use crate::c_api::tooltip_getter_args::signed;
use crate::items;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context_with_socket_projection;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, table_get, table_set, val_to_string,
};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

/// WM57: the backpack is bag id 0 and the ordinary carried bags are 1-4.
const BACKPACK: i32 = 0;
const FIRST_CARRIED_BAG: i32 = 1;
const LAST_CARRIED_BAG: i32 = 4;

/// The selected copy and the simulator instance identity the request carries.
struct OwnedInstance {
    guid: Option<String>,
    socket_projection: Vec<ItemSocketProjectionRow>,
}

/// Where the selected copy sits in the modelled player inventory.
#[derive(Clone, Copy)]
enum Location {
    Bag { bag: i32, slot: i32 },
    Equipment,
}

impl Location {
    /// The simulator identity, in the shape `C_Item.GetItemGUID` publishes. The
    /// equipment range has no modelled GUID string.
    fn instance(self, item_id: u32, sim: &SimState) -> OwnedInstance {
        let (guid, socket_projection) = match self {
            Location::Bag { bag, slot } => (
                Some(item_guid_for_bag_slot(bag, slot, item_id)),
                sim.item_tooltip_socket_projections
                    .get(&(bag, slot))
                    .cloned()
                    .unwrap_or_default(),
            ),
            Location::Equipment => (None, Vec::new()),
        };
        OwnedInstance {
            guid,
            socket_projection,
        }
    }
}

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "GetInventoryItemByID",
        get_inventory_item_by_id,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(builder);
    }
    builder.set_function(
        "SetItemTooltipSocketProjection",
        set_item_tooltip_socket_projection,
    )
}

fn set_item_tooltip_socket_projection(state: &mut LuaState) -> LuaResult<u32> {
    let bag = read_i32(stack_val(state, 1), "bagID")?;
    let slot = read_i32(stack_val(state, 2), "slotIndex")?;
    if !(0..=4).contains(&bag) || slot < 1 {
        return Err(rilua::runtime_error(
            "A_Admin.SetItemTooltipSocketProjection: invalid bag location",
        ));
    }
    let rows = read_socket_rows(state, stack_val(state, 3))?;
    let mut sim = borrow_state_mut(state)?;
    if !sim.bag_items.contains_key(&(bag, slot)) {
        return Err(rilua::runtime_error(
            "A_Admin.SetItemTooltipSocketProjection: bag item does not exist",
        ));
    }
    if rows.is_empty() {
        sim.item_tooltip_socket_projections.remove(&(bag, slot));
    } else {
        sim.item_tooltip_socket_projections
            .insert((bag, slot), rows);
    }
    Ok(0)
}

fn read_socket_rows(state: &mut LuaState, value: Val) -> LuaResult<Vec<ItemSocketProjectionRow>> {
    if value.is_nil() {
        return Ok(Vec::new());
    }
    let Val::Table(reference) = value else {
        return Err(rilua::runtime_error(
            "A_Admin.SetItemTooltipSocketProjection: rows must be a table or nil",
        ));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| rilua::runtime_error("socket projection table is unavailable"))?;
    let rows = table.array_slice().to_vec();
    rows.iter()
        .map(|row| read_socket_row(state, *row))
        .collect()
}

fn read_socket_row(state: &mut LuaState, value: Val) -> LuaResult<ItemSocketProjectionRow> {
    if !matches!(value, Val::Table(_)) {
        return Err(rilua::runtime_error(
            "socket projection row must be a table",
        ));
    }
    let socket_type = table_get(state, value, "socketType");
    let gem_icon = table_get(state, value, "gemIcon");
    let gem_item_id = table_get(state, value, "gemItemID");
    let row = ItemSocketProjectionRow {
        socket_type: read_optional_text(state, socket_type, "socketType")?,
        gem_icon: read_optional_text(state, gem_icon, "gemIcon")?,
        gem_item_id: read_optional_u32(gem_item_id, "gemItemID")?,
    };
    if row.socket_type.is_none() && row.gem_icon.is_none() && row.gem_item_id.is_none() {
        return Err(rilua::runtime_error(
            "socket projection row requires socketType, gemIcon, or gemItemID",
        ));
    }
    Ok(row)
}

fn read_optional_text(state: &mut LuaState, value: Val, field: &str) -> LuaResult<Option<String>> {
    if value.is_nil() {
        return Ok(None);
    }
    val_to_string(state, value)
        .filter(|text| !text.is_empty())
        .map(Some)
        .ok_or_else(|| {
            rilua::runtime_error(format!(
                "A_Admin.SetItemTooltipSocketProjection: {field} must be a nonempty string"
            ))
        })
}

fn read_optional_u32(value: Val, field: &str) -> LuaResult<Option<u32>> {
    if value.is_nil() {
        return Ok(None);
    }
    match value {
        Val::Num(value)
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            Ok(Some(value as u32))
        }
        _ => Err(rilua::runtime_error(format!(
            "A_Admin.SetItemTooltipSocketProjection: {field} must be an unsigned integer"
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
            "A_Admin.SetItemTooltipSocketProjection: {field} must be a signed integer"
        ))),
    }
}

fn get_inventory_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = signed(state, 1)?;
    let Ok(item_id) = u32::try_from(item_id) else {
        return unknown_getter_failure();
    };
    let Some(instance) = find_instance(state, item_id) else {
        return unknown_getter_failure();
    };
    let Some(item) = items::get_item(item_id) else {
        return unknown_getter_failure();
    };
    let tooltip = tooltip_for_item_context_with_socket_projection(
        state,
        item_id,
        Some(item),
        &instance.socket_projection,
    );
    if let Some(guid) = instance.guid {
        let guid = create_string(state, &guid);
        table_set(state, tooltip, "guid", guid);
    }
    state.push(tooltip);
    Ok(1)
}

/// One forward pass, first acceptable candidate wins: backpack, then the
/// equipment range, then the carried bags.
fn find_instance(state: &LuaState, item_id: u32) -> Option<OwnedInstance> {
    let sim = borrow_state(state).ok()?;
    let location = container_slot(&sim, BACKPACK, item_id)
        .map(|slot| Location::Bag {
            bag: BACKPACK,
            slot,
        })
        .or_else(|| equipment_slot(&sim, item_id).map(|_| Location::Equipment))
        .or_else(|| carried_bag_slot(&sim, item_id))?;
    Some(location.instance(item_id, &sim))
}

fn carried_bag_slot(sim: &SimState, item_id: u32) -> Option<Location> {
    (FIRST_CARRIED_BAG..=LAST_CARRIED_BAG)
        .find_map(|bag| container_slot(sim, bag, item_id).map(|slot| Location::Bag { bag, slot }))
}

/// Ascending slot order inside one container, which the native ranges visit
/// in ascending order too.
fn container_slot(sim: &SimState, bag: i32, item_id: u32) -> Option<i32> {
    sim.bag_items
        .iter()
        .filter(|((item_bag, _), item)| *item_bag == bag && item.item_id == item_id)
        .map(|((_, slot), _)| *slot)
        .min()
}

fn equipment_slot(sim: &SimState, item_id: u32) -> Option<i32> {
    sim.player
        .equipped_items
        .iter()
        .filter(|(_, item)| item.item_id == item_id)
        .map(|(slot, _)| *slot)
        .min()
}
