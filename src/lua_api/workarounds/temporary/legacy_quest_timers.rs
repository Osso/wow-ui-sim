//! Empty legacy global on profiles outside Forever's modeled C_QuestLog timer API.
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult};

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        LuaApiMut::register_function(lua, "GetQuestTimers", empty_quest_timers)?;
    }
    Ok(())
}

// Empty vararg, not one nil or the number zero: no timer rows may be rendered.
fn empty_quest_timers(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
