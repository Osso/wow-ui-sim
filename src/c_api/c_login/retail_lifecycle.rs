//! Retail Glue login teardown and character-list completion state (R88aa).

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set_static};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

use super::LoginState;

#[derive(Debug, Default)]
pub(super) struct RetailGlueState {
    login_cancellation_recorded: bool,
    login_error: Option<String>,
    /// Empty offline state. R88aa proves the 0/1 transitions, not startup timing.
    character_list_completion_state: i32,
}

pub(super) fn update_screen_registration(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_Login") {
        Val::Table(namespace) => namespace,
        _ => ensure_namespace(state, "C_Login")?,
    };
    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "DisconnectFromServer",
            disconnect_from_server,
        )?;
        LuaApiMut::register_function(
            state,
            "IsCharacterListUpdatePending",
            is_character_list_update_pending,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "DisconnectFromServer",
            Val::Nil,
        );
        LuaApiMut::set_global_val(state, "IsCharacterListUpdatePending", Val::Nil)
    }
}

pub(super) fn mark_character_list_pending(login: &mut LoginState) {
    login.retail_glue.character_list_completion_state = 0;
}

pub(super) fn complete_character_list(login: &mut LoginState) {
    login.retail_glue.character_list_completion_state = 0;
    login.retail_glue.character_list_completion_state = 1;
}

fn is_character_list_update_pending(state: &mut LuaState) -> LuaResult<u32> {
    let pending = borrow_state(state)?
        .glue_login
        .retail_glue
        .character_list_completion_state
        == 0;
    state.push(Val::Bool(pending));
    Ok(1)
}

fn disconnect_from_server(state: &mut LuaState) -> LuaResult<u32> {
    let notify_launcher = clear_login_state(state)?;
    fire_named_event_state(state, "AUTO_REALM_SWAP", &[Val::Bool(false)]);
    if notify_launcher {
        fire_named_event_state(state, "LAUNCHER_LOGIN_STATUS_CHANGED", &[]);
    }
    fire_named_event_state(state, "LOGIN_STATE_CHANGED", &[]);
    Ok(0)
}

fn clear_login_state(state: &mut LuaState) -> LuaResult<bool> {
    let login = &mut borrow_state_mut(state)?.glue_login;
    if login.connected_to_wow || login.pending_step != 0 {
        login.pending_step = 0;
    }
    login.aurora_state = 1;
    login.realm_list_available = false;
    login.received_connection_info = Default::default();
    login.connected_to_wow = false;
    // Keep GetState on this model snapshot so it reports the cleared connection.
    if login.retail_glue.login_error.is_some() {
        login.retail_glue.login_error = None;
    }
    let notify_launcher = !login.retail_glue.login_cancellation_recorded;
    login.retail_glue.login_cancellation_recorded = true;
    Ok(notify_launcher)
}

#[cfg(all(test, feature = "client-retail"))]
mod tests {
    use crate::{lua_api::WowLuaEnv, screen::ScreenKind};

    #[test]
    fn disconnect_clears_a_stored_login_error() {
        let env = WowLuaEnv::new().expect("Lua environment");
        env.set_screen_mode(ScreenKind::CharacterSelect);
        env.state().borrow_mut().glue_login.retail_glue.login_error = Some("stored".into());
        env.exec("C_Login.DisconnectFromServer()")
            .expect("Retail disconnect should succeed");
        assert!(
            env.state()
                .borrow()
                .glue_login
                .retail_glue
                .login_error
                .is_none()
        );
    }
}
