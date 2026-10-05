//! Outfit settings, catalog queries, and patch-gated applied selection.

#[cfg(feature = "retail-12-0-5")]
mod catalog;
#[cfg(feature = "retail-12-0-5")]
pub use catalog::{OutfitCatalog, OutfitEntry};

#[cfg(feature = "retail-12-0-5")]
mod actions;
#[cfg(feature = "retail-12-0-5")]
mod viewed;
#[cfg(feature = "retail-12-0-5")]
pub(crate) use actions::run_outfit_command;

#[cfg(feature = "retail-12-0-5")]
mod pending_cost;
#[cfg(feature = "retail-12-0-5")]
mod pending_cost_info;
#[cfg(feature = "retail-12-0-5")]
pub use pending_cost_info::PendingTransmogCost;

#[cfg(feature = "retail-12-0-5")]
mod viewed_slot_info;
#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod viewed_slots;
#[cfg(feature = "retail-12-0-5")]
pub use viewed_slot_info::ViewedOutfitSlotInfo;

#[cfg(feature = "retail-12-0-5")]
mod model;
#[cfg(feature = "retail-12-0-5")]
pub use model::{OutfitContents, OutfitState};
#[cfg(feature = "retail-12-0-5")]
mod lifecycle;
#[cfg(feature = "retail-12-0-5")]
mod pending;
#[cfg(feature = "retail-12-0-5")]
mod situations;

use super::helpers::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_TransmogOutfitInfo")?;
    #[cfg(feature = "retail-12-0-5")]
    pending_cost::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    lifecycle::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    pending::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    situations::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    catalog::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    actions::register(state, namespace)?;
    #[cfg(feature = "retail-12-0-5")]
    viewed::register(state, namespace)?;
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    viewed_slots::register(state, namespace)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(
        state,
        namespace,
        "CanPlayerTransmogSlot",
        can_player_transmog_slot,
    )?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(state, namespace, "IsTransmogEnabled", is_transmog_enabled)?;
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

#[cfg(feature = "retail-12-1-0")]
fn is_transmog_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.transmog_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn get_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.outfit_situations_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

fn set_outfit_situations_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(enabled) = stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "SetOutfitSituationsEnabled requires a boolean",
        ));
    };
    borrow_state_mut(state)?.outfit_situations_enabled = enabled;
    Ok(0)
}
