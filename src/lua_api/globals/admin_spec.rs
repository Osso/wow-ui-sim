//! Rilua A_Admin handlers — Spec & talents.
//!
//! Extracted from rilua_admin_extras.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state_mut, create_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::FromStack;
use rilua::LuaResult;
use rilua::vm::state::LuaState;

// ── Spec & talents ────────────────────────────────────────────────────────────

pub(super) fn set_wrath_glyph_socket(state: &mut LuaState) -> LuaResult<u32> {
    let group = u32::from_stack(state, 1)?;
    let index = u32::from_stack(state, 2)?;
    if group == 0 || !(1..=6).contains(&index) {
        return Err(rilua::runtime_error(
            "expected a positive talent group and glyph socket 1..6",
        ));
    }
    let socket = crate::lua_api::state::WrathGlyphSocket {
        enabled: bool::from_stack(state, 3)?,
        glyph_type: Option::<u32>::from_stack(state, 4)?,
        glyph_spell_id: Option::<u32>::from_stack(state, 5)?,
        icon: Option::<String>::from_stack(state, 6)?,
    };
    borrow_state_mut(state)?
        .wrath_glyph_sockets
        .insert((group, index), socket);
    Ok(0)
}

pub(super) fn set_wrath_unspent_talent_points(state: &mut LuaState) -> LuaResult<u32> {
    let group = u32::from_stack(state, 1)?;
    let points = u32::from_stack(state, 2)?;
    if group == 0 {
        return Err(rilua::runtime_error("expected a positive talent group"));
    }
    borrow_state_mut(state)?
        .wrath_unspent_talent_points
        .insert(group, points);
    Ok(0)
}

pub(super) fn set_spec(state: &mut LuaState) -> LuaResult<u32> {
    let spec_index = i32::from_stack(state, 1)?;
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.player.active_spec_index == spec_index {
            return Ok(0);
        }
        sim.player.active_spec_index = spec_index;
    }
    if ACTIVE != ClientProfile::Wrath {
        let player = create_string(state, "player");
        fire_named_event_state(state, "PLAYER_SPECIALIZATION_CHANGED", &[player]);
    }
    Ok(0)
}

pub(super) fn set_talent_rank(state: &mut LuaState) -> LuaResult<u32> {
    let node_id = u32::from_stack(state, 1)?;
    let rank = u32::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .talents
        .set_node_rank(node_id, rank);
    Ok(0)
}

pub(super) fn set_talent_selection(state: &mut LuaState) -> LuaResult<u32> {
    let node_id = u32::from_stack(state, 1)?;
    let entry_id = u32::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .talents
        .set_node_selection(node_id, Some(entry_id));
    Ok(0)
}

pub(super) fn reset_talents(state: &mut LuaState) -> LuaResult<u32> {
    let mut st = borrow_state_mut(state)?;
    st.talents.clear_ranks();
    st.talents.node_selections.clear();
    st.talents.active_hero_subtree_id = None;
    Ok(0)
}
