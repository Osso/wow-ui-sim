//! Spell proc membership supplied by the simulator host.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_SpellActivationOverlay")?;
    table_set_rust_fn_static(state, namespace, "IsSpellOverlayed", is_spell_overlayed)
}

fn is_spell_overlayed(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let active = borrow_state(state)?
        .spell_activation_overlays
        .contains(&spell_id);
    state.push(Val::Bool(active));
    Ok(1)
}
