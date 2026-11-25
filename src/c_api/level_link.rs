//! Explicit host-supplied action and spell locks, not player-level progression.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn uses_rust_spell_locks() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_LevelLink")?;
    table_set_rust_fn_static(state, namespace, "IsActionLocked", is_action_locked)?;
    // VERIFIED shape in both pinned LevelLinkDocumentation.lua files, line 26.
    if uses_rust_spell_locks() {
        table_set_rust_fn_static(state, namespace, "IsSpellLocked", is_spell_locked)?;
    }
    Ok(())
}

fn is_action_locked(state: &mut LuaState) -> LuaResult<u32> {
    let is_locked = is_action_slot_locked(state)?;
    state.push(Val::Bool(is_locked));
    Ok(1)
}

fn is_action_slot_locked(state: &LuaState) -> LuaResult<bool> {
    let Val::Num(slot) = stack_val(state, 1) else {
        return Ok(false);
    };
    Ok(borrow_state(state)?
        .locked_action_slots
        .contains(&(slot as i32)))
}

fn is_spell_locked(state: &mut LuaState) -> LuaResult<u32> {
    let id = spell_id(state);
    // INFERRED: empty/invalid/unknown IDs stay unlocked; no level rules guessed.
    let locked = match id {
        Some(id) => borrow_state(state)?.level_link_locked_spells.contains(&id),
        None => false,
    };
    state.push(Val::Bool(locked));
    Ok(1)
}

fn spell_id(state: &LuaState) -> Option<i32> {
    let number = match stack_val(state, 1) {
        Val::Num(number) => Some(number),
        Val::Str(key) => state
            .gc
            .string_arena
            .get(key)
            .and_then(|value| super::parse_numeric_string(value.data())),
        _ => None,
    }?;
    // R88f.D.1 observes signed bounds and truncation on the adjacent Retail build.
    (f64::from(i32::MIN)..=f64::from(i32::MAX))
        .contains(&number)
        .then_some(number as i32)
}
