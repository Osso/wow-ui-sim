//! `C_PetBattles.GetNumPets(owner)` + `GetBattleState()`.
//!
//! `owner` follows Blizzard's convention: 1 = player, 2 = enemy; anything
//! else returns 0. `GetBattleState` mirrors `Enum.PetbattleState` — default
//! `0` (no active battle). The sim doesn't simulate pet battles, but tests
//! that exercise PetBattleFrame OnLoad paths can nudge the counts via admin.
//!
//! Admin: `A_Admin.SetPetBattleCounts(player?, enemy?)` plus
//! `A_Admin.SetPetBattleState(state?)` — missing args default to 0.

use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
use crate::lua_api::methods::{table_get, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const PET_BATTLE_RUNTIME_STATE_KEY: &str = "__wow_pet_battle_state";
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const NUM_PETS_PLAYER_KEY: &str = "numPetsPlayer";
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const NUM_PETS_ENEMY_KEY: &str = "numPetsEnemy";
#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
const BATTLE_STATE_KEY: &str = "battleState";

pub fn get_num_pets(state: &mut LuaState) -> LuaResult<u32> {
    let owner = Option::<f64>::from_stack(state, 1)?.map(|n| n as i32);
    let count = match owner {
        Some(1) => borrow_state(state)?.pet_battles.num_pets_player,
        Some(2) => borrow_state(state)?.pet_battles.num_pets_enemy,
        _ => 0,
    };
    state.push(Val::Num(count as f64));
    Ok(1)
}

pub fn get_battle_state(state: &mut LuaState) -> LuaResult<u32> {
    let sim_state = borrow_state(state)?.pet_battles.battle_state;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    let s = sim_state;
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    let s = {
        let runtime_state = runtime_state_i32(state, BATTLE_STATE_KEY).unwrap_or(0);
        if runtime_state != 0 {
            runtime_state
        } else {
            sim_state
        }
    };
    state.push(Val::Num(s as f64));
    Ok(1)
}

fn ensure_c_pet_battles_table(state: &mut LuaState) -> GcRef<Table> {
    let key = state.gc.intern_string_static(b"C_PetBattles");
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|t| t.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(r)) = existing {
        return r;
    }
    let new_val = create_table(state);
    let Val::Table(new_ref) = new_val else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), new_val, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    new_ref
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    let table_ref = ensure_c_pet_battles_table(state);
    table_set_rust_fn_static(state, table_ref, "GetNumPets", get_num_pets)?;
    table_set_rust_fn_static(state, table_ref, "GetBattleState", get_battle_state)?;
    Ok(())
}

/// `A_Admin.SetPetBattleCounts(player?, enemy?)` — missing args default to 0.
pub fn admin_set_pet_battle_counts(state: &mut LuaState) -> LuaResult<u32> {
    let player = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    let enemy = Option::<f64>::from_stack(state, 2)?.unwrap_or(0.0) as i32;
    let player = player.max(0);
    let enemy = enemy.max(0);
    {
        let mut st = borrow_state_mut(state)?;
        st.pet_battles.num_pets_player = player;
        st.pet_battles.num_pets_enemy = enemy;
    }
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    sync_legacy_counts(state, player, enemy);
    Ok(0)
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn sync_legacy_counts(state: &mut LuaState, player: i32, enemy: i32) {
    let runtime_state = get_or_create_runtime_state(state);
    table_set(
        state,
        runtime_state,
        NUM_PETS_PLAYER_KEY,
        Val::Num(player as f64),
    );
    table_set(
        state,
        runtime_state,
        NUM_PETS_ENEMY_KEY,
        Val::Num(enemy as f64),
    );
}

/// `A_Admin.SetPetBattleState(state?)` — missing arg clears to 0.
pub fn admin_set_pet_battle_state(state: &mut LuaState) -> LuaResult<u32> {
    let s = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as i32;
    {
        let mut st = borrow_state_mut(state)?;
        st.pet_battles.battle_state = s;
    }
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    {
        let runtime_state = get_or_create_runtime_state(state);
        table_set(state, runtime_state, BATTLE_STATE_KEY, Val::Num(s as f64));
    }
    Ok(0)
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn runtime_state_i32(state: &mut LuaState, key: &str) -> Option<i32> {
    match runtime_state(state).map(|runtime_state| table_get(state, runtime_state, key)) {
        Some(Val::Num(value)) => Some(value as i32),
        _ => None,
    }
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn runtime_state(state: &mut LuaState) -> Option<Val> {
    let runtime_state = table_get(
        state,
        Val::Table(state.global),
        PET_BATTLE_RUNTIME_STATE_KEY,
    );
    (!matches!(runtime_state, Val::Nil)).then_some(runtime_state)
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
pub(crate) fn get_or_create_runtime_state(state: &mut LuaState) -> Val {
    runtime_state(state).unwrap_or_else(|| {
        let runtime_state = create_table(state);
        table_set(
            state,
            Val::Table(state.global),
            PET_BATTLE_RUNTIME_STATE_KEY,
            runtime_state,
        );
        runtime_state
    })
}
