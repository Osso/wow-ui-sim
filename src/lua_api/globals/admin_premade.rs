//! Rilua A_Admin handlers — Premade listings.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// ── Premade listings ──────────────────────────────────────────────────────────

pub(super) fn add_premade_listing(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state_types::PremadeListing;
    let name = String::from_stack(state, 1)?;
    let comment = String::from_stack(state, 2)?;
    let activity_id = u32::from_stack(state, 3)?;
    let num = i32::from_stack(state, 4)?;
    let max = i32::from_stack(state, 5)?;
    let mut st = borrow_state_mut(state)?;
    let id = st.world.premade_listings.len() as u32 + 1;
    st.world.premade_listings.push(PremadeListing {
        metadata: Default::default(),
        search_result_id: id,
        name,
        comment,
        leader_name: "Player".to_string(),
        activity_id,
        num_members: num,
        max_members: max,
        voice_chat: String::new(),
        auto_accept: false,
        is_delisted: false,
        party_guid: format!("Party-3-0000-1234-{:08X}", id),
        tanks: 0,
        healers: 0,
        damagers: num,
        no_role: 0,
        classes_by_role: std::collections::HashMap::new(),
        general_playstyle: 0,
        cross_faction_listing: false,
        leader_faction_group: 0,
        num_bnet_friends: 0,
        num_char_friends: 0,
        num_guild_mates: 0,
    });
    drop(st);
    state.push(Val::Num(id as f64));
    Ok(1)
}

pub(super) fn clear_premade_listings(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.world.premade_listings.clear();
    Ok(0)
}

pub(super) fn update_premade_listing(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::stack_val;
    let result_id = u32::from_stack(state, 1)?;
    let field = String::from_stack(state, 2)?;
    let value = stack_val(state, 3);
    let mut st = borrow_state_mut(state)?;
    let Some(listing) = st
        .world
        .premade_listings
        .iter_mut()
        .find(|l| l.search_result_id == result_id)
    else {
        return Ok(0);
    };
    match field.as_str() {
        "numMembers" => {
            if let Val::Num(n) = value {
                listing.num_members = n as i32;
            }
        }
        "isDelisted" => {
            if let Val::Bool(b) = value {
                listing.is_delisted = b;
            }
        }
        _ => {}
    }
    Ok(0)
}
