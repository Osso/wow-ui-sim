//! Forever C_GameRules queries backed by native numeric rule IDs.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const HARDCORE_RULE_ID: i32 = 10;

pub(crate) fn register_hardcore_active(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "IsHardcoreActive", is_hardcore_active)
}

/// VERIFIED: Forever 70707 reads native rule ID 10; an absent rule is false.
/// INFERRED: any nonzero numeric value makes the rule active.
fn is_hardcore_active(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .game_rules
        .rules_by_id
        .get(&HARDCORE_RULE_ID)
        .copied()
        .unwrap_or_default();
    state.push(Val::Bool(value != 0));
    Ok(1)
}
