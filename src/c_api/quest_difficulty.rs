//! Quest difficulty based on supplied quest and player levels.

use crate::c_api::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static, val_to_i32};
use rilua::{LuaResult, Val, vm::state::LuaState};

// Wrath 12340 data used by the simulator's shared quest difficulty model.
// The thresholds have not been measured in Retail, PTR, or Forever.
const QUEST_GREEN_RANGES: [u8; 20] = [4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];

pub(crate) fn quest_green_range(level: i32) -> u8 {
    let band = (level.max(0) as usize / 5).min(QUEST_GREEN_RANGES.len() - 1);
    QUEST_GREEN_RANGES[band]
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
        let globals = Val::Table(state.global);
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
    // DifficultyUtil.lua maps these level bands to display colors.
    let diff = i64::from(quest_level) - i64::from(player_level);
    let name = if diff >= 5 {
        "Impossible"
    } else if diff >= 3 {
        "Difficult"
    } else if diff >= -4 {
        "Fair"
    } else if -diff <= i64::from(quest_green_range(player_level)) {
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
