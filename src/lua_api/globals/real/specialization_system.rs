use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// VERIFIED — Retail 12.1.5.70688 R88ad.11 normal publication value.
const SPECIALIZATION_SYSTEM_SELECTOR_RESULT: f64 = 1.0;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetSpecializationSystem", get_specialization_system)?;
    Ok(())
}

fn get_specialization_system(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(SPECIALIZATION_SYSTEM_SELECTOR_RESULT));
    Ok(1)
}
