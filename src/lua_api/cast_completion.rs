//! Cast completion, healing, and spec change logic.

use rilua::Val;

#[cfg(all(test, feature = "player-cast-durations"))]
#[path = "../iced_app/casting/duration_tests.rs"]
mod duration_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
#[path = "../iced_app/casting/input_tests.rs"]
mod input_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
#[path = "../iced_app/casting/interrupted_tests.rs"]
mod interrupted_tests;
#[cfg(all(test, feature = "retail-12-1-0"))]
#[path = "../iced_app/casting/tests.rs"]
mod tests;

/// Check if a cast has completed and extract its info, clearing state.
pub(crate) fn extract_completed_cast(
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
pub(crate) fn fire_cast_complete_events(
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
pub(crate) fn apply_spell_effect(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    env: &crate::lua_api::WowLuaEnv,
    spell_id: u32,
) {
    match crate::lua_api::game_data::apply_spell_to_state(state, spell_id) {
        Some(crate::lua_api::game_data::SpellEffectResult::UnitHealthChanged(unit_id)) => {
            let _ = env.fire_event_with_args("UNIT_HEALTH", &[env.lua_string(&unit_id)]);
        }
        Some(crate::lua_api::game_data::SpellEffectResult::PlayerAurasChanged) => {
            if let Err(error) = env.fire_unit_aura_full_update("player") {
                crate::logging::eprintln_elapsed(&format!("[UNIT_AURA] payload: {error}"))
            }
        }
        None => {}
    }
}

/// If a spec change was pending, apply it and fire PLAYER_SPECIALIZATION_CHANGED.
pub(crate) fn apply_spec_change(
    state: &std::rc::Rc<std::cell::RefCell<crate::lua_api::SimState>>,
    env: &crate::lua_api::WowLuaEnv,
) {
    let changed = {
        let mut s = state.borrow_mut();
        s.player.pending_spec_change.take().map(|idx| {
            s.player.active_spec_index = idx;
            #[cfg(feature = "retail-12-0-5")]
            {
                let spec_id = crate::specializations::specs_for_class(s.player.class_index as u32)
                    .nth((idx - 1) as usize)
                    .expect("pending specialization was validated at cast initiation")
                    .id;
                s.talents.switch_to_spec(spec_id);
            }
        })
    };
    if changed.is_some() {
        let _ =
            env.fire_event_with_args("PLAYER_SPECIALIZATION_CHANGED", &[env.lua_string("player")]);
        #[cfg(feature = "retail-12-0-5")]
        if let Err(error) = env.fire_event("ACTIVE_PLAYER_SPECIALIZATION_CHANGED") {
            eprintln!("Failed to publish completed specialization refresh: {error}");
        }
    }
}

/// Advance the modeled cast lifecycle. GUI and headless callers share this
/// producer; callers may tick before the deadline without completing a cast.
pub fn tick_casting(env: &crate::lua_api::WowLuaEnv) {
    if let Some((cast_id, spell_id)) = extract_completed_cast(env.state()) {
        fire_cast_complete_events(env, cast_id, spell_id);
        apply_spell_effect(env.state(), env, spell_id);
        apply_spec_change(env.state(), env);
    }
}
