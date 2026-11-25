//! Temporary `C_PerksActivities` state surface.
//!
//! Traveler's Log activity data is not backed by a simulator subsystem yet.
//! Keep the small mutable Lua surface explicit as temporary compatibility state.

const PERKS_ACTIVITIES_STATE_LUA: &str = r#"
if type(C_PerksActivities) ~= "table" then
    C_PerksActivities = {}
end

local state = rawget(_G, "__wow_perks_activities_state")
if type(state) ~= "table" then
    state = {
        trackedIDs = {},
        removeCount = 0,
        lastRemovedID = nil,
        activityInfoByID = {},
        chatLinkByID = {},
        activitiesInfo = nil,
        allTags = nil,
        pendingCompletion = nil,
    }
    rawset(_G, "__wow_perks_activities_state", state)
end

C_PerksActivities._state = state

local function TrackedIDs()
    if type(state.trackedIDs) ~= "table" then
        state.trackedIDs = {}
    end
    return state.trackedIDs
end

if rawget(C_PerksActivities, "AddTrackedPerksActivity") == nil then
    function C_PerksActivities.AddTrackedPerksActivity(id)
        table.insert(TrackedIDs(), tonumber(id) or id)
    end
end

if rawget(C_PerksActivities, "ClearPerksActivitiesPendingCompletion") == nil then
    function C_PerksActivities.ClearPerksActivitiesPendingCompletion()
        state.pendingCompletion = { pendingIDs = {} }
    end
end

if rawget(C_PerksActivities, "GetAllPerksActivityTags") == nil then
    function C_PerksActivities.GetAllPerksActivityTags()
        if type(state.allTags) == "table" then
            return state.allTags
        end
        return { tagName = {} }
    end
end

if rawget(C_PerksActivities, "GetPerksActivitiesInfo") == nil then
    function C_PerksActivities.GetPerksActivitiesInfo()
        if type(state.activitiesInfo) == "table" then
            return state.activitiesInfo
        end
        return {
            activePerksMonth = 1,
            displayMonthName = "",
            secondsRemaining = 0,
            activities = {},
            thresholds = {},
        }
    end
end

if rawget(C_PerksActivities, "GetPerksActivitiesPendingCompletion") == nil then
    function C_PerksActivities.GetPerksActivitiesPendingCompletion()
        if type(state.pendingCompletion) == "table" then
            return state.pendingCompletion
        end
        return { pendingIDs = {} }
    end
end

if rawget(C_PerksActivities, "GetPerksActivityChatLink") == nil then
    function C_PerksActivities.GetPerksActivityChatLink(id)
        local lookupID = tonumber(id) or id
        local info = state.chatLinkByID and state.chatLinkByID[lookupID]
        return info or ""
    end
end

if rawget(C_PerksActivities, "GetPerksActivityInfo") == nil then
    function C_PerksActivities.GetPerksActivityInfo(id)
        local lookupID = tonumber(id) or id
        return state.activityInfoByID and state.activityInfoByID[lookupID] or nil
    end
end

if rawget(C_PerksActivities, "GetPerksUIThemePrefix") == nil then
    function C_PerksActivities.GetPerksUIThemePrefix()
        return ""
    end
end

if rawget(C_PerksActivities, "GetTrackedPerksActivities") == nil then
    function C_PerksActivities.GetTrackedPerksActivities()
        return { trackedIDs = TrackedIDs() }
    end
end

if rawget(C_PerksActivities, "RemoveTrackedPerksActivity") == nil then
    function C_PerksActivities.RemoveTrackedPerksActivity(id)
        local trackedIDs = TrackedIDs()
        local targetID = tonumber(id) or id
        for index = #trackedIDs, 1, -1 do
            if tonumber(trackedIDs[index]) == targetID then
                table.remove(trackedIDs, index)
                state.removeCount = (tonumber(state.removeCount) or 0) + 1
                state.lastRemovedID = targetID
                return true
            end
        end
        return false
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !crate::c_api::perks_activities::active() {
        lua.exec(PERKS_ACTIVITIES_STATE_LUA)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    use crate::lua_api::WowLuaEnv;

    #[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
    #[test]
    fn installs_tracking_defaults_and_lookup_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                C_PerksActivities.AddTrackedPerksActivity("42")
                C_PerksActivities.AddTrackedPerksActivity(77)
                local tracked = C_PerksActivities.GetTrackedPerksActivities().trackedIDs
                if #tracked ~= 2 or tracked[1] ~= 42 or tracked[2] ~= 77 then
                    return "bad_tracked"
                end
                if not C_PerksActivities.RemoveTrackedPerksActivity("42") then
                    return "remove_failed"
                end
                if C_PerksActivities.RemoveTrackedPerksActivity(404) then
                    return "removed_missing"
                end
                if C_PerksActivities._state.removeCount ~= 1 or C_PerksActivities._state.lastRemovedID ~= 42 then
                    return "bad_remove_state"
                end
                C_PerksActivities._state.chatLinkByID[77] = "|Hperksactivity:77|h[test]|h"
                if C_PerksActivities.GetPerksActivityChatLink("77") ~= "|Hperksactivity:77|h[test]|h" then
                    return "bad_chat_link"
                end
                C_PerksActivities._state.activityInfoByID[77] = { ID = 77, name = "Test" }
                if C_PerksActivities.GetPerksActivityInfo("77").name ~= "Test" then
                    return "bad_activity_info"
                end
                C_PerksActivities.ClearPerksActivitiesPendingCompletion()
                if #C_PerksActivities.GetPerksActivitiesPendingCompletion().pendingIDs ~= 0 then
                    return "bad_pending"
                end
                local activitiesInfo = C_PerksActivities.GetPerksActivitiesInfo()
                if activitiesInfo.activePerksMonth ~= 1 or type(activitiesInfo.activities) ~= "table" then
                    return "bad_activities_info"
                end
                if type(C_PerksActivities.GetAllPerksActivityTags().tagName) ~= "table" then
                    return "bad_tags"
                end
                if C_PerksActivities.GetPerksUIThemePrefix() ~= "" then
                    return "bad_theme"
                end
                return "ok"
                "#,
            )
            .expect("perks activities probe should run");

        assert_eq!(result, "ok");
    }
}
