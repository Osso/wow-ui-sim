use crate::c_api::ensure_namespace;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

/// Catalog classification, independent of queue membership or active match state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingGroundKind {
    Arena,
    Battleground,
}

pub(crate) fn register_c_pvp_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_PvP")?;
    register_patch_12_1_c_pvp_surface(state, ns)?;
    register_training_grounds(state, ns)
}

#[cfg(feature = "retail-12-1-5")]
fn register_training_grounds(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "IsTrainingGroundsArena", |state| {
        query_training_ground(state, TrainingGroundKind::Arena)
    })?;
    table_set_rust_fn_static(state, ns, "IsTrainingGroundsBG", |state| {
        query_training_ground(state, TrainingGroundKind::Battleground)
    })
}

#[cfg(not(feature = "retail-12-1-5"))]
fn register_training_grounds(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    use crate::lua_api::methods::{create_table, table_set};
    // Prevent the namespace fallback from fabricating PTR-only queries.
    let removed = create_table(state);
    table_set(state, removed, "IsTrainingGroundsArena", Val::Bool(true));
    table_set(state, removed, "IsTrainingGroundsBG", Val::Bool(true));
    table_set(state, Val::Table(ns), "__wow_removed_keys", removed);
    Ok(())
}

#[cfg(feature = "retail-12-1-5")]
fn read_training_dungeon_id(state: &LuaState) -> LuaResult<i32> {
    let Val::Num(id) = crate::lua_bridge::stack_val(state, 1) else {
        return Err(rilua::runtime_error("LFG dungeon ID must be a number"));
    };
    let integral = id.is_finite() && id.fract() == 0.0;
    let representable = id >= i32::MIN as f64 && id <= i32::MAX as f64;
    if !integral || !representable {
        return Err(rilua::runtime_error(
            "LFG dungeon ID must be a finite integer representable as i32",
        ));
    }
    Ok(id as i32)
}

#[cfg(feature = "retail-12-1-5")]
fn query_training_ground(state: &mut LuaState, kind: TrainingGroundKind) -> LuaResult<u32> {
    let id = read_training_dungeon_id(state)?;
    let matches = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        sim.lfd_dungeons
            .iter()
            .find(|entry| entry.dungeon_id == id)
            .is_some_and(|entry| entry.training_ground_kind == Some(kind))
    };
    state.push(Val::Bool(matches));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_c_pvp_surface(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "CanSurrenderArena", can_surrender_arena)
}

#[cfg(not(feature = "retail-12-1-0"))]
fn register_patch_12_1_c_pvp_surface(_state: &mut LuaState, _ns: GcRef<Table>) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn can_surrender_arena(state: &mut LuaState) -> LuaResult<u32> {
    // The simulator does not model active rated-arena matches.
    state.push(Val::Bool(false));
    Ok(1)
}
