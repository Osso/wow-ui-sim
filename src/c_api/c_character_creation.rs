//! Glue-only character-creation mode and facing inputs.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const SET_IN_CHARACTER_CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.SetInCharacterCreate(inCharacterCreate)";

/// Update Glue-only character-creation members for the selected screen.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    update_registration(lua.state_mut(), is_glue)
        .expect("register Glue-only C_CharacterCreation members for the selected screen");
}

fn update_registration(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_CharacterCreation") {
        Val::Table(namespace) => namespace,
        _ if is_glue => ensure_namespace(state, "C_CharacterCreation")?,
        _ => return Ok(()),
    };

    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        if is_glue {
            table_set_rust_fn_static(
                state,
                namespace,
                "SetInCharacterCreate",
                set_in_character_create,
            )?;
        } else {
            table_set_static(
                state,
                Val::Table(namespace),
                "SetInCharacterCreate",
                Val::Nil,
            );
        }
    }

    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetDefaultCharacterCreateFacing",
            read_default_character_create_facing,
        )?;
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetDefaultCharacterCreateFacing",
            Val::Nil,
        );
    }

    Ok(())
}

fn read_default_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let race_id = read_selected_character_create_id(state, "__wow_selected_race_id");
    let sex_id = read_selected_character_create_id(state, "__wow_selected_sex_id");
    let facing = match (race_id, sex_id) {
        (Some(race_id), Some(sex_id)) => borrow_state(state)?
            .character_create_default_facing_by_race_sex
            .get(&(race_id, sex_id))
            .copied()
            .unwrap_or(0.0),
        _ => 0.0,
    };
    state.push(Val::Num(facing));
    Ok(1)
}

fn read_selected_character_create_id(state: &mut LuaState, name: &str) -> Option<i64> {
    let Val::Num(value) = LuaApiMut::get_global_val(state, name) else {
        return None;
    };
    if value.is_finite() && value.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&value) {
        Some(value as i64)
    } else {
        None
    }
}

pub(crate) fn set_in_character_create(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    if matches!(value, Val::Nil) {
        return Err(rilua::runtime_error(SET_IN_CHARACTER_CREATE_USAGE));
    }

    let entering = !matches!(value, Val::Bool(false));
    let mut sim = borrow_state_mut(state)?;
    if sim.in_character_create != entering {
        sim.in_character_create = entering;
    }
    Ok(0)
}
