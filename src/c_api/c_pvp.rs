#[cfg(all(
    feature = "retail-12-0-5",
    any(feature = "profile-retail", feature = "client-ptr")
))]
mod active_brawl;
#[cfg(feature = "retail-12-0-5")]
mod brawl_info;
#[cfg(feature = "retail-12-0-5")]
pub use brawl_info::PvpBrawlInfo;

#[cfg(feature = "retail-12-0-0")]
pub mod catalog;
use crate::c_api::ensure_namespace;
#[cfg(feature = "retail-12-1-0")]
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
#[cfg(feature = "retail-12-1-0")]
use rilua::Val;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

/// Catalog classification, independent of queue membership or active match state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrainingGroundKind {
    Arena,
    Battleground,
}

pub(crate) fn register_c_pvp_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_PvP")?;
    #[cfg(feature = "retail-12-0-0")]
    catalog::register(state, ns)?;
    #[cfg(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    ))]
    active_brawl::register(state, ns)?;
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
    // Prevent the namespace fallback from fabricating PTR-only queries.
    crate::c_api::mark_namespace_keys_removed(
        state,
        ns,
        &["IsTrainingGroundsArena", "IsTrainingGroundsBG"],
    );
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
    // 12.1.0 split the random Training Grounds join into arena/battleground calls.
    crate::c_api::mark_namespace_keys_removed(state, ns, &["JoinRandomTrainingGround"]);
    table_set_rust_fn_static(
        state,
        ns,
        "JoinRandomTrainingGroundArena",
        join_random_training_ground_arena,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "JoinRandomTrainingGroundBattleground",
        join_random_training_ground_battleground,
    )?;
    table_set_rust_fn_static(state, ns, "CanSurrenderArena", can_surrender_arena)
}

/// Random training-ground queues occupy the single modeled battlefield slot,
/// named after the PVPUI queue option that issues them.
#[cfg(feature = "retail-12-1-0")]
fn join_random_training_ground_arena(state: &mut LuaState) -> LuaResult<u32> {
    let name = "Random Training Ground Arena".to_string();
    crate::lua_api::globals::battlefield_verbs::queue_battlefield(state, 1, name)?;
    Ok(0)
}

#[cfg(feature = "retail-12-1-0")]
fn join_random_training_ground_battleground(state: &mut LuaState) -> LuaResult<u32> {
    let name = "Random Training Ground".to_string();
    crate::lua_api::globals::battlefield_verbs::queue_battlefield(state, 1, name)?;
    Ok(0)
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
