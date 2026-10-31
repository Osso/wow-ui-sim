//! Profile-cached dungeon score rarity colors.
use crate::c_api::global_val;
use crate::dungeon_score_rarity_runtime::{self, Family};
use crate::lua_api::methods::{borrow_state, call_function_state};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_ChallengeMode")?;
    for (name, f) in [
        (
            "GetDungeonScoreRarityColor",
            overall as rilua::vm::closure::RustFn,
        ),
        (
            "GetSpecificDungeonOverallScoreRarityColor",
            dungeon as rilua::vm::closure::RustFn,
        ),
        (
            "GetSpecificDungeonScoreRarityColor",
            run as rilua::vm::closure::RustFn,
        ),
        (
            "GetKeystoneLevelRarityColor",
            level as rilua::vm::closure::RustFn,
        ),
    ] {
        table_set_rust_fn_static(state, namespace, name, f)?;
    }
    Ok(())
}
fn overall(s: &mut LuaState) -> LuaResult<u32> {
    result(s, Family::Overall)
}
fn dungeon(s: &mut LuaState) -> LuaResult<u32> {
    result(s, Family::Dungeon)
}
fn run(s: &mut LuaState) -> LuaResult<u32> {
    result(s, Family::Run)
}
fn level(s: &mut LuaState) -> LuaResult<u32> {
    result(s, Family::Level)
}
fn result(state: &mut LuaState, family: Family) -> LuaResult<u32> {
    let Val::Num(score) = unwrap_secret(state, stack_val(state, 1))? else {
        return Err(runtime_error("expected number as argument 1"));
    };
    let color = {
        let sim = borrow_state(state)?;
        dungeon_score_rarity_runtime::color(score, family, &sim.passed_time_events)
            .map_err(runtime_error)?
    };
    let Some((r, g, b)) = color else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let create_color = global_val(state, "CreateColor");
    let color = call_function_state(
        state,
        create_color,
        &[Val::Num(r), Val::Num(g), Val::Num(b), Val::Num(1.0)],
    )?;
    state.push(color);
    Ok(1)
}
