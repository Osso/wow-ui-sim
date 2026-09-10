//! Shared modeled player-cast payload; GUID spelling is simulator-specific.
use rilua::Val;
use rilua::vm::state::LuaState;

use super::methods::create_string;
use super::script_helpers::fire_named_event_state;

pub(crate) fn player_cast_args(
    cast_id: u32,
    spell_id: u32,
    mut lua_string: impl FnMut(&str) -> Val,
) -> [Val; 4] {
    let guid = format!("Cast-Sim-{cast_id}");
    [
        lua_string("player"),
        lua_string(&guid),
        Val::Num(spell_id as f64),
        Val::Num(cast_id as f64),
    ]
}

pub(crate) fn fire_player_cast_start(state: &mut LuaState, cast_id: u32, spell_id: u32) {
    let args = player_cast_args(cast_id, spell_id, |text| create_string(state, text));
    fire_named_event_state(state, "UNIT_SPELLCAST_START", &args);
}
