//! Global PvP talent info lookup used by Blizzard talent and inspect frames.
//!
//! The native return tuple and missing-row behavior are INFERRED under R88eu;
//! native evidence is pending. Optional context arguments are not modeled.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{FromStack, LuaResult, Val};

const PVP_TALENT_INFO_BY_ID_RETURN_COUNT: u32 = 9;

/// `GetPvpTalentInfoByID(talentID [, specGroupIndex, isInspect, inspectUnit])`.
///
/// INFERRED R88eu (native evidence pending): return the supplied record as
/// `(talentID, name, icon, selected, available, spellID, unlocked, known,
/// grantedByAura)`. A missing record returns no values.
pub fn publish_pvp_talent_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let talent_id = i32::from_stack(state, 1)?;
    let record = borrow_state(state)?
        .pvp_talents
        .info_by_id
        .get(&talent_id)
        .cloned();
    let Some(record) = record else {
        return Ok(0);
    };

    let name = Val::Str(state.gc.intern_string(record.name.as_bytes()));
    for value in [
        Val::Num(f64::from(record.talent_id)),
        name,
        Val::Num(f64::from(record.icon)),
        Val::Bool(record.selected),
        Val::Bool(record.available),
        Val::Num(f64::from(record.spell_id)),
        Val::Bool(record.unlocked),
        Val::Bool(record.known),
        Val::Bool(record.granted_by_aura),
    ] {
        state.push(value);
    }
    Ok(PVP_TALENT_INFO_BY_ID_RETURN_COUNT)
}

pub fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    table_set_rust_fn_static(
        state,
        state.global,
        "GetPvpTalentInfoByID",
        publish_pvp_talent_info_by_id,
    )?;
    Ok(())
}
