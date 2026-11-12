//! R45 SelectCharacter; model refresh and inferred CVar writes are omitted.

use crate::c_api::c_login::character_roster::slots_for_sim;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set, val_to_string};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    table_set_rust_fn_static(state, state.global, "SelectCharacter", select_character)?;
    register_season_query(state)
}

/// R49 Glue registration; Glue-only exposure is INFERRED from native evidence.
pub(crate) fn register_season_query(state: &mut LuaState) -> LuaResult<()> {
    super::glue_background::register(state)?;
    super::glue_enter_world::register(state)?;
    let glue = borrow_state(state)?.screen_kind.is_glue();
    if glue {
        table_set_rust_fn_static(
            state,
            state.global,
            "GetCharacterTimerunningSeasonID",
            character_season,
        )
    } else {
        table_set(
            state,
            Val::Table(state.global),
            "GetCharacterTimerunningSeasonID",
            Val::Nil,
        );
        Ok(())
    }
}

fn character_season(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let guid = match value {
        Val::Str(_) => val_to_string(state, value).unwrap_or_default(),
        Val::Num(number) => number.to_string(),
        _ => {
            return Err(runtime_error(
                "Usage: local timerunningSeasonID = GetCharacterTimerunningSeasonID(guid)",
            ));
        }
    };
    let season = {
        let sim = borrow_state(state)?;
        slots_for_sim(&sim)
            .iter()
            .find(|slot| slot.character_id == guid)
            .and_then(|slot| slot.resolved_record())
            .and_then(|record| record.timerunning_season_id)
            .filter(|season| *season != 0)
    };
    state.push(
        season
            .map(|season| Val::Num(f64::from(season)))
            .unwrap_or(Val::Nil),
    );
    Ok(1)
}

fn select_character(state: &mut LuaState) -> LuaResult<u32> {
    let value = stack_val(state, 1);
    let number = match value {
        Val::Num(number) => Some(number),
        // Convertible strings are INFERRED; the full native grammar is UNKNOWN.
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| runtime_error("Usage: SelectCharacter(index)"))?;
    let truncated = number.trunc();
    // Defensive simulator coverage boundary, NOT a native conversion contract.
    // R45 leaves non-finite and conversion-overflow behavior UNKNOWN. Do not let
    // a saturating Rust cast accidentally define either native behavior.
    if !truncated.is_finite() || truncated < f64::from(i32::MIN) || truncated > f64::from(i32::MAX)
    {
        return Err(runtime_error(
            "SelectCharacter: numeric conversion outside simulator coverage (native behavior unknown)",
        ));
    }
    let slots = {
        let sim = borrow_state(state)?;
        slots_for_sim(&sim)
    };
    if slots.is_empty() {
        return Ok(0);
    }
    let index = if truncated >= 1.0 && truncated <= slots.len() as f64 {
        (truncated - 1.0) as usize
    } else {
        0
    };
    if slots[index].resolved_record().is_none() {
        return Ok(0);
    }
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.glue_login.selected_character_index == index as i64 {
            return Ok(0);
        }
        sim.glue_login.selected_character_index = index as i64;
    }
    fire_named_event_state(
        state,
        "UPDATE_SELECTED_CHARACTER",
        &[Val::Num((index + 1) as f64)],
    );
    Ok(0)
}
