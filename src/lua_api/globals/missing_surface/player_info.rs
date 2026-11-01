//! `C_PlayerInfo` probe surface backed by `SimState.player`.
//!
//! Migrates 6 entries off the namespace stub tables:
//!
//! - `GetAlternateFormInfo()` — returns `(hasAlternateForm, inAlternateForm)` bools.
//! - `GetContentDifficultyCreatureForPlayer(unitToken)` — returns an
//!   `Enum.RelativeContentDifficulty` number (default 2 = equal).
//! - `GetGlidingInfo()` — returns movement-backed skyriding/gliding state.
//! - `GetPlayerMythicPlusRatingSummary(playerToken)` — returns a
//!   `MythicPlusRatingSummary` table or nothing if no data.
//! - `IsPlayerEligibleForNPE()` / `IsPlayerEligibleForNPEv2()` — return
//!   `(isEligible, failureReason)`. Best guess (native UNKNOWN): both read the
//!   single `is_npe_eligible` flag; the simulator models one NPE gate.
//! - `IsPlayerNPERestricted()` — returns the `is_npe_restricted` bool.
//! - `IsPlayerInRPE()` — returns the `is_in_rpe` bool.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_api::state_types::MythicPlusRatingMapSummary;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Relative content difficulty: 2 = equal level.
const RELATIVE_CONTENT_DIFFICULTY_EQUAL: f64 = 2.0;

pub(super) fn register_player_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_PlayerInfo")?;
    table_set_rust_fn_static(state, ns, "GetAlternateFormInfo", get_alternate_form_info)?;
    table_set_rust_fn_static(
        state,
        ns,
        "CanPlayerUseMountEquipment",
        can_player_use_mount_equipment,
    )?;
    table_set_rust_fn_static(state, ns, "GetDisplayID", get_display_id)?;
    table_set_rust_fn_static(state, ns, "GetNativeDisplayID", get_native_display_id)?;
    table_set_rust_fn_static(state, ns, "GetGlidingInfo", get_gliding_info)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetContentDifficultyCreatureForPlayer",
        get_content_difficulty_creature,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetPlayerMythicPlusRatingSummary",
        get_player_mythic_plus_rating_summary,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsTradingPostAvailable",
        is_trading_post_available,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsTravelersLogAvailable",
        is_travelers_log_available,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsTutorialsTabAvailable",
        is_tutorials_tab_available,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsPlayerEligibleForNPE",
        is_player_eligible_for_npe,
    )?;
    table_set_rust_fn_static(
        state,
        ns,
        "IsPlayerEligibleForNPEv2",
        is_player_eligible_for_npe,
    )?;
    table_set_rust_fn_static(state, ns, "IsPlayerNPERestricted", is_player_npe_restricted)?;
    table_set_rust_fn_static(state, ns, "IsPlayerInRPE", is_player_in_rpe)?;
    Ok(())
}

fn get_alternate_form_info(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let has = sim.player.is_alternate_form;
    let in_form = sim.player.alternate_form_is_default;
    drop(sim);
    state.push(Val::Bool(has));
    state.push(Val::Bool(in_form));
    Ok(2)
}

fn get_display_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_native_display_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn get_gliding_info(state: &mut LuaState) -> LuaResult<u32> {
    let movement = borrow_state(state)?.player.movement.clone();
    let forward_speed = if movement.flying || movement.moving {
        100.0
    } else {
        0.0
    };
    state.push(Val::Bool(movement.flying));
    state.push(Val::Bool(movement.flying || movement.mounted));
    state.push(Val::Num(forward_speed));
    Ok(3)
}

fn get_content_difficulty_creature(state: &mut LuaState) -> LuaResult<u32> {
    // Ignores unitToken — returns equal difficulty for the simulated player.
    let _ = state;
    state.push(Val::Num(RELATIVE_CONTENT_DIFFICULTY_EQUAL));
    Ok(1)
}

fn can_player_use_mount_equipment(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_player_mythic_plus_rating_summary(state: &mut LuaState) -> LuaResult<u32> {
    // Ignores playerToken — always returns the local player's data.
    let summary = borrow_state(state)?
        .player
        .mythic_plus_rating_summary
        .clone();
    let Some(summary) = summary else {
        return Ok(0); // mayreturnnothing
    };
    let result = create_table(state);
    table_set(
        state,
        result,
        "currentSeasonScore",
        Val::Num(summary.current_season_score),
    );
    let runs_table = build_mythic_plus_runs_table(state, summary.runs);
    table_set(state, result, "runs", runs_table);
    state.push(result);
    Ok(1)
}

fn build_mythic_plus_runs_table(
    state: &mut LuaState,
    runs: Vec<MythicPlusRatingMapSummary>,
) -> Val {
    let runs_table = create_table(state);
    for (i, run) in runs.into_iter().enumerate() {
        let entry = build_mythic_plus_run_entry(state, &run);
        set_table_array(state, runs_table, i as i64 + 1, entry);
    }
    runs_table
}

fn build_mythic_plus_run_entry(state: &mut LuaState, run: &MythicPlusRatingMapSummary) -> Val {
    let entry = create_table(state);
    table_set(
        state,
        entry,
        "challengeModeID",
        Val::Num(run.challenge_mode_id as f64),
    );
    table_set(state, entry, "mapScore", Val::Num(run.map_score));
    table_set(
        state,
        entry,
        "bestRunLevel",
        Val::Num(run.best_run_level as f64),
    );
    table_set(
        state,
        entry,
        "bestRunDurationMS",
        Val::Num(run.best_run_duration_ms as f64),
    );
    table_set(
        state,
        entry,
        "finishedSuccess",
        Val::Bool(run.finished_success),
    );
    entry
}

fn is_player_eligible_for_npe(state: &mut LuaState) -> LuaResult<u32> {
    let eligible = borrow_state(state)?.player.is_npe_eligible;
    let reason = if eligible { "" } else { "level" };
    state.push(Val::Bool(eligible));
    let reason_str = create_string(state, reason);
    state.push(reason_str);
    Ok(2)
}

fn is_player_npe_restricted(state: &mut LuaState) -> LuaResult<u32> {
    let restricted = borrow_state(state)?.player.is_npe_restricted;
    state.push(Val::Bool(restricted));
    Ok(1)
}

fn is_trading_post_available(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_tutorials_tab_available(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_travelers_log_available(state: &mut LuaState) -> LuaResult<u32> {
    let _ = state;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_player_in_rpe(state: &mut LuaState) -> LuaResult<u32> {
    let in_rpe = borrow_state(state)?.player.is_in_rpe;
    state.push(Val::Bool(in_rpe));
    Ok(1)
}
