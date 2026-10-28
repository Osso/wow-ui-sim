//! Quest difficulty based on supplied quest and player levels.

use crate::c_api::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::{LuaResult, Val, vm::state::LuaState};

// User-approved for Retail/PTR; values are Wrath 12340 native data, not 12.x evidence.
const QUEST_GREEN_RANGES: [u8; 20] = [4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8];

pub(crate) fn quest_green_range(level: i32) -> u8 {
    let band = (level.max(0) as usize / 5).min(QUEST_GREEN_RANGES.len() - 1);
    QUEST_GREEN_RANGES[band]
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        let namespace = ensure_namespace(state, "C_PlayerInfo")?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetContentDifficultyQuestForPlayer",
            quest_difficulty,
        )?;
    }
    Ok(())
}

fn quest_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let quest_id = i32::from_stack(state, 1)?;
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
    // User-approved, not native evidence. DifficultyUtil.lua:33-45 maps these bands to colors.
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
