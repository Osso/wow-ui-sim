//! Seeded quest/runtime surface restored for the rilua global registrar.
//!
//! Master used `c_quest_api.rs` for this. The rilua branch removed that file
//! without replacing the quest/watch/objective surface, which leaves the
//! objective tracker with no watched quests to render.

pub mod data;
mod handlers;
mod legacy_globals;
mod register;
mod task_quest;
#[cfg(feature = "client-wrath")]
mod wrath_gossip;
#[cfg(feature = "client-wrath")]
mod wrath_objectives;
#[cfg(feature = "client-wrath")]
mod wrath_quest_log;
#[cfg(feature = "client-wrath")]
mod wrath_quest_watch;

use crate::lua_api::methods::create_table;
use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

pub(super) use handlers::is_world_quest;
pub use register::register_all;

pub(super) type SurfaceFn = fn(&mut LuaState) -> rilua::LuaResult<u32>;

/// Look up or create a global table by name, returning a GcRef to it.
pub(super) fn ensure_global_table(state: &mut LuaState, name: &'static str) -> GcRef<Table> {
    let key = state.gc.intern_string_static(name.as_bytes());
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|table| table.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }
    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), table, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table_ref
}
