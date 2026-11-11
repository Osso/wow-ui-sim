//! Glue-only character-creation mode and facing inputs.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set_static};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const SET_IN_CHARACTER_CREATE_USAGE: &str =
    "Usage: C_CharacterCreation.SetInCharacterCreate(inCharacterCreate)";
const SET_CHARACTER_CREATE_FACING_USAGE: &str =
    "Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)";
// R36 marks the displayed bounds as inferred; tests check only the stable prefix and usage.
const SET_CHARACTER_CREATE_FACING_RANGE_ERROR: &str = concat!(
    "outside of expected range -3.402823e+38 to 3.402823e+38 - ",
    "Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)"
);
const SET_CHARACTER_CREATE_FACING_FINITE_ERROR: &str =
    "must be a finite number - Usage: C_CharacterCreation.SetCharacterCreateFacing(degrees)";
const DEGREES_PER_HALF_TURN: f32 = 180.0;

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

    update_default_facing_registration(state, namespace, is_glue)?;
    update_retail_and_forever_registration(state, namespace, is_glue)
}

fn update_default_facing_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if is_glue {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetDefaultCharacterCreateFacing",
            read_default_character_create_facing,
        )
    } else {
        table_set_static(
            state,
            Val::Table(namespace),
            "GetDefaultCharacterCreateFacing",
            Val::Nil,
        );
        Ok(())
    }
}

fn update_retail_and_forever_registration(
    state: &mut LuaState,
    namespace: GcRef<Table>,
    is_glue: bool,
) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }

    if is_glue {
        register_retail_and_forever_glue_apis(state, namespace)
    } else {
        for member in [
            "SetInCharacterCreate",
            "GetCharacterCreateFacing",
            "SetCharacterCreateFacing",
        ] {
            table_set_static(state, Val::Table(namespace), member, Val::Nil);
        }
        Ok(())
    }
}

fn register_retail_and_forever_glue_apis(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "SetInCharacterCreate",
        set_in_character_create,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCharacterCreateFacing",
        get_character_create_facing,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetCharacterCreateFacing",
        set_character_create_facing,
    )
}

fn get_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let radians = borrow_state(state)?.character_create_facing_radians;
    let degrees = radians * DEGREES_PER_HALF_TURN / std::f32::consts::PI;
    state.push(Val::Num(f64::from(degrees)));
    Ok(1)
}

fn set_character_create_facing(state: &mut LuaState) -> LuaResult<u32> {
    let degrees = read_character_create_facing_degrees(state)?;
    let radians = wrap_character_create_facing(degrees);
    borrow_state_mut(state)?.character_create_facing_radians = radians;
    Ok(0)
}

fn read_character_create_facing_degrees(state: &LuaState) -> LuaResult<f32> {
    let degrees = read_character_create_facing_argument(state)?;
    if !degrees.is_finite() {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_FINITE_ERROR,
        ));
    }
    if degrees < f64::from(f32::MIN) || degrees > f64::from(f32::MAX) {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_RANGE_ERROR,
        ));
    }
    Ok(degrees as f32)
}

fn read_character_create_facing_argument(state: &LuaState) -> LuaResult<f64> {
    let value = stack_val(state, 1);
    if matches!(value, Val::Nil) {
        return Err(character_create_facing_argument_error(
            SET_CHARACTER_CREATE_FACING_USAGE,
        ));
    }

    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| crate::c_api::helpers::parse_numeric_string(string.data())),
        _ => None,
    };
    number.ok_or_else(|| {
        character_create_facing_argument_error(SET_CHARACTER_CREATE_FACING_RANGE_ERROR)
    })
}

fn character_create_facing_argument_error(detail: &'static str) -> rilua::LuaError {
    crate::c_api::helpers::bad_argument(1, "SetCharacterCreateFacing", detail)
}

fn wrap_character_create_facing(degrees: f32) -> f32 {
    let pi = std::f32::consts::PI;
    let full_turn = 2.0 * pi;
    let remainder = (degrees * pi / DEGREES_PER_HALF_TURN) % full_turn;
    if remainder > pi {
        remainder - full_turn
    } else if remainder < -pi {
        remainder + full_turn
    } else {
        remainder
    }
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
