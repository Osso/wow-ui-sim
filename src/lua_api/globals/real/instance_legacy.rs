//! Legacy instance globals backed by the current simulated world.
//!
//! The difficulty mapping is shared with `C_EncounterJournal`. Mapping 233→16
//! and 250→17 is VERIFIED for Forever (R88gb) and INFERRED for Retail.

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{Lua, LuaApiMut, LuaResult, Val};

const DEFAULT_DUNGEON_DIFFICULTY_ID: i32 = 1;

pub(crate) fn register_all(lua: &mut Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "GetBaseDifficultyID",
        crate::c_api::c_encounter_journal_difficulty::get_base_difficulty_id,
    )?;
    LuaApiMut::register_function(lua, "GetDungeonDifficultyID", get_dungeon_difficulty_id)
}

fn get_dungeon_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (R88g9, native evidence pending): use current difficulty; default to normal when unset.
    let difficulty = match borrow_state(state)?.world.instance_difficulty {
        id if id > 0 => id,
        _ => DEFAULT_DUNGEON_DIFFICULTY_ID,
    };
    state.push(Val::Num(f64::from(difficulty)));
    Ok(1)
}
