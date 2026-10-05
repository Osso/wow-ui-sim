//! Instance + mirror-timer probe globals backed by `SimState.world`.
//!
//! Migrates 3 entries off `GLOBAL_ZERO_STUBS`:
//!
//! - `GetInstanceInfo()`          → 10 values (11 with Retail 12.0.7's
//!   `hasWorldTier`) assembled from
//!   `WorldState.instance_*` (name, type, difficulty id + name, max
//!   players, dynamic-difficulty flags, instance id, group size, LFG
//!   dungeon id).
//! - `GetMirrorTimerInfo(index)`  → 7 values from
//!   `WorldState.mirror_timers[index-1]` (name, startValue, maxValue,
//!   scale, paused, label, spellID). Empty slots return the retail-safe
//!   `"UNKNOWN"` sentinel tuple because Blizzard startup iterates a fixed
//!   three-slot range and only skips setup when the name is `"UNKNOWN"`.
//! - `GetMirrorTimerProgress(name)` → current progress for the timer
//!   with the matching `name`, or nil when absent.
//! - `GetWorldElapsedTimers()`      → no active timer IDs by default.
//! - `GetWorldElapsedTime(id)`      → `(id, 0, 0)` so scenario / challenge
//!   timer probes avoid nil arithmetic when no timer state is seeded.
//! - `GetNumSavedInstances()` / `GetNumSavedWorldBosses()` → zero, because
//!   no raid-lock backing store is seeded by default.
//! - `GetSavedInstanceInfo(index)` / `GetSavedWorldBossInfo(index)` → no
//!   values for empty slots.
//! - `IsInLFGDungeon()` → true only when the seeded world state is inside an
//!   instance with an LFG dungeon id.

use crate::lua_api::methods::{borrow_state, create_string, create_string_static, val_to_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

fn stack_i32(state: &LuaState, index: i32) -> Option<i32> {
    match stack_val(state, index) {
        Val::Num(n) => Some(n as i32),
        _ => None,
    }
}

fn stack_string(state: &LuaState, index: i32) -> Option<String> {
    val_to_string(state, stack_val(state, index))
}

struct InstanceInfoSnapshot {
    name: String,
    instance_type: String,
    difficulty_id: i32,
    difficulty_name: String,
    max_players: i32,
    dynamic_difficulty: i32,
    is_dynamic: bool,
    instance_id: i32,
    group_size: i32,
    lfg_dungeon_id: Option<i32>,
    has_world_tier: bool,
}

fn snapshot_instance_info(state: &LuaState) -> LuaResult<InstanceInfoSnapshot> {
    let sim = borrow_state(state)?;
    let w = &sim.world;
    Ok(InstanceInfoSnapshot {
        name: w.instance_name.clone(),
        instance_type: w.instance_type.clone(),
        difficulty_id: w.instance_difficulty,
        difficulty_name: w.instance_difficulty_name.clone(),
        max_players: w.instance_max_players,
        dynamic_difficulty: w.instance_dynamic_difficulty,
        is_dynamic: w.instance_is_dynamic,
        instance_id: w.instance_id,
        group_size: w.instance_group_size,
        lfg_dungeon_id: w.instance_lfg_dungeon_id,
        has_world_tier: w.instance_has_world_tier,
    })
}

fn push_instance_info_fields(state: &mut LuaState, snap: InstanceInfoSnapshot) {
    let name_val = create_string(state, &snap.name);
    state.push(name_val);
    let type_val = create_string(state, &snap.instance_type);
    state.push(type_val);
    state.push(Val::Num(snap.difficulty_id as f64));
    let diff_name_val = create_string(state, &snap.difficulty_name);
    state.push(diff_name_val);
    state.push(Val::Num(snap.max_players as f64));
    state.push(Val::Num(snap.dynamic_difficulty as f64));
    state.push(Val::Bool(snap.is_dynamic));
    state.push(Val::Num(snap.instance_id as f64));
    state.push(Val::Num(snap.group_size as f64));
    match snap.lfg_dungeon_id {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
}

fn get_instance_info(state: &mut LuaState) -> LuaResult<u32> {
    let snap = snapshot_instance_info(state)?;
    let has_world_tier = snap.has_world_tier;
    push_instance_info_fields(state, snap);
    if cfg!(feature = "retail-12-0-7") {
        // Retail 12.0.7 appended the non-nilable `hasWorldTier` return.
        state.push(Val::Bool(has_world_tier));
        return Ok(11);
    }
    Ok(10)
}

fn is_in_lfg_dungeon(state: &mut LuaState) -> LuaResult<u32> {
    let result = {
        let sim = borrow_state(state)?;
        sim.world.in_instance && sim.world.instance_lfg_dungeon_id.is_some()
    };
    state.push(Val::Bool(result));
    Ok(1)
}

fn get_mirror_timer_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = stack_i32(state, 1).unwrap_or(0);
    let timer = usize::try_from(index.saturating_sub(1))
        .ok()
        .and_then(|idx| {
            borrow_state(state)
                .ok()?
                .world
                .mirror_timers
                .get(idx)
                .cloned()
        });
    let Some(t) = timer else {
        push_unknown_mirror_timer(state);
        return Ok(7);
    };

    let name_val = create_string(state, &t.name);
    state.push(name_val);
    state.push(Val::Num(t.start_value));
    state.push(Val::Num(t.max_value));
    state.push(Val::Num(t.scale));
    state.push(Val::Num(t.paused as f64));
    let label_val = create_string(state, &t.label);
    state.push(label_val);
    state.push(Val::Num(t.spell_id as f64));
    Ok(7)
}

fn push_unknown_mirror_timer(state: &mut LuaState) {
    let name_val = create_string_static(state, "UNKNOWN");
    state.push(name_val);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    let label_val = create_string_static(state, "");
    state.push(label_val);
    state.push(Val::Num(0.0));
}

fn get_mirror_timer_progress(state: &mut LuaState) -> LuaResult<u32> {
    let Some(name) = stack_string(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let progress = borrow_state(state)?
        .world
        .mirror_timers
        .iter()
        .find(|t| t.name == name)
        .map(|t| t.progress);
    match progress {
        Some(value) => state.push(Val::Num(value)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_world_elapsed_timers(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_world_elapsed_time(state: &mut LuaState) -> LuaResult<u32> {
    let timer_id = stack_i32(state, 1).unwrap_or(0);
    state.push(Val::Num(timer_id as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(3)
}

fn get_num_saved_instances(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_num_saved_world_bosses(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_saved_instance_info(state: &mut LuaState) -> LuaResult<u32> {
    push_nil_values(state, 14);
    Ok(14)
}

fn get_saved_world_boss_info(state: &mut LuaState) -> LuaResult<u32> {
    push_nil_values(state, 3);
    Ok(3)
}

fn set_saved_instance_extend(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn push_nil_values(state: &mut LuaState, count: usize) {
    for _ in 0..count {
        state.push(Val::Nil);
    }
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::set_global_val(lua, "MIRRORTIMER_NUMTIMERS", Val::Num(3.0))?;
    LuaApiMut::register_function(lua, "GetInstanceInfo", get_instance_info)?;
    LuaApiMut::register_function(lua, "IsInLFGDungeon", is_in_lfg_dungeon)?;
    LuaApiMut::register_function(lua, "GetMirrorTimerInfo", get_mirror_timer_info)?;
    LuaApiMut::register_function(lua, "GetMirrorTimerProgress", get_mirror_timer_progress)?;
    LuaApiMut::register_function(lua, "GetWorldElapsedTimers", get_world_elapsed_timers)?;
    LuaApiMut::register_function(lua, "GetWorldElapsedTime", get_world_elapsed_time)?;
    LuaApiMut::register_function(lua, "GetNumSavedInstances", get_num_saved_instances)?;
    LuaApiMut::register_function(lua, "GetNumSavedWorldBosses", get_num_saved_world_bosses)?;
    LuaApiMut::register_function(lua, "GetSavedInstanceInfo", get_saved_instance_info)?;
    LuaApiMut::register_function(lua, "GetSavedWorldBossInfo", get_saved_world_boss_info)?;
    LuaApiMut::register_function(lua, "SetSavedInstanceExtend", set_saved_instance_extend)?;
    Ok(())
}
