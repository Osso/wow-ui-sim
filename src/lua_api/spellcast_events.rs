//! Shared modeled player-cast payload; GUID spelling is simulator-specific.
use rilua::Val;
use rilua::vm::state::LuaState;

use super::methods::create_string;
use super::script_helpers::fire_named_event_state;

pub(crate) fn cast_guid(cast_id: u32) -> String {
    format!("Cast-Sim-{cast_id}")
}

pub(crate) fn player_cast_args(
    cast_id: u32,
    spell_id: u32,
    mut lua_string: impl FnMut(&str) -> Val,
) -> [Val; 4] {
    let guid = cast_guid(cast_id);
    [
        lua_string("player"),
        lua_string(&guid),
        Val::Num(spell_id as f64),
        Val::Num(cast_id as f64),
    ]
}

pub(crate) fn fire_player_cast_start(state: &mut LuaState, cast_id: u32, spell_id: u32) {
    #[cfg(feature = "player-cast-durations")]
    {
        // A replacement callback can supersede this producer before it publishes START.
        if !is_current_cast(state, cast_id) {
            return;
        }
        crate::lua_api::channeling::cancel_for_cast(state)
            .expect("registered cast producer has simulator state");
        if !is_current_cast(state, cast_id) {
            return;
        }
    }
    fire_player_cast_event(state, "UNIT_SPELLCAST_START", cast_id, spell_id);
}

#[cfg(feature = "player-cast-durations")]
fn is_current_cast(state: &LuaState, cast_id: u32) -> bool {
    super::methods::borrow_state(state)
        .expect("registered cast producer has simulator state")
        .casting
        .as_ref()
        .is_some_and(|cast| cast.cast_id == cast_id)
}

/// Discard a deferred specialization only when replacing its owning cast.
#[cfg(feature = "player-cast-durations")]
pub(crate) fn clear_replaced_specialization(sim: &mut super::SimState) {
    if sim
        .casting
        .as_ref()
        .is_some_and(|cast| cast.spell_id == crate::c_api::c_spec::SPEC_ACTIVATION_SPELL_ID)
    {
        sim.player.pending_spec_change = None;
    }
}

pub(crate) fn fire_player_cast_event(
    state: &mut LuaState,
    event: &str,
    cast_id: u32,
    spell_id: u32,
) {
    let args = player_cast_args(cast_id, spell_id, |text| create_string(state, text));
    fire_named_event_state(state, event, &args);
}

/// Self-cancel policy: interruption first, then STOP for non-interruption listeners.
#[cfg(feature = "player-cast-durations")]
pub(crate) fn fire_player_cast_interrupted(
    state: &mut LuaState,
    cast_id: u32,
    spell_id: u32,
    interrupted_by: &str,
) {
    let [unit, guid, spell, bar_id] =
        player_cast_args(cast_id, spell_id, |text| create_string(state, text));
    let actor = create_string(state, interrupted_by);
    fire_named_event_state(
        state,
        "UNIT_SPELLCAST_INTERRUPTED",
        &[unit, guid, spell, actor, bar_id],
    );
    // Recreate Lua values after callbacks; IDs still describe the canceled cast.
    let stopped = player_cast_args(cast_id, spell_id, |text| create_string(state, text));
    fire_named_event_state(state, "UNIT_SPELLCAST_STOP", &stopped);
}
