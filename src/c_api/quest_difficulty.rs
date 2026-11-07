//! Quest difficulty from supplied quest and player levels (WM89 bands).

use crate::c_api::helpers::{ensure_namespace, global_val};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static, val_to_i32};
use rilua::{LuaResult, Val, vm::state::LuaState};

/// WM89 §2: Retail and PTR select the lower Easy boundary from the fixed constant 4.
const RETAIL_EASY_BOUNDARY: i32 = 4;

/// WM89 §3: Forever selects the `GlobalCurve` row for selector keys 47 and 0. When that
/// row is absent the lower Easy boundary is the constant 5.
const FOREVER_ABSENT_ROW_BOUNDARY: i32 = 5;

/// UNKNOWN: the evaluated `GlobalCurve` (47, 0) curve value for the player's own level.
///
/// The simulator has no `GlobalCurve`, `Curve` or `CurvePoint` data, so the absent-row arm
/// always applies. Whatever a populated seam would return is truncated toward zero and
/// negated by [`boundary_from_curve_value`], which is the native arithmetic.
fn forever_curve_value(_player_level: i32) -> Option<f64> {
    None
}

/// WM89 §3: truncate toward zero, then negate in 32-bit arithmetic.
fn boundary_from_curve_value(curve_value: f64) -> i32 {
    (curve_value.trunc() as i32).wrapping_neg()
}

fn forever_easy_boundary(player_level: i32) -> i32 {
    match forever_curve_value(player_level) {
        Some(value) => boundary_from_curve_value(value),
        None => FOREVER_ABSENT_ROW_BOUNDARY,
    }
}

/// WM89 §2/§3: the fixed Retail/PTR constant versus the curve-backed Forever boundary.
fn easy_boundary(player_level: i32) -> i32 {
    match ACTIVE {
        ClientProfile::WowForever => forever_easy_boundary(player_level),
        _ => RETAIL_EASY_BOUNDARY,
    }
}

/// EMULATED quest comparison level.
///
/// WM89 §3: with a `ContentTuning` record the native code clamps the player level into the
/// record's bracket. `MinLevelSquish`/`MaxLevelSquish` are adjusted by the scaling-offset
/// modes: mode 1 adds 1, mode 2 adds an expansion-relative level, mode 3 adds the same
/// after first lowering the expansion index, and every other mode adds nothing. A level
/// below the lower bound uses that bound, otherwise the smaller of the level and the upper
/// bound is used.
///
/// The simulator supplies no quest tuning key, so no record can exist here. WM89 §4 gives
/// the no-record arms: Forever compares the stored quest level, Retail and PTR supply 0.
fn quest_comparison_level(quest_level: i32, _player_level: i32) -> i32 {
    match ACTIVE {
        ClientProfile::WowForever => quest_level,
        _ => 0,
    }
}

/// WM89 §6 legacy gate: quest flags below 8, comparison level 45 and player level 50 force
/// Trivial. The simulator models no quest flag field, so the gate is not modelled.
fn difficulty_name(difference: i32, player_level: i32) -> &'static str {
    if difference >= 5 {
        "Impossible"
    } else if difference >= 3 {
        "Difficult"
    } else if difference >= -2 {
        "Fair"
    } else if difference >= -easy_boundary(player_level) {
        "Easy"
    } else {
        "Trivial"
    }
}

/// Supplied inputs for the classifier. WM89 §4: a quest ID below 1, a missing player and a
/// missing quest row each prepare Fair instead of a value.
fn supplied_levels(state: &mut LuaState, quest_id: i32) -> LuaResult<Option<(i32, i32)>> {
    let sim = borrow_state(state)?;
    if quest_id < 1 || !sim.player.present {
        return Ok(None);
    }
    let Some(entry) = sim
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id == quest_id)
    else {
        return Ok(None);
    };
    Ok(Some((entry.level, sim.player.level)))
}

fn push_difficulty(state: &mut LuaState, name: &str) -> LuaResult<u32> {
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
    let Some((quest_level, player_level)) = supplied_levels(state, quest_id)? else {
        return push_difficulty(state, "Fair");
    };
    let comparison_level = quest_comparison_level(quest_level, player_level);
    let difference = comparison_level.wrapping_sub(player_level);
    push_difficulty(state, difficulty_name(difference, player_level))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WM89 §3: 5.9 and 5.0 both truncate to 5 and negate to -5; 4.9 truncates to 4.
    #[test]
    fn curve_value_truncates_toward_zero_before_negation() {
        assert_eq!(boundary_from_curve_value(5.9), -5);
        assert_eq!(boundary_from_curve_value(5.0), -5);
        assert_eq!(boundary_from_curve_value(4.9), -4);
        assert_eq!(boundary_from_curve_value(0.25), 0);
    }

    /// WM89 §3: an absent configuration row selects the constant 5, not a curve value.
    #[test]
    fn absent_curve_row_selects_the_constant_boundary() {
        assert_eq!(forever_curve_value(80), None);
        assert_eq!(forever_easy_boundary(80), FOREVER_ABSENT_ROW_BOUNDARY);
    }
}
