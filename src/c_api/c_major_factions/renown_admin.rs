//! `A_Admin.SetMajorFactionRenown(factionID, renownLevel, maxLevel)`: supplied
//! Renown progress for one major faction.
//!
//! Forever's Camelot `CanShowLegacyChallenges` and `ToggleLegacySystemUI`
//! (Blizzard_AchievementUI/Camelot/Blizzard_AchievementUI.lua:3-10,
//! Blizzard_LegacySystem_Bootstrap.lua:7-10) open only when
//! `C_MajorFactions.GetCurrentRenownLevel(LEGACY_REWARD_TRACK_FACTION_ID)` is
//! positive. The account's Legacy renown is server state, so the host supplies it.
//! A level change fires the documented synchronous
//! `MAJOR_FACTION_RENOWN_LEVEL_CHANGED(majorFactionID, newRenownLevel,
//! oldRenownLevel)` (MajorFactionsDocumentation.lua:233-242), which
//! `LegacyMicroButtonMixin:OnEvent` observes. INFERRED: a new row starts at 0.

use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::state::RenownLevelInfo;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

use super::MajorFactionData;

const ADMIN_NAME: &str = "A_Admin.SetMajorFactionRenown";

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetMajorFactionRenown", set_major_faction_renown)
}

/// Validated `(factionID, renownLevel, maxLevel)`.
fn read_renown_input(state: &LuaState) -> LuaResult<(i64, i32, i32)> {
    let faction_id = i64::from_stack(state, 1)?;
    let level = i64::from_stack(state, 2)?;
    let max_level = i64::from_stack(state, 3)?;
    if faction_id <= 0 {
        return Err(runtime_error(format!(
            "{ADMIN_NAME}: factionID must be positive"
        )));
    }
    if max_level <= 0 || max_level > i64::from(i32::MAX) {
        return Err(runtime_error(format!(
            "{ADMIN_NAME}: maxLevel must be a positive integer"
        )));
    }
    if level < 0 || level > max_level {
        return Err(runtime_error(format!(
            "{ADMIN_NAME}: renownLevel must be within 0..=maxLevel"
        )));
    }
    Ok((faction_id, level as i32, max_level as i32))
}

/// Levels `1..=max_level`, unlocked through `level`; the last is the capstone.
fn renown_levels(faction_id: i64, level: i32, max_level: i32) -> Vec<RenownLevelInfo> {
    (1..=max_level)
        .map(|entry| RenownLevelInfo {
            faction_id,
            level: entry,
            locked: entry > level,
            is_milestone: false,
            is_capstone: entry == max_level,
        })
        .collect()
}

/// Updates an existing row, or adds an unlocked row with no imported metadata.
/// Returns the previous level (0 for a new row).
fn store_renown(
    state: &mut LuaState,
    faction_id: i64,
    level: i32,
    max_level: i32,
) -> LuaResult<i32> {
    let mut sim = borrow_state_mut(state)?;
    let data = sim
        .major_factions
        .entry(faction_id)
        .or_insert_with(|| MajorFactionData {
            faction_id,
            is_unlocked: true,
            ..Default::default()
        });
    let old_level = data.renown_level;
    data.renown_level = level;
    data.max_level = max_level;
    sim.major_faction_renown_levels
        .insert(faction_id, renown_levels(faction_id, level, max_level));
    Ok(old_level)
}

fn set_major_faction_renown(state: &mut LuaState) -> LuaResult<u32> {
    let (faction_id, level, max_level) = read_renown_input(state)?;
    let old_level = store_renown(state, faction_id, level, max_level)?;
    if old_level != level {
        let args = [
            Val::Num(faction_id as f64),
            Val::Num(f64::from(level)),
            Val::Num(f64::from(old_level)),
        ];
        dispatch_event_now(state, "MAJOR_FACTION_RENOWN_LEVEL_CHANGED", &args)?;
    }
    Ok(0)
}
