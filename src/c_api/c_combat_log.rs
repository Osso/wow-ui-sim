//! Forever combat-log publication boundaries from the pinned API documentation.
//!
//! Current-event data belongs to Internal/Secure, not public C_CombatLog. Mark
//! the public member absent before generic namespace lookup can synthesize it.

#[cfg(feature = "client-wowforever")]
use crate::lua_api::methods::{create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn register_restriction(state: &mut LuaState) -> LuaResult<()> {
    let ns = super::ensure_namespace(state, "C_CombatLog")?;
    crate::lua_bridge::table_set_rust_fn_static(state, ns, "IsCombatLogRestricted", |s| {
        let restricted = crate::lua_api::methods::borrow_state(s)?.combat_log_restricted;
        s.push(Val::Bool(restricted));
        Ok(1)
    })
}

#[cfg(feature = "client-wowforever")]
pub(crate) fn register_publication(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_CombatLog")?;
    let removed = create_table(state);
    table_set(state, removed, "GetCurrentEventInfo", Val::Bool(true));
    table_set(state, Val::Table(public), "__wow_removed_keys", removed);
    Ok(())
}
