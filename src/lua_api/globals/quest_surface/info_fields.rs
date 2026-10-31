//! Field writers for `C_QuestLog.GetInfo()` result tables.

use crate::lua_api::methods::{create_string, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn write_quest_header_fields(state: &mut LuaState, info: Val, title: &str) {
    let title_val = create_string(state, title);
    table_set(state, info, "title", title_val);
    table_set(state, info, "questID", Val::Num(0.0));
    table_set(state, info, "isHeader", Val::Bool(true));
    table_set(state, info, "isCollapsed", Val::Bool(false));
    table_set(state, info, "isTask", Val::Bool(false));
    table_set(state, info, "isBounty", Val::Bool(false));
    table_set(state, info, "isHidden", Val::Bool(false));
    table_set(state, info, "isOnMap", Val::Bool(false));
}

pub(super) fn write_quest_entry_fields(
    state: &mut LuaState,
    info: Val,
    quest_id: i32,
    title: &str,
    campaign_id: i32,
) {
    let title_val = create_string(state, title);
    table_set(state, info, "title", title_val);
    table_set(state, info, "questID", Val::Num(quest_id as f64));
    table_set(state, info, "campaignID", Val::Num(f64::from(campaign_id)));
    table_set(state, info, "level", Val::Num(80.0));
    table_set(state, info, "difficultyLevel", Val::Num(80.0));
    table_set(state, info, "suggestedGroup", Val::Num(0.0));
    table_set(state, info, "isHeader", Val::Bool(false));
    table_set(state, info, "isCollapsed", Val::Bool(false));
    table_set(state, info, "isTask", Val::Bool(false));
    table_set(state, info, "isBounty", Val::Bool(false));
    table_set(state, info, "isStory", Val::Bool(false));
    table_set(state, info, "isOnMap", Val::Bool(true));
    table_set(state, info, "hasLocalPOI", Val::Bool(false));
    table_set(state, info, "isHidden", Val::Bool(false));
    table_set(state, info, "isAutoComplete", Val::Bool(false));
    table_set(state, info, "overridesSortOrder", Val::Bool(false));
    table_set(state, info, "startEvent", Val::Bool(false));
    table_set(state, info, "isScaling", Val::Bool(false));
    table_set(state, info, "readyForTranslation", Val::Bool(false));
}
