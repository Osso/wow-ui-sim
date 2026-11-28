//! Match-type predicates of the current PvP instance (PvpInfoDocumentation,
//! byte-identical Retail/Forever blocks). Original consumers: GroupFrameVisibility
//! (`IsInBrawl`), QueueStatusFrame (`IsInBrawl`/`IsSoloShuffle`), Timer.lua and
//! NamePlateClassificationFrame (`IsPVPMap`) and Blizzard_PVPMatch
//! (`IsMatchFactional`). Arena and battleground presence read the existing
//! `A_Admin.SetBattlefieldArena` / `A_Admin.SetInstanceInfo(…, "pvp", …)` inputs;
//! brawl, solo-shuffle and factional flags come from `A_Admin.SetPvPMatch`.
//! INFERRED (R9-9): a PvP map is an arena or battleground; world PvP zones are unmodeled.

use super::helpers::ensure_namespace;
use crate::lua_api::SimState;
use crate::lua_api::globals::real::admin_fields as fields;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Default, Clone, Copy)]
pub struct PvpMatchFlags {
    pub brawl: bool,
    pub solo_shuffle: bool,
    pub factional: bool,
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_PvP")?;
    let queries: [(&str, rilua::vm::closure::RustFn); 6] = [
        ("IsArena", |state| push(state, is_arena)),
        ("IsBattleground", |state| push(state, is_battleground)),
        ("IsPVPMap", |state| push(state, on_pvp_map)),
        ("IsInBrawl", |state| {
            push(state, |sim| on_pvp_map(sim) && sim.pvp_match.brawl)
        }),
        ("IsSoloShuffle", |state| {
            push(state, |sim| is_arena(sim) && sim.pvp_match.solo_shuffle)
        }),
        ("IsMatchFactional", |state| {
            push(state, |sim| on_pvp_map(sim) && sim.pvp_match.factional)
        }),
    ];
    for (name, query) in queries {
        table_set_rust_fn_static(state, namespace, name, query)?;
    }
    Ok(())
}

fn push(state: &mut LuaState, read: fn(&SimState) -> bool) -> LuaResult<u32> {
    let value = read(&*borrow_state(state)?);
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_arena(sim: &SimState) -> bool {
    sim.world.battlefield_arena
}

fn is_battleground(sim: &SimState) -> bool {
    sim.world.in_instance && sim.world.instance_type == "pvp"
}

fn on_pvp_map(sim: &SimState) -> bool {
    is_arena(sim) || is_battleground(sim)
}

/// `A_Admin.SetPvPMatch({brawl=, soloShuffle=, factional=})`.
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPvPMatch", |state| {
        const API: &str = "A_Admin.SetPvPMatch";
        let table = match stack_val(state, 1) {
            table @ Val::Table(_) => table,
            _ => return Err(runtime_error(format!("{API} expects an info table"))),
        };
        let flags = PvpMatchFlags {
            brawl: fields::boolean(state, table, API, "brawl")?,
            solo_shuffle: fields::boolean(state, table, API, "soloShuffle")?,
            factional: fields::boolean(state, table, API, "factional")?,
        };
        borrow_state_mut(state)?.pvp_match = flags;
        Ok(0)
    })
}
