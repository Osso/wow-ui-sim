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
mod retail_lifecycle;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
mod submission;

/// R42: supplied Battle.net queue record, separate from the WoW connection state.
#[derive(Clone, Copy, Debug)]
pub struct LogonQueue {
    pub server_position: i32,
    pub estimated_seconds: i32,
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
    /// Host dispatch queue, not a native cadence/debounce contract (R74).
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    world_entry_requests: std::collections::VecDeque<character_roster::CharacterRecord>,
    /// Latest accepted R76 order, separate from the live roster.
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub(crate) saved_character_order: Vec<i32>,
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
    retail_glue: retail_lifecycle::RetailGlueState,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            characters: Vec::new(),
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            world_entry_requests: Default::default(),
            #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
            saved_character_order: Vec::new(),
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
            retail_glue: Default::default(),
        }
    }
}

impl LoginState {
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub(crate) fn queue_world_entry_request(&mut self, record: character_roster::CharacterRecord) {
        self.world_entry_requests.push_back(record);
    }

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub(crate) fn take_world_entry_request(&mut self) -> Option<character_roster::CharacterRecord> {
        self.world_entry_requests.pop_front()
    }

    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pub(crate) fn has_world_entry_request(&self) -> bool {
        !self.world_entry_requests.is_empty()
    }

    pub(crate) fn reset_connection(&mut self) {
        // Keep copied world-entry requests: the host transition resets connection
        // state before consuming subsequent queued requests. This is host policy.
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
    #[cfg(feature = "client-ptr")]
    register_ptr_glue_producers(state)?;
    permanent_shims::c_login::register_c_login_defaults(state, ns)?;
    Ok(())
}

// Narrow, individually approved inferred producers. Preserve the existing order.
#[cfg(feature = "client-ptr")]
fn register_ptr_glue_producers(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Ptr {
        return Ok(());
    }
    crate::lua_api::globals::real::glue_connection::register(state)?;
    crate::lua_api::globals::real::glue_character_season::register(state)?;
    crate::lua_api::globals::real::glue_select_character::register(state)?;
    crate::lua_api::globals::real::glue_background::register_background_only(state)?;
    crate::lua_api::globals::real::glue_background::register_model_ffx_only(state)
}

/// Update screen-scoped C_Login members.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, screen: ScreenKind) {
    if ACTIVE == ClientProfile::Retail {
        let mut lua = env.lua.borrow_mut();
        retail_lifecycle::update_screen_registration(lua.state_mut(), screen.is_glue())
            .expect("register Retail Glue login lifecycle APIs");
        return;
    }
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
    let Some(step) = advance_login_state(env) else {
        return Ok(());
    };
    publish_login_step(env, step)
}

fn advance_login_state(env: &crate::lua_api::WowLuaEnv) -> Option<u8> {
    let mut sim = env.state().borrow_mut();
    let login = &mut sim.glue_login;
    let step = login.pending_step;
    match step {
        1 => advance_login_connecting(login),
        2 => advance_login_connected(login),
        3 => process_character_list(login),
        _ => return None,
    }
    Some(step)
}

fn advance_login_connecting(login: &mut LoginState) {
    login.aurora_state = 4;
    login.pending_step = 2;
}

fn advance_login_connected(login: &mut LoginState) {
    login.aurora_state = 1;
    login.connected_to_wow = true;
    login.pending_step = 3;
    if ACTIVE == ClientProfile::Retail {
        retail_lifecycle::mark_character_list_pending(login);
    }
}

fn process_character_list(login: &mut LoginState) {
    login.received_connection_info = login.connection_info;
    login.selected_character_index = -1;
    login.resolved_saved_character_index =
        character_roster::resolve_saved_candidate(&login.characters, login.saved_character_index);
    if ACTIVE == ClientProfile::Retail {
        retail_lifecycle::complete_character_list(login);
    }
    login.pending_step = 0;
}

fn publish_login_step(env: &crate::lua_api::WowLuaEnv, step: u8) -> crate::Result<()> {
    if step == 3 {
        let count = env.state().borrow().glue_login.characters.len();
        return env.fire_event_with_args("CHARACTER_LIST_UPDATE", &[Val::Num(count as f64)]);
    }
    if step == 2 && ACTIVE == ClientProfile::Retail {
        env.fire_event("CHARACTER_LIST_RETRIEVING")?;
    }
    env.fire_event("LOGIN_STATE_CHANGED")
}

/// Publish a host queue snapshot. R42 requires an event for each active update,
/// even unchanged ones. Clearing the host record emits no inferred native event.
pub fn update_logon_queue(
    env: &crate::lua_api::WowLuaEnv,
    queue: Option<LogonQueue>,
) -> crate::Result<()> {
    let queue = queue.filter(|snapshot| snapshot.server_position > 1);
    env.state().borrow_mut().glue_login.logon_queue = queue;
    if queue.is_some() {
        env.fire_event("LOGIN_STATE_CHANGED")?;
    }
    Ok(())
}
