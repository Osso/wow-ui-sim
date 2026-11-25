//! Outfit settings, catalog queries, and applied selection.
//!
//! The full module is compiled for retail API epochs and modern-engine classic
//! profiles whose pinned API docs declare `C_TransmogOutfitInfo`. Forever uses
//! only the two existing lifecycle handlers required by its selected consumer.

mod catalog;
pub use catalog::{OutfitCatalog, OutfitEntry};

#[cfg(not(feature = "client-wowforever"))]
mod actions;
#[cfg(not(feature = "client-wowforever"))]
mod viewed;
#[cfg(feature = "retail-12-0-5")]
pub(crate) use actions::run_outfit_command;

#[cfg(not(feature = "client-wowforever"))]
mod pending_cost;
#[cfg(not(feature = "client-wowforever"))]
mod pending_cost_info;
#[cfg(not(feature = "client-wowforever"))]
pub use pending_cost_info::PendingTransmogCost;

mod viewed_slot_info;
#[cfg(not(feature = "client-wowforever"))]
mod viewed_slots;
pub use viewed_slot_info::ViewedOutfitSlotInfo;

mod model;
pub use model::{OutfitContents, OutfitState, SituationCategory, SituationGroup};
#[cfg(not(feature = "client-wowforever"))]
mod imports;
mod lifecycle;
#[cfg(not(feature = "client-wowforever"))]
mod pending;
#[cfg(not(feature = "client-wowforever"))]
mod situation_catalog;
#[cfg(not(feature = "client-wowforever"))]
mod situations;
#[cfg(not(feature = "client-wowforever"))]
mod slots;
#[cfg(not(feature = "client-wowforever"))]
mod visuals;

use super::helpers::ensure_namespace;
#[cfg(not(feature = "client-wowforever"))]
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
#[cfg(feature = "client-wowforever")]
use crate::lua_bridge::TableBuilder;
#[cfg(not(feature = "client-wowforever"))]
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(not(feature = "client-wowforever"))]
use rilua::Val;
use rilua::vm::state::LuaState;

#[cfg(feature = "client-wowforever")]
pub(super) fn register_forever(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TransmogOutfitInfo")?;
    lifecycle::register_forever(state, namespace)
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    lifecycle::register_admin(builder)
}

#[cfg(not(feature = "client-wowforever"))]
pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TransmogOutfitInfo")?;
    table_set_rust_fn_static(state, namespace, "InTransmogEvent", |s| {
        let value = borrow_state(s)?.transmog_outfits.in_event;
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, namespace, "TransmogEventActive", |s| {
        let value = borrow_state(s)?.transmog_outfits.event_active;
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    table_set_rust_fn_static(state, namespace, "IsUsableDiscountAvailable", |s| {
        let value = borrow_state(s)?.transmog_outfits.usable_discount_available;
        s.push(Val::Bool(value));
        Ok(1)
    })?;
    pending_cost::register(state, namespace)?;
    lifecycle::register(state, namespace)?;
    pending::register(state, namespace)?;
    situations::register(state, namespace)?;
    slots::register(state, namespace)?;
    imports::register(state, namespace)?;
    visuals::register(state)?;
    catalog::register(state, namespace)?;
    actions::register(state, namespace)?;
    viewed::register(state, namespace)?;
    viewed_slots::register(state, namespace)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        namespace,
        "CanPlayerTransmogSlot",
        can_player_transmog_slot,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetOutfitSituationsEnabled",
        get_outfit_situations_enabled,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetOutfitSituationsEnabled",
        set_outfit_situations_enabled,
    )
}

/// `Enum.TransmogOutfitSlot` spans Head (0) through WeaponRanged (14).
#[cfg(feature = "retail-12-1-0")]
const TRANSMOG_SLOT_WEAPON_RANGED: i32 = 14;
#[cfg(feature = "retail-12-1-0")]
const HUNTER_CLASS_ID: i32 = 3;

/// INFERRED: retail ranged weapons are Hunter-only, so only Hunters own a
/// transmoggable ranged slot (Blizzard_Transmog hides the ranged preview
/// toggle otherwise); every other outfit slot is available to all classes.
#[cfg(feature = "retail-12-1-0")]
fn can_player_transmog_slot(state: &mut LuaState) -> LuaResult<u32> {
    let valid = match stack_val(state, 1) {
        Val::Num(slot) if slot.fract() == 0.0 => match slot as i32 {
            0..TRANSMOG_SLOT_WEAPON_RANGED => true,
            TRANSMOG_SLOT_WEAPON_RANGED => {
                borrow_state(state)?.player.class_index == HUNTER_CLASS_ID
            }
            _ => false,
        },
        _ => false,
    };
    state.push(Val::Bool(valid));
    Ok(1)
}

#[cfg(not(feature = "client-wowforever"))]
fn get_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.outfit_situations_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

#[cfg(not(feature = "client-wowforever"))]
fn set_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "SetOutfitSituationsEnabled requires a boolean",
        ));
    };
    borrow_state_mut(state)?.outfit_situations_enabled = enabled;
    Ok(0)
}
