//! Quest difficulty based on supplied quest and player levels.

use crate::c_api::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static, val_to_i32};
use rilua::{LuaResult, Val, vm::state::LuaState};

// WM89: UNKNOWN populated GlobalCurve; EMULATED absent configuration row.
fn forever_quest_trivial_range(_player_comparison_level: i32) -> i32 {
    5
}

// EMULATED: ContentTuning and the native effective-level interface are not modeled here.
fn quest_comparison_levels(quest_level: i32, player_level: i32) -> (i32, i32) {
    (quest_level, player_level)
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        let namespace = ensure_namespace(state, "C_PlayerInfo")?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetContentDifficultyQuestForPlayer",
            quest_difficulty,
        )?;
    }
    if ACTIVE == ClientProfile::WowForever {
        // Forever QuestLogDocumentation.lua:684 and UnitDocumentation.lua:2895,2910.
        let quest_log = ensure_namespace(state, "C_QuestLog")?;
        table_set_rust_fn_static(state, quest_log, "GetTrivialRange", get_trivial_range)?;
        let globals = state.global;
        table_set_rust_fn_static(
            state,
            globals,
            "UnitQuestTrivialLevelRange",
            unit_trivial_range,
        )?;
        table_set_rust_fn_static(
            state,
            globals,
            "UnitQuestTrivialLevelRangeScaling",
            unit_trivial_range,
        )?;
    }
    Ok(())
}

fn get_trivial_range(state: &mut LuaState) -> LuaResult<u32> {
    let present = borrow_state(state)?.player.present;
    state.push(Val::Num(if present { 5.0 } else { 0.0 }));
    Ok(1)
}

fn unit_trivial_range(state: &mut LuaState) -> LuaResult<u32> {
    let unit = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    if !matches!(unit, Val::Str(_)) {
        return Err(rilua::runtime_error(
            "UnitQuestTrivialLevelRange requires a unit token",
        ));
    }
    state.push(Val::Num(5.0));
    Ok(1)
}

fn quest_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let quest_id = val_to_i32(value, 1)?;
    let (quest_level, player_level) = {
        let sim = borrow_state(state)?;
        let level = sim
            .quest_log_entries
            .entries
            .iter()
            .find(|entry| entry.quest_id == quest_id)
            .map(|entry| entry.level)
            .ok_or_else(|| {
                rilua::runtime_error(format!("quest level not modeled for questID {quest_id}"))
            })?;
        (level, sim.player.level)
    };
    let (quest_level, player_level) = quest_comparison_levels(quest_level, player_level);
    let diff = quest_level.wrapping_sub(player_level);
    let name = if diff >= 5 {
        "Impossible"
    } else if diff >= 3 {
        "Difficult"
    } else if diff >= -2 {
        "Fair"
    } else if diff
        >= -match ACTIVE {
            ClientProfile::WowForever => forever_quest_trivial_range(player_level),
            _ => 4,
        }
    {
        "Easy"
    } else {
        "Trivial"
    };
    let enums = global_val(state, "Enum");
    let difficulties = table_get(state, enums, "RelativeContentDifficulty");
    let result = table_get(state, difficulties, name);
    if matches!(result, Val::Nil) {
        return Err(rilua::runtime_error(
            "Enum.RelativeContentDifficulty is unavailable",
        ));
    }
    state.push(result);
    Ok(1)
}
