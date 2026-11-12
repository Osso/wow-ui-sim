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

pub fn register_c_login(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Login")?;
    table_set_rust_fn_static(state, ns, "IsLoginReady", c_login_is_login_ready)?;
    table_set_rust_fn_static(state, ns, "GetState", c_login_get_state)?;
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
    let screen_kind = borrow_state(state)?.screen_kind;
    let (aurora_state, connected_to_wow, wow_connection_state, has_realm_list) =
        screen_kind.login_state();
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
