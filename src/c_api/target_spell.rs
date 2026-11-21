//! INFERRED R88cn: report only flags supplied for the current pending spell.
//! With no pending spell, these queries return false; native default behavior is unknown.

use crate::c_api::helpers::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PendingTargetSpell {
    spell_id: Option<i32>,
    is_enchanting: bool,
    jumps_upgrade_track: bool,
    replaces_bonus_tree: bool,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    let spell = ensure_namespace(state, "C_Spell")?;
    table_set_rust_fn_static(state, spell, "TargetSpellIsEnchanting", is_enchanting)?;
    table_set_rust_fn_static(
        state,
        spell,
        "TargetSpellJumpsUpgradeTrack",
        jumps_upgrade_track,
    )?;
    table_set_rust_fn_static(
        state,
        spell,
        "TargetSpellReplacesBonusTree",
        replaces_bonus_tree,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPendingTargetSpell", set_pending_target_spell)
}

fn set_pending_target_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = i32::from_stack(state, 1)?;
    let is_enchanting = bool::from_stack(state, 2)?;
    let jumps_upgrade_track = bool::from_stack(state, 3)?;
    let replaces_bonus_tree = bool::from_stack(state, 4)?;
    borrow_state_mut(state)?.pending_target_spell = PendingTargetSpell {
        spell_id: Some(spell_id),
        is_enchanting,
        jumps_upgrade_track,
        replaces_bonus_tree,
    };
    Ok(0)
}

fn is_enchanting(state: &mut LuaState) -> LuaResult<u32> {
    push_pending_flag(state, |pending| pending.is_enchanting)
}

fn jumps_upgrade_track(state: &mut LuaState) -> LuaResult<u32> {
    push_pending_flag(state, |pending| pending.jumps_upgrade_track)
}

fn replaces_bonus_tree(state: &mut LuaState) -> LuaResult<u32> {
    push_pending_flag(state, |pending| pending.replaces_bonus_tree)
}

fn push_pending_flag(
    state: &mut LuaState,
    select: impl FnOnce(&PendingTargetSpell) -> bool,
) -> LuaResult<u32> {
    let pending = borrow_state(state)?.pending_target_spell;
    state.push(Val::Bool(
        pending.spell_id.is_some_and(|_| select(&pending)),
    ));
    Ok(1)
}
