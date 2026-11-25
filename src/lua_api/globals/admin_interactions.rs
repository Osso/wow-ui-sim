//! Simulator-only admin helpers for starting and ending player interactions.
//!
//! These belong under `A_Admin` because they drive test/simulator state. They
//! must not leak as WoW globals.

use crate::lua_api::globals::enum_data::PLAYER_INTERACTION_TYPE;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::script_helpers::fire_named_event_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn open_mailbox(state: &mut LuaState) -> LuaResult<u32> {
    let interaction_type = player_interaction_type("MailInfo");
    borrow_state_mut(state)?
        .active_player_interactions
        .insert(interaction_type);
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
        fire_named_event_state(state, "MAIL_SHOW", &[]);
    } else {
        fire_named_event_state(
            state,
            "PLAYER_INTERACTION_MANAGER_FRAME_SHOW",
            &[Val::Num(interaction_type as f64)],
        );
    }
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(super) fn open_guild_rename(state: &mut LuaState) -> LuaResult<u32> {
    let interaction_type = player_interaction_type("GuildRename");
    borrow_state_mut(state)?
        .active_player_interactions
        .insert(interaction_type);
    fire_named_event_state(
        state,
        "PLAYER_INTERACTION_MANAGER_FRAME_SHOW",
        &[Val::Num(interaction_type as f64)],
    );
    Ok(0)
}

pub(super) fn close_mailbox(state: &mut LuaState) -> LuaResult<u32> {
    let interaction_type = player_interaction_type("MailInfo");
    let was_active = borrow_state_mut(state)?
        .active_player_interactions
        .remove(&interaction_type);
    if was_active {
        if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Wrath {
            fire_named_event_state(state, "MAIL_CLOSED", &[]);
        } else {
            fire_named_event_state(
                state,
                "PLAYER_INTERACTION_MANAGER_FRAME_HIDE",
                &[Val::Num(interaction_type as f64)],
            );
        }
    }
    Ok(0)
}

fn player_interaction_type(name: &str) -> i32 {
    PLAYER_INTERACTION_TYPE
        .1
        .iter()
        .position(|entry| *entry == name)
        .expect("PlayerInteractionType enum entry must exist") as i32
}

#[cfg(test)]
mod tests {
    #[test]
    fn mail_info_interaction_type_matches_retail_ordinal() {
        assert_eq!(super::player_interaction_type("MailInfo"), 17);
    }
}
