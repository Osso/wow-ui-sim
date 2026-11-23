//! `C_PlayerInteractionManager` surface consumed by NPC-interaction frames
//! (allied races, gossip, vendor, ...).
//!
//! State source:
//!
//! - `state.active_player_interactions: HashSet<i32>` — the set of
//!   `Enum.PlayerInteractionType` values whose corresponding NPC dialog is
//!   currently open. `ClearInteraction(type)` removes an active entry, fires
//!   its mapped close event when present, then requests the generic manager-hide
//!   notification with the interaction type outside Wrath.
//!
//! Only the type-specific close-event mapping the project currently consumes
//! is wired up. New interaction types should add a row to
//! [`close_event_for_interaction`] when their addon code lands.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

/// `Enum.PlayerInteractionType.AlliedRaceDetailsGiver`. Sourced from
/// `globals/enum_data/game_system.rs` `PLAYER_INTERACTION_TYPE` (position 9
/// after `None=0`, `TradePartner=1`, ..., `Banker=8`).
const ALLIED_RACE_DETAILS_GIVER: i32 = 9;

pub(crate) fn register_c_player_interaction_manager_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_PlayerInteractionManager")?;
    table_set_rust_fn_static(state, ns, "ClearInteraction", clear_interaction)?;
    // Retail/PTR/WowForever PlayerInteractionManagerDocumentation.lua:64-70 and
    // Mists/Era/Anniversary PlayerInteractionManagerDocumentation.lua:60-65 declare
    // `IsReplacingUnit`; Blizzard_UnitFrame/Classic/TargetFrame.lua:168 calls it on
    // every target change. The pinned 3.3.5 Wrath tree has no such documentation file.
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Wrath {
        table_set_rust_fn_static(
            state,
            ns,
            "IsInteractingWithNpcOfType",
            is_interacting_with_npc_of_type,
        )?;
        table_set_rust_fn_static(state, ns, "IsReplacingUnit", is_replacing_unit)?;
    }
    Ok(())
}

fn is_interacting_with_npc_of_type(state: &mut LuaState) -> LuaResult<u32> {
    let interaction_type = i32::from_stack(state, 1)?;
    let is_interacting = borrow_state(state)?
        .active_player_interactions
        .contains(&interaction_type);
    state.push(rilua::Val::Bool(is_interacting));
    Ok(1)
}

fn is_replacing_unit(state: &mut LuaState) -> LuaResult<u32> {
    // User-approved default, not native evidence.
    state.push(rilua::Val::Bool(false));
    Ok(1)
}

fn clear_interaction(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(interaction_type) = i32::from_stack(state, 1) else {
        return Ok(0);
    };
    clear_interaction_type(state, interaction_type)?;
    Ok(0)
}

pub(crate) fn clear_interaction_type(state: &mut LuaState, interaction_type: i32) -> LuaResult<()> {
    let was_active = borrow_state_mut(state)?
        .active_player_interactions
        .remove(&interaction_type);
    if was_active {
        if let Some(event) = close_event_for_interaction(interaction_type) {
            fire_named_event_state(state, event, &[]);
        }
        if ACTIVE != ClientProfile::Wrath {
            // PlayerInteractionManagerDocumentation.lua:95-103 declares the type payload.
            fire_named_event_state(
                state,
                "PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
                &[rilua::Val::Num(f64::from(interaction_type))],
            );
        }
    }
    Ok(())
}

fn close_event_for_interaction(interaction_type: i32) -> Option<&'static str> {
    match interaction_type {
        ALLIED_RACE_DETAILS_GIVER => Some("ALLIED_RACE_CLOSE"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allied_race_details_giver_matches_sequential_enum_position() {
        let entries = crate::lua_api::globals::enum_data::PLAYER_INTERACTION_TYPE.1;
        let position = entries
            .iter()
            .position(|name| *name == "AlliedRaceDetailsGiver")
            .expect("AlliedRaceDetailsGiver must be registered in PLAYER_INTERACTION_TYPE");
        assert_eq!(position as i32, ALLIED_RACE_DETAILS_GIVER);
    }
}
