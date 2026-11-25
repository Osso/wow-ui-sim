//! Global PvP talent info lookup used by Blizzard talent and inspect frames.
//!
//! VERIFIED (R88eu): tuple count, types, missing-row result, and zero-ID result.
//! INFERRED (R88eu): tuple order. Optional context arguments are not modeled.

use crate::c_api::bad_argument;
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const PVP_TALENT_INFO_BY_ID: &str = "GetPvpTalentInfoByID";
const PVP_TALENT_INFO_BY_ID_USAGE: &str =
    "Usage: GetPvpTalentInfoByID(talentID [, specGroupIndex, isInspect, inspectUnit])";
const PVP_TALENT_INFO_BY_ID_RETURN_COUNT: u32 = 11;

/// `GetPvpTalentInfoByID(talentID [, specGroupIndex, isInspect, inspectUnit])`.
///
/// VERIFIED R88eu: return eleven values with the established field types;
/// missing rows and talent ID zero return no values. INFERRED R88eu: the order
/// is `(talentID, name, icon, selected, available, spellID, unlocked, tier,
/// column, known, grantedByAura)`; tier and column default to zero.
pub fn publish_pvp_talent_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let talent_id = i32::from_stack(state, 1)
        .map_err(|_| bad_argument(1, PVP_TALENT_INFO_BY_ID, PVP_TALENT_INFO_BY_ID_USAGE))?;
    if talent_id == 0 {
        return Ok(0);
    }
    let record = borrow_state(state)?
        .pvp_talents
        .info_by_id
        .get(&talent_id)
        .cloned();
    let Some(record) = record else {
        return Ok(0);
    };

    push_pvp_talent_info_by_id_result(state, &record);
    Ok(PVP_TALENT_INFO_BY_ID_RETURN_COUNT)
}

fn push_pvp_talent_info_by_id_result(
    state: &mut LuaState,
    record: &crate::c_api::c_spec::PvpTalentInfoRecord,
) {
    let name = Val::Str(state.gc.intern_string(record.name.as_bytes()));
    for value in [
        Val::Num(f64::from(record.talent_id)),
        name,
        record
            .icon
            .map_or(Val::Nil, |icon| Val::Num(f64::from(icon))),
        Val::Bool(record.selected),
        Val::Bool(record.available),
        Val::Num(f64::from(record.spell_id)),
        record.unlocked.map_or(Val::Nil, Val::Bool),
        Val::Num(f64::from(record.tier)),
        Val::Num(f64::from(record.column)),
        Val::Bool(record.known),
        Val::Bool(record.granted_by_aura),
    ] {
        state.push(value);
    }
}

pub fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    use rilua::LuaApiMut;
    let state = lua.state_mut();
    table_set_rust_fn_static(
        state,
        state.global,
        PVP_TALENT_INFO_BY_ID,
        publish_pvp_talent_info_by_id,
    )?;
    Ok(())
}
