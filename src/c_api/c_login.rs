//! `C_Login` connection state and selected-superdistrict helpers.

use crate::c_api::{c_district_utils, ensure_namespace, permanent_shims};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_table, table_set, table_set_static,
};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::screen::ScreenKind;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

pub mod backgrounds;
pub mod character_roster;
#[cfg(feature = "client-ptr")]
pub mod ptr_state;
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
    pub characters: Vec<character_roster::CharacterSlot>,
    /// Explicit host DB2 projections; empty until a data provider supplies rows.
    pub backgrounds: backgrounds::BackgroundRows,
    /// Host saved candidate input, zero-based. Native initial value/storage UNKNOWN.
    pub saved_character_index: Option<i64>,
    /// Clamp/search output, not a selected index or a native setting write.
    pub resolved_saved_character_index: Option<usize>,
    pub(crate) selected_character_index: i64,
    /// Availability is independent of the number of realm records (R40).
    /// False is the empty offline scenario; realm discovery is not emulated.
    pub realm_list_available: bool,
    pub logon_queue: Option<LogonQueue>,
    pub connection_info: ConnectionInfo,
    #[cfg(feature = "client-ptr")]
    pub ptr_connection: ptr_state::PtrConnectionState,
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
            backgrounds: Default::default(),
            saved_character_index: None,
            resolved_saved_character_index: None,
            selected_character_index: -1,
            realm_list_available: false,
            logon_queue: None,
            connection_info: Default::default(),
            #[cfg(feature = "client-ptr")]
            ptr_connection: ptr_state::PtrConnectionState::for_screen(
                crate::screen::ScreenKind::Game,
            ),
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
        self.selected_character_index = -1;
        self.resolved_saved_character_index = None;
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
        crate::lua_api::globals::real::glue_selection::register(state)?;
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

/// Update the Glue-only selected-superdistrict query and login-start state.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, screen: ScreenKind) {
    if !matches!(ACTIVE, ClientProfile::Ptr | ClientProfile::WowForever) {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    let state = lua.state_mut();
    update_selected_super_district_registration(state, screen.is_glue())
        .expect("register Glue-only C_Login.GetSelectedSuperDistrictID and SelectSuperDistrict");

    if screen == ScreenKind::Login {
        borrow_state_mut(state)
            .expect("borrow login state when starting the login screen")
            .selected_super_district_id = 0;
    }
}

fn update_selected_super_district_registration(
    state: &mut LuaState,
    is_glue: bool,
) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_Login") {
        Val::Table(namespace) => namespace,
        _ if is_glue => ensure_namespace(state, "C_Login")?,
        _ => return Ok(()),
    };

    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetSelectedSuperDistrictID",
            get_selected_super_district_id,
        )?;
        LuaApiMut::register_function(state, "SelectSuperDistrict", select_super_district)
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetSelectedSuperDistrictID",
            Val::Nil,
        );
        LuaApiMut::set_global_val(state, "SelectSuperDistrict", Val::Nil)
    }
}

fn get_selected_super_district_id(state: &mut LuaState) -> LuaResult<u32> {
    let selected_id = borrow_state(state)?.selected_super_district_id;
    state.push(Val::Num(f64::from(selected_id)));
    Ok(1)
}

fn select_super_district(state: &mut LuaState) -> LuaResult<u32> {
    let super_district_id = c_district_utils::read_super_district_id(
        state,
        "SelectSuperDistrict",
        "Usage: SelectSuperDistrict(superDistrictID)",
    )?;
    let is_available = borrow_state(state)?
        .super_district_info_by_id
        .contains_key(&super_district_id);
    if is_available {
        borrow_state_mut(state)?.selected_super_district_id = super_district_id;
    }
    Ok(0)
}

fn c_login_is_login_ready(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn c_login_get_state(state: &mut LuaState) -> LuaResult<u32> {
    if ACTIVE == ClientProfile::Ptr {
        #[cfg(feature = "client-ptr")]
        return ptr_state::get_state(state);
    }
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
    if ACTIVE == ClientProfile::WowForever {
        let has_selected_preset = {
            let sim = borrow_state(state)?;
            crate::c_api::experience_preset::has_selected(&sim.cvars)?
        };
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
            (
                "hasSelectedExperiencePreset",
                Val::Bool(has_selected_preset),
            ),
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
                login.selected_character_index = -1;
                login.resolved_saved_character_index = character_roster::resolve_saved_candidate(
                    &login.characters,
                    login.saved_character_index,
                );
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
