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
//! Not modelled, because the evidence leaves it UNKNOWN and the shared item
//! producer emits no such rows today: which items produce the item producer's
//! gem/socket metadata rows (`socketType`, `gemIcon`, `gemItemID`) and their
//! line type. An equipped copy also has no simulator GUID string, because
//! `C_Item.GetItemGUID` only models bag locations, so only a bag copy publishes
//! `guid`.

use crate::c_api::item_spell::item_guid_for_bag_slot;
use crate::c_api::tooltip_data::unknown_getter_failure;
use crate::c_api::tooltip_getter_args::signed;
use crate::items;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::{borrow_state, create_string, table_set};
use crate::lua_api::state::SimState;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

/// WM57: the backpack is bag id 0 and the ordinary carried bags are 1-4.
const BACKPACK: i32 = 0;
const FIRST_CARRIED_BAG: i32 = 1;
const LAST_CARRIED_BAG: i32 = 4;

/// The selected copy and the simulator instance identity the request carries.
struct OwnedInstance {
    guid: Option<String>,
}

/// Where the selected copy sits in the modelled player inventory.
enum Location {
    Bag { bag: i32, slot: i32 },
    Equipment,
}

impl Location {
    /// The simulator identity, in the shape `C_Item.GetItemGUID` publishes. The
    /// equipment range has no modelled GUID string.
    fn instance(self, item_id: u32) -> OwnedInstance {
        let guid = match self {
            Location::Bag { bag, slot } => Some(item_guid_for_bag_slot(bag, slot, item_id)),
            Location::Equipment => None,
        };
        OwnedInstance { guid }
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
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
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
    Some(location.instance(item_id))
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
