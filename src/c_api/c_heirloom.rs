//! `C_Heirloom` class/spec filters, item predicate and pending-upgrade action.
//!
//! `C_Heirloom` predates `Blizzard_APIDocumentationGenerated`: no pinned
//! profile declares the namespace, so the evidence is the pinned consumer.
//!
//! - `Blizzard_Collections/Classic/Blizzard_HeirloomCollection.lua` reads the
//!   numeric class/spec pair at lines 22, 277, 539, 694 and 699 and writes it
//!   at lines 26, 32, 545, 557 and 706. `UNSPECIFIED_CLASS_FILTER` and
//!   `UNSPECIFIED_SPEC_FILTER` are both `0`
//!   (`Blizzard_ClassMenu/Blizzard_ClassMenu.lua:1-2`).
//! - `Blizzard_Collections/Classic/Blizzard_HeirloomCollection.lua:74-78`
//!   reads `IsPendingHeirloomUpgrade()` and calls `UpgradeHeirloom(itemID)`.
//! - `Blizzard_UIPanels_Game/Mainline/MerchantFrame.lua:360` and
//!   `Blizzard_UIPanels_Game/Vanilla/MerchantFrame.lua:262` call
//!   `IsItemHeirloom(itemID)`.
//!
//! The native pending-upgrade targeting flow is UNKNOWN. `HeirloomUpgradeTargeting`
//! defaults to no pending target, so `IsPendingHeirloomUpgrade()` reports
//! false, and `UpgradeHeirloom` only applies when a host supplies the pending
//! item ID. No other upgrade policy (cost, currency, cap) is inferred.

use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Host-supplied pending heirloom-upgrade target; absence means no pending
/// upgrade, matching the consumer's default branch. Native behavior is UNKNOWN.
#[derive(Debug, Default)]
pub struct HeirloomUpgradeTargeting {
    pub pending_item_id: Option<u32>,
}

pub(crate) fn register(state: &mut LuaState, table: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table,
        "GetClassAndSpecFilters",
        get_class_and_spec_filters,
    )?;
    table_set_rust_fn_static(
        state,
        table,
        "SetClassAndSpecFilters",
        set_class_and_spec_filters,
    )?;
    table_set_rust_fn_static(state, table, "IsItemHeirloom", is_item_heirloom)?;
    table_set_rust_fn_static(state, table, "IsPendingHeirloomUpgrade", is_pending_upgrade)?;
    table_set_rust_fn_static(state, table, "UpgradeHeirloom", upgrade_heirloom)
}

fn get_class_and_spec_filters(state: &mut LuaState) -> LuaResult<u32> {
    let (class_filter, spec_filter) = {
        let sim = borrow_state(state)?;
        (sim.heirloom_class_filter, sim.heirloom_spec_filter)
    };
    state.push(Val::Num(class_filter as f64));
    state.push(Val::Num(spec_filter as f64));
    Ok(2)
}

fn set_class_and_spec_filters(state: &mut LuaState) -> LuaResult<u32> {
    let class_filter = i32::from_stack(state, 1)?;
    let spec_filter = i32::from_stack(state, 2)?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.heirloom_class_filter = class_filter;
        sim.heirloom_spec_filter = spec_filter;
    }
    Ok(0)
}

fn is_item_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let is_heirloom = borrow_state(state)?
        .world
        .heirlooms
        .iter()
        .any(|heirloom| heirloom.item_id == item_id);
    state.push(Val::Bool(is_heirloom));
    Ok(1)
}

fn is_pending_upgrade(state: &mut LuaState) -> LuaResult<u32> {
    let pending = borrow_state(state)?
        .heirloom_upgrade_targeting
        .pending_item_id
        .is_some();
    state.push(Val::Bool(pending));
    Ok(1)
}

fn upgrade_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let upgraded = {
        let mut sim = borrow_state_mut(state)?;
        if sim.heirloom_upgrade_targeting.pending_item_id != Some(item_id) {
            false
        } else if let Some(heirloom) = sim
            .world
            .heirlooms
            .iter_mut()
            .find(|heirloom| heirloom.item_id == item_id)
        {
            // INFERRED: one pending targeting request advances one upgrade
            // level. Cost, currency and upgrade caps are not modeled.
            heirloom.upgrade_level = heirloom.upgrade_level.saturating_add(1);
            sim.heirloom_upgrade_targeting.pending_item_id = None;
            true
        } else {
            false
        }
    };
    if upgraded {
        fire_named_event_state(state, "HEIRLOOMS_UPDATED", &[Val::Num(item_id as f64)]);
    }
    Ok(0)
}
