//! R45 SelectCharacter; model refresh and inferred CVar writes are omitted.

use rilua::LuaResult;
use rilua::vm::state::LuaState;

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    super::glue_select_character::register(state)?;
    register_season_query(state)
}

/// R49 Glue registration; Glue-only exposure is INFERRED from native evidence.
pub(crate) fn register_season_query(state: &mut LuaState) -> LuaResult<()> {
    super::glue_background::register(state)?;
    super::glue_enter_world::register(state)?;
    super::glue_save_character_order::register(state)?;
    super::glue_character_season::register(state)
}
