//! Forever C_GameRules queries backed by native numeric rule IDs.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const HARDCORE_RULE_ID: i32 = 10;
const SELF_FOUND_ALLOWED_RULE_ID: i32 = 22;

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
        .rules_by_id
        .get(&rule_id)
        .copied()
        .unwrap_or_default();
    Ok(value)
}
