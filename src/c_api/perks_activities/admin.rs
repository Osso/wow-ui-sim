use super::PerksActivities;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};
use std::collections::HashSet;

const INPUT_NAME: &str = "A_Admin.SetPerksActivities";

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetPerksActivities", set_activities)
}

fn set_activities(state: &mut LuaState) -> LuaResult<u32> {
    let json = String::from_stack(state, 1)?;
    let snapshot: PerksActivities = serde_json::from_str(&json)
        .map_err(|error| runtime_error(format!("{INPUT_NAME}: invalid JSON snapshot: {error}")))?;
    validate_snapshot(&snapshot)?;
    borrow_state_mut(state)?.perks_activities = snapshot;
    Ok(0)
}

fn validate_snapshot(snapshot: &PerksActivities) -> LuaResult<()> {
    validate_activity_ids(&snapshot.activities)?;
    validate_positive_ids(&snapshot.pending_completion, "pendingIDs")?;
    validate_positive_ids(&snapshot.tracked_ids, "trackedIDs")
}

fn validate_activity_ids(activities: &[super::PerksActivity]) -> LuaResult<()> {
    let mut seen = HashSet::with_capacity(activities.len());
    for activity in activities {
        if activity.id <= 0 || !seen.insert(activity.id) {
            return Err(invalid_input("activities.ID"));
        }
    }
    Ok(())
}

fn validate_positive_ids(ids: &[i32], field: &str) -> LuaResult<()> {
    if ids.iter().any(|id| *id <= 0) {
        return Err(invalid_input(field));
    }
    Ok(())
}

fn invalid_input(field: &str) -> rilua::LuaError {
    runtime_error(format!("{INPUT_NAME}: invalid {field}"))
}
