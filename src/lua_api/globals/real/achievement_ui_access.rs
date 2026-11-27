use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

const NAME: &str = "CanShowAchievementUI";
const COMPLETED_ANY_NAME: &str = "HasCompletedAnyAchievement";

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !is_published_for_active_profile() {
        return Ok(());
    }
    LuaApiMut::register_function(lua, NAME, can_show_achievement_ui)?;
    LuaApiMut::register_function(lua, COMPLETED_ANY_NAME, has_completed_any_achievement)?;
    Ok(())
}

fn is_published_for_active_profile() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

fn can_show_achievement_ui(state: &mut LuaState) -> LuaResult<u32> {
    let can_show = borrow_state(state)?.can_show_achievement_ui;
    state.push(Val::Bool(can_show));
    Ok(1)
}

fn has_completed_any_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let has_completed = !borrow_state(state)?.world.earned_achievements.is_empty();
    state.push(Val::Bool(has_completed));
    Ok(1)
}
