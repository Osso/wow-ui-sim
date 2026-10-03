//! Retail 12.0.5 journal spell lookup over existing mount records.
//! Aliases use the shared explicit registry, not inferred name or link grammar.
//! Native AllowedWhenTainted secret-input permissions remain unmodeled.

use super::c_spell::read_public_spell_identifier_at;
use super::helpers::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_MountJournal")?;
    table_set_rust_fn_static(state, namespace, "GetMountFromSpell", get_mount_from_spell)
}

fn get_mount_from_spell(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = read_public_spell_identifier_at(state, 1, "C_MountJournal.GetMountFromSpell")?;
    let mount_id = match spell_id {
        Some(spell_id) => {
            let sim = borrow_state(state)?;
            sim.world
                .mounts
                .iter()
                .find(|mount| mount.spell_id == spell_id)
                .map(|mount| mount.mount_id)
        }
        None => None,
    };
    state.push(mount_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}
