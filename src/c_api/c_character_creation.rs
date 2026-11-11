//! Glue-only character-creation mode input.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{borrow_state_mut, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const SET_IN_CHARACTER_CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.SetInCharacterCreate(inCharacterCreate)";

/// Install this native member only for the Glue UI surface.
pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return;
    }

    let mut lua = env.lua.borrow_mut();
    update_registration(lua.state_mut(), is_glue)
        .expect("register C_CharacterCreation.SetInCharacterCreate for the selected screen");
}

fn update_registration(state: &mut LuaState, is_glue: bool) -> LuaResult<()> {
    let namespace = match LuaApiMut::get_global_val(state, "C_CharacterCreation") {
        Val::Table(namespace) => namespace,
        _ if is_glue => ensure_namespace(state, "C_CharacterCreation")?,
        _ => return Ok(()),
    };

    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "SetInCharacterCreate",
            set_in_character_create,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "SetInCharacterCreate",
            Val::Nil,
        );
        Ok(())
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
