//! VERIFIED: pr46-49-decomp-answers.md R88ap part A §3 (Retail 12.1.5).
//! UNKNOWN settings packet ingestion and refresh events remain unmodeled.
//! Forever parity is INFERRED "R88g9 (native evidence pending)". The simulator mapping reads
//! rule 93 from the ID-keyed game-rule entry plus the
//! `non_integer_rule_ids` type tags, standing in for the native settings-packet rule store.
use crate::lua_api::methods::borrow_state;
use crate::lua_api::state::GameRuleKey;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "CanShowSetRoleButton", can_show)
}

fn can_show(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let rules = &sim.game_rules;
    let value = rules.rules.get(&GameRuleKey::Id(93));
    #[cfg(all(feature = "profile-retail", feature = "retail-12-1-5"))]
    let is_integer = !rules.non_integer_rule_ids.contains(&93);
    #[cfg(feature = "client-wowforever")]
    let is_integer = value.is_some_and(|value| value.integer_value.is_some());
    let enabled = is_integer && value.is_some_and(|value| value.as_int & 0xff != 0);
    drop(sim);
    state.push(Val::Bool(enabled));
    Ok(1)
}
