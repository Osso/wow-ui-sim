//! `C_Login` glue-screen connection state helpers.

use crate::c_api::{ensure_namespace, permanent_shims};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, create_table, table_set};
use crate::lua_api::state_types::BnetGameAccount;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
mod submission;

/// R42: supplied Battle.net queue record, separate from the WoW connection state.
#[derive(Clone, Copy, Debug)]
pub struct LogonQueue {
    pub server_position: f64,
    pub estimated_seconds: f64,
}

/// R43 connection-info input. Zero is an empty simulator input, not a realm limit.
#[derive(Clone, Copy, Debug, Default)]
pub struct ConnectionInfo {
    pub ready: bool,
    pub character_limit: i32,
}

/// Host inputs and the offline login policy (lt-a667, ANSWER 30 / R40).
/// No credentials are retained. Existing direct-screen boot remains separate.
#[derive(Debug)]
pub struct LoginState {
    pub characters: Vec<BnetGameAccount>,
    /// Availability is independent of the number of realm records (R40).
    /// False is the empty offline scenario; realm discovery is not emulated.
    pub realm_list_available: bool,
    pub logon_queue: Option<LogonQueue>,
    pub connection_info: ConnectionInfo,
    pub(crate) received_connection_info: ConnectionInfo,
    pub(crate) submitted: bool,
    aurora_state: i32,
    connected_to_wow: bool,
    pending_step: u8,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            characters: Vec::new(),
            realm_list_available: false,
            logon_queue: None,
            connection_info: Default::default(),
            received_connection_info: Default::default(),
            submitted: false,
            aurora_state: 1,
            connected_to_wow: false,
            pending_step: 0,
        }
    }
}

impl LoginState {
    pub(crate) fn reset_connection(&mut self) {
        self.submitted = false;
        self.pending_step = 0;
        self.aurora_state = 1;
        self.connected_to_wow = false;
        self.received_connection_info = Default::default();
    }
}

pub fn register_c_login(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Login")?;
    table_set_rust_fn_static(state, ns, "IsLoginReady", c_login_is_login_ready)?;
    table_set_rust_fn_static(state, ns, "GetState", c_login_get_state)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    {
        table_set_rust_fn_static(state, ns, "Login", submission::login)?;
        crate::lua_api::globals::real::glue_connection::register(state)?;
        table_set_rust_fn_static(
            state,
            ns,
            "GetLogonQueueInfo",
            submission::get_logon_queue_info,
        )?;
    }
    permanent_shims::c_login::register_c_login_defaults(state, ns)?;
    Ok(())
}

fn c_login_is_login_ready(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_login_get_state(state: &mut LuaState) -> LuaResult<u32> {
    let (aurora_state, connected_to_wow, wow_connection_state, has_realm_list) = {
        let sim = borrow_state(state)?;
        let login = &sim.glue_login;
        if login.submitted {
            (
                login.aurora_state,
                login.connected_to_wow,
                1,
                login.realm_list_available,
            )
        } else {
            sim.screen_kind.login_state()
        }
    };
    if matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        let result = create_table(state);
        for (key, value) in [
            ("auroraState", Val::Num(f64::from(aurora_state))),
            ("connectedToWoW", Val::Bool(connected_to_wow)),
            (
                "wowConnectionState",
                Val::Num(f64::from(wow_connection_state)),
            ),
            ("hasRealmList", Val::Bool(has_realm_list)),
            // User-approved, not native evidence; native login policy is not modeled.
            ("waitingForRealmList", Val::Bool(false)),
            ("superDistrictChoicePending", Val::Bool(false)),
            ("keepingCharacterListOnRealmJoin", Val::Bool(false)),
            ("waitingForRuleSets", Val::Bool(false)),
            ("hasSelectedExperiencePreset", Val::Bool(true)),
        ] {
            table_set(state, result, key, value);
        }
        state.push(result);
        return Ok(1);
    }
    state.push(Val::Num(f64::from(aurora_state)));
    state.push(Val::Bool(connected_to_wow));
    state.push(Val::Num(f64::from(wow_connection_state)));
    state.push(Val::Bool(has_realm_list));
    state.push(Val::Bool(false));
    Ok(5)
}

/// Advance the explicit offline policy once per update, not native frame timing.
pub(crate) fn advance(env: &crate::lua_api::WowLuaEnv) -> crate::Result<()> {
    let step = {
        let mut sim = env.state().borrow_mut();
        let login = &mut sim.glue_login;
        let step = login.pending_step;
        match step {
            1 => {
                login.aurora_state = 4;
                login.pending_step = 2;
            }
            2 => {
                login.aurora_state = 1;
                login.connected_to_wow = true;
                login.pending_step = 3;
            }
            3 => {
                login.received_connection_info = login.connection_info;
                login.pending_step = 0;
            }
            _ => return Ok(()),
        }
        step
    };
    if step == 3 {
        let count = env.state().borrow().glue_login.characters.len();
        env.fire_event_with_args("CHARACTER_LIST_UPDATE", &[Val::Num(count as f64)])?;
    } else {
        env.fire_event("LOGIN_STATE_CHANGED")?;
    }
    Ok(())
}

/// Publish a host queue snapshot. R42 requires an event for each active update,
/// even unchanged ones. Clearing the host record emits no inferred native event.
pub fn update_logon_queue(
    env: &crate::lua_api::WowLuaEnv,
    queue: Option<LogonQueue>,
) -> crate::Result<()> {
    env.state().borrow_mut().glue_login.logon_queue = queue;
    if queue.is_some() {
        env.fire_event("LOGIN_STATE_CHANGED")?;
    }
    Ok(())
}
