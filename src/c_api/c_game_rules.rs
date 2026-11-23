//! Forever C_GameRules queries backed by native numeric rule IDs.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::GameRuleKey;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const HARDCORE_RULE_ID: i32 = 10;
const SELF_FOUND_ALLOWED_RULE_ID: i32 = 22;

pub(crate) fn key_from_name(state: &mut LuaState, name: &str) -> GameRuleKey {
    if ACTIVE == ClientProfile::WowForever {
        if let Some(id) = find_rule_id_for_name(state, name) {
            return GameRuleKey::Id(id);
        }
    }
    GameRuleKey::Name(name.to_owned())
}

pub(crate) fn key_from_id(state: &mut LuaState, id: i64) -> Option<GameRuleKey> {
    let name = find_rule_name_for_id(state, id)?;
    Some(if ACTIVE == ClientProfile::WowForever {
        GameRuleKey::Id(id as i32)
    } else {
        GameRuleKey::Name(name)
    })
}

fn game_rule_table(state: &mut LuaState) -> Option<GcRef<Table>> {
    let enum_key = state.gc.intern_string_static(b"Enum");
    let enum_value = state
        .gc
        .tables
        .get(state.global)?
        .get_str(enum_key, &state.gc.string_arena);
    let Val::Table(enum_table) = enum_value else {
        return None;
    };
    let rules_key = state.gc.intern_string_static(b"GameRule");
    match state
        .gc
        .tables
        .get(enum_table)?
        .get_str(rules_key, &state.gc.string_arena)
    {
        Val::Table(table) => Some(table),
        _ => None,
    }
}

fn find_rule_id_for_name(state: &mut LuaState, name: &str) -> Option<i32> {
    let table = game_rule_table(state)?;
    let mut key = Val::Nil;
    while let Some((next_key, value)) = state
        .gc
        .tables
        .get(table)?
        .next(key, &state.gc.string_arena)
        .ok()?
    {
        if let (Val::Str(name_key), Val::Num(id)) = (next_key, value) {
            let matches = state
                .gc
                .string_arena
                .get(name_key)
                .and_then(|value| std::str::from_utf8(value.data()).ok())
                == Some(name);
            if matches {
                return Some(id as i32);
            }
        }
        key = next_key;
    }
    None
}

fn find_rule_name_for_id(state: &mut LuaState, id: i64) -> Option<String> {
    let table = game_rule_table(state)?;
    let mut key = Val::Nil;
    while let Some((next_key, value)) = state
        .gc
        .tables
        .get(table)?
        .next(key, &state.gc.string_arena)
        .ok()?
    {
        if let (Val::Str(name_key), Val::Num(value)) = (next_key, value) {
            if value as i64 == id {
                return state
                    .gc
                    .string_arena
                    .get(name_key)
                    .and_then(|value| std::str::from_utf8(value.data()).ok())
                    .map(str::to_owned);
            }
        }
        key = next_key;
    }
    None
}

pub(crate) fn register_hardcore_active(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "IsHardcoreActive", is_hardcore_active)
}

pub(crate) fn register_self_found_allowed(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        namespace,
        "IsSelfFoundAllowed",
        is_self_found_allowed,
    )
}

/// VERIFIED: Forever 70707 reads native rule ID 10; an absent rule is false.
/// INFERRED: any nonzero numeric value makes the rule active.
fn is_hardcore_active(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_numeric_rule_value(state, HARDCORE_RULE_ID)?;
    state.push(Val::Bool(value != 0));
    Ok(1)
}

/// VERIFIED: Forever 70707 queries native rule ID 22; an absent rule is false.
/// INFERRED: a nonzero numeric value publishes true.
fn is_self_found_allowed(state: &mut LuaState) -> LuaResult<u32> {
    let value = read_numeric_rule_value(state, SELF_FOUND_ALLOWED_RULE_ID)?;
    state.push(Val::Bool(value != 0));
    Ok(1)
}

fn read_numeric_rule_value(state: &mut LuaState, rule_id: i32) -> LuaResult<i64> {
    let value = borrow_state(state)?
        .game_rules
        .rules
        .get(&GameRuleKey::Id(rule_id))
        .map(|value| value.as_int)
        .unwrap_or_default();
    Ok(value)
}
