//! Creature identifiers extracted from simulator GUID strings.

/// Preserve the GUID parsing policy shared with the legacy UnitCreatureID API.
pub(crate) fn creature_id_from_guid(guid: &str) -> Option<i32> {
    let mut parts = guid.split('-');
    if parts.next()? != "Creature" {
        return None;
    }
    parts.nth(4)?.parse().ok()
}

#[cfg(feature = "retail-12-0-0")]
pub(crate) fn get_creature_id(state: &mut rilua::vm::state::LuaState) -> rilua::LuaResult<u32> {
    use crate::lua_bridge::FromStack;

    let guid = String::from_stack(state, 1)?;
    let result =
        creature_id_from_guid(&guid).map_or(rilua::Val::Nil, |id| rilua::Val::Num(f64::from(id)));
    state.push(result);
    Ok(1)
}
