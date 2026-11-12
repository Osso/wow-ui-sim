//! Game-global quest faction classification from supplied quest records.

use crate::c_api::parse_numeric_string;
use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const USAGE: &str = "Usage: GetQuestFactionGroup(questID";
const FUNCTION_NAME: &str = "GetQuestFactionGroup";
const ALLIANCE_ONLY_FACTION_MASK: i32 = 1;
const HORDE_ONLY_FACTION_MASK: i32 = 2;
const C_FLOAT_FRACTIONAL_DIGITS: usize = 6;

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, FUNCTION_NAME, get_quest_faction_group)
}

pub(crate) fn update_screen_registration(env: &WowLuaEnv, is_glue: bool) {
    let mut lua = env.lua.borrow_mut();
    let registration = if is_glue {
        LuaApiMut::set_global_val(&mut *lua, FUNCTION_NAME, Val::Nil)
    } else {
        LuaApiMut::register_function(&mut *lua, FUNCTION_NAME, get_quest_faction_group)
    };
    registration.expect("register Game-only GetQuestFactionGroup");
}

fn get_quest_faction_group(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = read_quest_id(state)?;
    if quest_id < 1 {
        return Ok(0);
    }

    let faction_mask = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
        .and_then(|entry| entry.faction_mask);
    let faction_group = match faction_mask {
        Some(ALLIANCE_ONLY_FACTION_MASK) => ALLIANCE_ONLY_FACTION_MASK,
        Some(HORDE_ONLY_FACTION_MASK) => HORDE_ONLY_FACTION_MASK,
        _ => return Ok(0),
    };

    state.push(Val::Num(f64::from(faction_group)));
    Ok(1)
}

fn read_quest_id(state: &LuaState) -> LuaResult<i32> {
    let value = stack_val(state, 1);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| parse_numeric_string(string.data())),
        _ => None,
    }
    .ok_or_else(|| rilua::runtime_error(USAGE))?;

    let is_outside_range = number < f64::from(i32::MIN) || number > f64::from(i32::MAX);
    if !number.is_finite() || is_outside_range {
        return Err(rilua::runtime_error(format!(
            "integer overflow attempting to store {}",
            format_c_float(number)
        )));
    }
    Ok(number.trunc() as i32)
}

fn format_c_float(number: f64) -> String {
    if number.is_nan() {
        return if number.is_sign_negative() {
            "-nan".to_string()
        } else {
            "nan".to_string()
        };
    }
    if number == f64::INFINITY {
        return "inf".to_string();
    }
    if number == f64::NEG_INFINITY {
        return "-inf".to_string();
    }
    format!(
        "{number:.precision$}",
        precision = C_FLOAT_FRACTIONAL_DIGITS
    )
}
