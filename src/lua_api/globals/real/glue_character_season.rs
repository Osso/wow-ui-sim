//! Existing R49 requested-GUID season producer; narrow shared registration only.
//! PTR applicability is INFERRED (lt-941b #1299); native producer semantics unchanged.
use crate::c_api::c_login::character_roster::slots_for_sim;
use crate::lua_api::methods::{borrow_state, table_set, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Preserve the existing inferred Glue-only exposure and Game removal policy.
pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
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
