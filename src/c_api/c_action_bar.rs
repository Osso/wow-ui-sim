//! State-backed action cooldown queries; other action-bar APIs retain their existing owners.

use crate::lua_api::globals::action_bar_api::spell_cooldown_times;
use crate::lua_api::globals::lua_duration_object::push_timed_duration_object;
use crate::lua_api::methods::{borrow_state, create_table_with_capacity, table_set};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

// These pinned profiles return SpellCooldownInfo; earlier profiles retain the
// four-field ActionBarCooldownInfo payload. This does not date its introduction.
const HAS_ACTIVE_FIELD: bool = cfg!(any(
    feature = "retail-12-1-0",
    feature = "client-wowforever"
));
const ACTION_COOLDOWN_HASH_FIELDS: usize = 4 + HAS_ACTIVE_FIELD as usize;

pub(crate) fn get_action_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let (start, duration) = read_action_cooldown(state)?;
    let is_enabled = true;
    let info = create_table_with_capacity(state, ACTION_COOLDOWN_HASH_FIELDS);
    table_set(state, info, "startTime", Val::Num(start));
    table_set(state, info, "duration", Val::Num(duration));
    table_set(state, info, "isEnabled", Val::Bool(is_enabled));
    table_set(state, info, "modRate", Val::Num(1.0));
    if HAS_ACTIVE_FIELD {
        let is_active = is_enabled && start != 0.0 && duration != 0.0;
        table_set(state, info, "isActive", Val::Bool(is_active));
    }
    state.push(info);
    Ok(1)
}

pub(crate) fn read_action_cooldown(state: &LuaState) -> LuaResult<(f64, f64)> {
    let slot = match stack_val(state, 1) {
        Val::Num(number) if number >= 0.0 => Some(number as u32),
        _ => None,
    };
    let sim = borrow_state(state)?;
    let now = sim.start_time.elapsed().as_secs_f64();
    Ok(slot
        .and_then(|slot| sim.action_bars.get(&slot).copied())
        .map(|spell_id| spell_cooldown_times(&sim, spell_id, now))
        .unwrap_or((0.0, 0.0)))
}

pub(crate) fn get_action_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let (start, seconds) = read_action_cooldown(state)?;
    push_timed_duration_object(state, start, seconds)
}
