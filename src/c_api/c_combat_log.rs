//! Forever combat-log publication boundaries from the pinned API documentation.
//!
//! Current-event data belongs to Internal/Secure, not public C_CombatLog. Mark
//! the public member absent before generic namespace lookup can synthesize it.

use crate::lua_api::methods::{create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn register_publication(state: &mut LuaState) -> LuaResult<()> {
    let public = super::ensure_namespace(state, "C_CombatLog")?;
    let removed = create_table(state);
    table_set(state, removed, "GetCurrentEventInfo", Val::Bool(true));
    table_set(state, Val::Table(public), "__wow_removed_keys", removed);
    Ok(())
}
