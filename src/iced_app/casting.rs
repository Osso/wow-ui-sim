//! Cast completion, healing, and spec change logic.

use rilua::Val;

#[cfg(all(test, feature = "player-cast-durations"))]
mod duration_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
mod input_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
mod interrupted_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
mod tests;

/// Check if a cast has completed and extract its info, clearing state.
pub(super) fn extract_completed_cast(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
) -> Option<(u32, u32)> {
    let mut s = state.borrow_mut();
    let c = s.casting.as_ref()?;
    let now = s.start_time.elapsed().as_secs_f64();
    if now < c.end_time {
        return None;
    }
    let cast_id = c.cast_id;
    let spell_id = c.spell_id;
    s.casting = None;
    Some((cast_id, spell_id))
}

/// Fire UNIT_SPELLCAST_STOP and UNIT_SPELLCAST_SUCCEEDED events.
pub(super) fn fire_cast_complete_events(
    env: &crate::lua_api::WowLuaEnv,
    cast_id: u32,
    spell_id: u32,
) {
    let args = crate::lua_api::spellcast_events::player_cast_args(cast_id, spell_id, |text| {
        env.lua_string(text)
    });
    let _ = env.fire_event_with_args("UNIT_SPELLCAST_STOP", &args);
    let _ = env.fire_event_with_args("UNIT_SPELLCAST_SUCCEEDED", &args);
    if crate::lua_api::globals::profession_data::get_recipe(spell_id as i32).is_some() {
        let _ = env.fire_event_with_args("UPDATE_TRADESKILL_CAST_STOPPED", &[Val::Bool(false)]);
    }
}

/// Apply spell effects (damage or healing) based on spell target type.
pub(super) fn apply_spell_effect(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    env: &crate::lua_api::WowLuaEnv,
    spell_id: u32,
) {
    match crate::lua_api::game_data::apply_spell_to_state(state, spell_id) {
        Some(crate::lua_api::game_data::SpellEffectResult::UnitHealthChanged(unit_id)) => {
            let _ = env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string(&unit_id)]);
        }
        Some(crate::lua_api::game_data::SpellEffectResult::PlayerAurasChanged) => {
            let unit = env.lua_string("player");
            if let Ok(info) = env.eval::<rilua::Val>("return { isFullUpdate = true }") {
                let _ = env.fire_event_with_args("UNIT_AURA", &[unit, info]);
            }
        }
        None => {}
    }
}

/// If a spec change was pending, apply it and fire PLAYER_SPECIALIZATION_CHANGED.
pub(super) fn apply_spec_change(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    env: &crate::lua_api::WowLuaEnv,
) {
    let changed = {
        let mut s = state.borrow_mut();
        s.player.pending_spec_change.take().map(|idx| {
            s.player.active_spec_index = idx;
        })
    };
    if changed.is_some() {
        let _ =
            env.fire_event_with_args("PLAYER_SPECIALIZATION_CHANGED", &[env.lua_string("player")]);
    }
}
