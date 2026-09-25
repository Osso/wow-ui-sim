//! `C_Heirloom` probe surface backed by `WorldState.heirlooms` and
//! `WorldState.collected_heirlooms`.
//!
//! Migrates the heirloom collection API off `NAMESPACE_NIL_STUBS`:
//!
//! - `C_Heirloom.GetHeirloomInfo(itemID)` — returns the 10-value
//!   retail tuple (`name, itemEquipLoc, isPvP, itemTexture,
//!   upgradeLevel, source, searchFiltered, effectiveLevel, minLevel,
//!   maxLevel`). Reads `WorldState.heirlooms` for matching `itemID`.
//! - `C_Heirloom.GetHeirloomItemIDFromDisplayedIndex(index)` — returns
//!   the itemID at a 1-based display index, or `0` out of range.
//!
//! The stub list misspelled the index method as
//! `GetHeirloomItemIDFromDisplayedSlot` — that name is unused by the
//! Blizzard collection UI (confirmed via
//! `Blizzard_HeirloomCollection.lua:276`). Dropping the dead entry
//! and registering the real API instead.
//!
//! `itemEquipLoc` is returned as a string (`"INVTYPE_HEAD"`) even
//! though `apis.yaml` claims `number` — the collection UI feeds the
//! value to `GetHeirloomCategoryFromInvType(invType)` which string-
//! compares it against `"INVTYPE_*"` constants.

use super::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_api::state::{BagItem, SimState};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

pub(super) fn register_heirloom_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Heirloom")?;
    register_heirloom_item_fns(state, table_ref)?;
    register_heirloom_filter_fns(state, table_ref)?;
    register_heirloom_info_fns(state)
}

fn register_heirloom_item_fns(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_heirloom_item_mutation_fns(state, table_ref)?;
    register_heirloom_item_query_fns(state, table_ref)
}

fn register_heirloom_item_mutation_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "CreateHeirloom",
        c_heirloom_create_heirloom,
    )
}

fn register_heirloom_item_query_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_heirloom_info_query_fns(state, table_ref)?;
    register_heirloom_count_query_fns(state, table_ref)?;
    register_heirloom_status_query_fns(state, table_ref)
}

fn register_heirloom_info_query_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeirloomInfo",
        c_heirloom_get_heirloom_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeirloomItemIDFromDisplayedIndex",
        c_heirloom_get_item_id_from_displayed_index,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeirloomMaxUpgradeLevel",
        c_heirloom_get_max_upgrade_level,
    )
}

fn register_heirloom_count_query_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetNumHeirlooms",
        c_heirloom_get_num_heirlooms,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetNumKnownHeirlooms",
        c_heirloom_get_num_known_heirlooms,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetNumDisplayedHeirlooms",
        c_heirloom_get_num_displayed_heirlooms,
    )
}

fn register_heirloom_status_query_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "PlayerHasHeirloom",
        c_heirloom_player_has_heirloom,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeirloomLink",
        c_heirloom_get_heirloom_link,
    )
}

fn register_heirloom_filter_fns(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    register_heirloom_collection_filter_fns(state, table_ref)?;
    register_heirloom_source_filter_fns(state, table_ref)?;
    register_heirloom_class_filter_fns(state, table_ref)
}

fn register_heirloom_collection_filter_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetCollectedHeirloomFilter",
        c_heirloom_get_collected_filter,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetCollectedHeirloomFilter",
        c_heirloom_set_collected_filter,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetUncollectedHeirloomFilter",
        c_heirloom_get_uncollected_filter,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetUncollectedHeirloomFilter",
        c_heirloom_set_uncollected_filter,
    )
}

fn register_heirloom_source_filter_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetHeirloomSourceFilter",
        c_heirloom_get_source_filter,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetHeirloomSourceFilter",
        c_heirloom_set_source_filter,
    )
}

fn register_heirloom_class_filter_fns(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetClassAndSpecFilters",
        c_heirloom_get_class_and_spec_filters,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanHeirloomUpgradeFromPending",
        c_heirloom_can_upgrade_from_pending,
    )?;
    Ok(())
}

fn register_heirloom_info_fns(state: &mut LuaState) -> LuaResult<()> {
    let info_ref = ensure_namespace(state, "C_HeirloomInfo")?;
    table_set_rust_fn_static(
        state,
        info_ref,
        "IsHeirloomSourceValid",
        c_heirloom_info_is_source_valid,
    )?;
    table_set_rust_fn_static(
        state,
        info_ref,
        "SetAllSourceFilters",
        c_heirloom_info_set_all_source_filters,
    )?;
    table_set_rust_fn_static(
        state,
        info_ref,
        "SetDefaultFilters",
        c_heirloom_info_set_default_filters,
    )?;
    Ok(())
}

fn c_heirloom_create_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let Some(slot) = insert_heirloom_copy(state, item_id)? else {
        return Ok(0);
    };

    fire_named_event_state(state, "BAG_UPDATE", &[Val::Num(slot.0 as f64)]);
    fire_named_event_state(state, "BAG_UPDATE_DELAYED", &[]);
    Ok(0)
}

fn insert_heirloom_copy(state: &mut LuaState, item_id: u32) -> LuaResult<Option<(i32, i32)>> {
    let mut sim = borrow_state_mut(state)?;
    if !can_create_heirloom(&sim, item_id) {
        return Ok(None);
    }

    let Some(slot) = first_free_backpack_slot(&sim) else {
        return Ok(None);
    };

    sim.bag_items.insert(
        slot,
        BagItem {
            item_id,
            stack_count: 1,
            hyperlink: None,
        },
    );
    Ok(Some(slot))
}

fn can_create_heirloom(sim: &SimState, item_id: u32) -> bool {
    heirloom_is_collected(sim, item_id)
        && sim
            .world
            .heirlooms
            .iter()
            .any(|heirloom| heirloom.item_id == item_id)
}

fn heirloom_is_collected(sim: &SimState, item_id: u32) -> bool {
    sim.world.collected_heirlooms.contains(&item_id)
}

fn first_free_backpack_slot(sim: &SimState) -> Option<(i32, i32)> {
    (1..=sim.bag_num_slots(0))
        .map(|slot| (0, slot))
        .find(|slot| !sim.bag_items.contains_key(slot))
}

fn c_heirloom_get_heirloom_info(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let entry = {
        let sim = borrow_state(state)?;
        sim.world
            .heirlooms
            .iter()
            .find(|h| h.item_id == item_id)
            .cloned()
    };
    let Some(h) = entry else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let name = create_string(state, &h.name);
    let equip_loc = create_string(state, &h.equip_loc);
    let source = create_string(state, &h.source);
    state.push(name);
    state.push(equip_loc);
    state.push(Val::Bool(false));
    state.push(Val::Num(h.icon as f64));
    state.push(Val::Num(h.upgrade_level as f64));
    state.push(source);
    state.push(Val::Bool(false));
    state.push(Val::Num(h.max_level as f64));
    state.push(Val::Num(h.min_level as f64));
    state.push(Val::Num(h.max_level as f64));
    Ok(10)
}

fn c_heirloom_get_item_id_from_displayed_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let item_id = usize::try_from(index.saturating_sub(1))
        .ok()
        .and_then(|idx| {
            borrow_state(state)
                .ok()?
                .world
                .heirlooms
                .get(idx)
                .map(|h| h.item_id)
        });
    match item_id {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Num(0.0)),
    }
    Ok(1)
}

fn c_heirloom_get_max_upgrade_level(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let level = borrow_state(state)?
        .world
        .heirlooms
        .iter()
        .find(|heirloom| heirloom.item_id == item_id)
        .map(|heirloom| heirloom.upgrade_level)
        .unwrap_or(0);
    state.push(Val::Num(level as f64));
    Ok(1)
}

fn c_heirloom_get_num_heirlooms(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.world.heirlooms.len() as i32;
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn c_heirloom_get_num_known_heirlooms(state: &mut LuaState) -> LuaResult<u32> {
    let count = {
        let sim = borrow_state(state)?;
        sim.world
            .heirlooms
            .iter()
            .filter(|heirloom| heirloom_is_collected(&sim, heirloom.item_id))
            .count() as i32
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn c_heirloom_get_num_displayed_heirlooms(state: &mut LuaState) -> LuaResult<u32> {
    c_heirloom_get_num_heirlooms(state)
}

fn c_heirloom_player_has_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let has = {
        let sim = borrow_state(state)?;
        heirloom_is_collected(&sim, item_id)
    };
    state.push(Val::Bool(has));
    Ok(1)
}

fn c_heirloom_get_heirloom_link(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let link = borrow_state(state).ok().and_then(|sim| {
        sim.world
            .heirlooms
            .iter()
            .find(|heirloom| heirloom.item_id == item_id)
            .map(|heirloom| {
                format!(
                    "|cff00ccff|Hitem:{}::::::::80:::::|h[{}]|h|r",
                    heirloom.item_id, heirloom.name
                )
            })
    });
    match link {
        Some(link) => {
            let lua_link = create_string(state, &link);
            state.push(lua_link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_heirloom_get_collected_filter(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_heirloom_set_collected_filter(state: &mut LuaState) -> LuaResult<u32> {
    let _enabled = bool::from_stack(state, 1)?;
    Ok(0)
}

fn c_heirloom_get_uncollected_filter(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_heirloom_set_uncollected_filter(state: &mut LuaState) -> LuaResult<u32> {
    let _enabled = bool::from_stack(state, 1)?;
    Ok(0)
}

fn c_heirloom_get_source_filter(state: &mut LuaState) -> LuaResult<u32> {
    let _source = i32::from_stack(state, 1)?;
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_heirloom_set_source_filter(state: &mut LuaState) -> LuaResult<u32> {
    let _source = i32::from_stack(state, 1)?;
    let _enabled = bool::from_stack(state, 2)?;
    Ok(0)
}

fn c_heirloom_get_class_and_spec_filters(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(2)
}

fn c_heirloom_can_upgrade_from_pending(state: &mut LuaState) -> LuaResult<u32> {
    let _item_id = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_heirloom_info_is_source_valid(state: &mut LuaState) -> LuaResult<u32> {
    let _source = i32::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn c_heirloom_info_set_all_source_filters(state: &mut LuaState) -> LuaResult<u32> {
    let _enabled = bool::from_stack(state, 1)?;
    Ok(0)
}

fn c_heirloom_info_set_default_filters(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
