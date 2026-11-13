//! R74 EnterWorld: resolve and copy the selected record without reading arguments.
//! Glue-only publication and zero returns are approved policy based on INFERRED
//! evidence. Dispatch is deferred to the host; native lifecycle remains UNKNOWN.

use crate::c_api::c_login::character_roster::slots_for_sim;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if borrow_state(state)?.screen_kind.is_glue() {
        table_set_rust_fn_static(state, state.global, "EnterWorld", enter_world)
    } else {
        table_set(state, Val::Table(state.global), "EnterWorld", Val::Nil);
        Ok(())
    }
}

fn enter_world(state: &mut LuaState) -> LuaResult<u32> {
    let record = {
        let sim = borrow_state(state)?;
        let slots = slots_for_sim(&sim);
        usize::try_from(sim.glue_login.selected_character_index)
            .ok()
            .and_then(|index| slots.get(index))
            .and_then(|slot| slot.resolved_record())
            .cloned()
    };
    if let Some(record) = record {
        // One owned snapshot per valid call is host queueing policy, not a claim
        // about native request cadence. Never bootstrap Game in this callback.
        borrow_state_mut(state)?
            .glue_login
            .queue_world_entry_request(record);
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::c_api::c_login::character_roster::CharacterSlot;
    use crate::lua_api::WowLuaEnv;
    use crate::screen::ScreenKind;

    fn glue_env() -> WowLuaEnv {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(ScreenKind::Login);
        env
    }

    #[test]
    fn glue_publishes_enter_world_and_invalid_selections_are_silent() {
        let env = glue_env();
        env.exec("assert(type(EnterWorld) == 'function'); EnterWorld()")
            .unwrap();
        for index in [-1, 0, 5] {
            env.state().borrow_mut().glue_login.selected_character_index = index;
            env.exec("EnterWorld('ignored', {})").unwrap();
            assert!(
                env.state()
                    .borrow_mut()
                    .glue_login
                    .take_world_entry_request()
                    .is_none()
            );
        }
        let account = env.state().borrow().bnet_friends[0].game_accounts[0].clone();
        let mut mismatched = CharacterSlot::from(account.clone());
        mismatched.character_id = "unresolved-guid".into();
        let mut empty_guid = CharacterSlot::from(account);
        empty_guid.character_id.clear();
        env.state().borrow_mut().glue_login.characters = vec![
            CharacterSlot::default(),
            CharacterSlot {
                character_id: "missing-record".into(),
                record: None,
            },
            mismatched,
            empty_guid,
        ];
        for index in 0..4 {
            env.state().borrow_mut().glue_login.selected_character_index = index;
            env.exec("EnterWorld()").unwrap();
            assert!(
                env.state()
                    .borrow_mut()
                    .glue_login
                    .take_world_entry_request()
                    .is_none()
            );
        }
    }

    #[test]
    fn host_dispatch_keeps_owned_snapshots_across_connection_reset() {
        let env = glue_env();
        let mut account = env.state().borrow().bnet_friends[0].game_accounts[0].clone();
        account.character_name = "First".into();
        {
            let state = env.state();
            let mut sim = state.borrow_mut();
            sim.glue_login.characters = vec![CharacterSlot::from(account)];
            sim.glue_login.selected_character_index = 0;
        }
        env.exec("EnterWorld(false, 'ignored')").unwrap();
        env.state().borrow_mut().glue_login.characters[0]
            .record
            .as_mut()
            .unwrap()
            .account
            .character_name = "Second".into();
        env.exec("EnterWorld()").unwrap();
        let state = env.state();
        let mut sim = state.borrow_mut();
        sim.glue_login.characters.clear();
        sim.glue_login.reset_connection();
        assert_eq!(
            sim.glue_login
                .take_world_entry_request()
                .unwrap()
                .account
                .character_name,
            "First"
        );
        assert_eq!(
            sim.glue_login
                .take_world_entry_request()
                .unwrap()
                .account
                .character_name,
            "Second"
        );
        assert!(sim.glue_login.take_world_entry_request().is_none());
    }
}
