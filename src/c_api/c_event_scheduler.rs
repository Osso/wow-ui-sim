//! `C_EventScheduler` Lua-owned host state and detached query DTOs.
//!
//! Retains existing demo inputs; host updates `_state` lists before publishing an event.

const EVENT_SCHEDULER_STATE_LUA: &str = r#"
if type(C_EventScheduler) ~= "table" then
    C_EventScheduler = {}
end

local useRetailEventDisplayInfo =
    rawget(_G, "__wow_event_scheduler_retail_display_info") == true
rawset(_G, "__wow_event_scheduler_retail_display_info", nil)

local function EventSchedulerDisplayInfo()
    if not useRetailEventDisplayInfo then
        return {}
    end
    return {
        hideDescription = false,
        hideTimeLeft = false,
        overrideAtlas = nil,
        overrideTooltipWidgetSetID = nil,
    }
end

-- Returned DTOs never expose mutable host state or nested display records.
local function Snapshot(value)
    if type(value) ~= "table" then return value end
    local copy = {}
    for key, child in pairs(value) do copy[key] = Snapshot(child) end
    return copy
end

-- INFERRED: retain simulator demo events, not live server records.
local function EventSchedulerSeedState()
    local now = (os and type(os.time) == "function") and os.time() or 0
    return {
        canShowEvents = nil,
        suppressDisplay = false,
        ongoingEvents = {
            {
                areaPoiID = 1001,
                eventID = 1001,
                eventKey = "warsong-gulch",
                displayInfo = EventSchedulerDisplayInfo(),
                rewardsClaimed = false,
            },
            {
                areaPoiID = 1002,
                eventID = 1002,
                eventKey = "cinderbrew-meadery",
                displayInfo = EventSchedulerDisplayInfo(),
                rewardsClaimed = false,
            },
        },
        scheduledEvents = {
            {
                areaPoiID = 1001,
                eventID = 2001,
                eventKey = "pvp-brawl-blitz",
                startTime = now + 3600,
                endTime = now + 7200,
                duration = 3600,
                hasReminder = false,
                rewardsClaimed = false,
                displayInfo = EventSchedulerDisplayInfo(),
            },
            {
                areaPoiID = 1004,
                eventID = 2002,
                eventKey = "darkmoon-island",
                startTime = now + 7200,
                endTime = now + 10800,
                duration = 3600,
                hasReminder = true,
                rewardsClaimed = false,
                displayInfo = EventSchedulerDisplayInfo(),
            },
        },
        reminders = {},
    }
end

if type(rawget(C_EventScheduler, "_state")) ~= "table" then
    C_EventScheduler._state = EventSchedulerSeedState()
end

if rawget(C_EventScheduler, "CanShowEvents") == nil then
    function C_EventScheduler.CanShowEvents()
        local state = C_EventScheduler._state
        if type(state) ~= "table" then
            return false
        end
        if state.canShowEvents ~= nil then
            return state.canShowEvents == true
        end
        if state.suppressDisplay == true then
            return false
        end
        return #(state.ongoingEvents or {}) > 0 or #(state.scheduledEvents or {}) > 0
    end
end

if rawget(C_EventScheduler, "RequestEvents") == nil then
    function C_EventScheduler.RequestEvents()
        C_EventScheduler._state = EventSchedulerSeedState()
    end
end

if rawget(C_EventScheduler, "GetOngoingEvents") == nil then
    function C_EventScheduler.GetOngoingEvents()
        return Snapshot(C_EventScheduler._state.ongoingEvents)
    end
end

if rawget(C_EventScheduler, "GetScheduledEvents") == nil then
    function C_EventScheduler.GetScheduledEvents()
        return Snapshot(C_EventScheduler._state.scheduledEvents)
    end
end

if rawget(C_EventScheduler, "HasData") == nil then
    function C_EventScheduler.HasData()
        local state = C_EventScheduler._state
        return #(state.ongoingEvents or {}) > 0 or #(state.scheduledEvents or {}) > 0
    end
end

if rawget(C_EventScheduler, "GetEventZoneName") == nil then
    function C_EventScheduler.GetEventZoneName(areaPoiID)
        local poi = C_AreaPoiInfo.GetAreaPOIInfo(nil, areaPoiID)
        return poi and poi.name or ""
    end
end

if rawget(C_EventScheduler, "GetEventUiMapID") == nil then
    function C_EventScheduler.GetEventUiMapID(areaPoiID)
        local poi = C_AreaPoiInfo.GetAreaPOIInfo(nil, areaPoiID)
        return (poi and poi.uiMapID) or 0
    end
end

if rawget(C_EventScheduler, "HasSavedReminders") == nil then
    function C_EventScheduler.HasSavedReminders()
        local reminders = C_EventScheduler._state.reminders or {}
        return next(reminders) ~= nil
    end
end

if rawget(C_EventScheduler, "SetReminder") == nil then
    function C_EventScheduler.SetReminder(eventKey)
        if eventKey ~= nil then
            C_EventScheduler._state.reminders[tostring(eventKey)] = true
        end
    end
end

if rawget(C_EventScheduler, "ClearReminder") == nil then
    function C_EventScheduler.ClearReminder(eventKey)
        if eventKey ~= nil then
            C_EventScheduler._state.reminders[tostring(eventKey)] = nil
        end
    end
end

"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
    ) {
        lua.exec("rawset(_G, '__wow_event_scheduler_retail_display_info', true)")?;
    }
    lua.exec(EVENT_SCHEDULER_STATE_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_seeded_events_and_reminders() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let expects_retail_display_info = matches!(
            crate::client_profile::ACTIVE,
            crate::client_profile::ClientProfile::Retail
        );
        env.exec(if expects_retail_display_info {
            "rawset(_G, '__test_event_scheduler_retail_display_info', true)"
        } else {
            "rawset(_G, '__test_event_scheduler_retail_display_info', false)"
        })
        .expect("test profile marker should install");

        let result: String = env
            .eval(
                r#"
                local expect_retail_display_info =
                    rawget(_G, "__test_event_scheduler_retail_display_info") == true
                if #C_EventScheduler.GetOngoingEvents() ~= 2 then
                    return "bad_ongoing"
                end
                local ongoing = C_EventScheduler.GetOngoingEvents()
                local scheduled = C_EventScheduler.GetScheduledEvents()
                if #scheduled ~= 2 then
                    return "bad_scheduled"
                end
                for _, event in ipairs(ongoing) do
                    local display_info = event.displayInfo
                    if expect_retail_display_info then
                        if type(display_info) ~= "table"
                            or type(display_info.hideDescription) ~= "boolean"
                            or display_info.hideDescription ~= false
                            or type(display_info.hideTimeLeft) ~= "boolean"
                            or display_info.hideTimeLeft ~= false
                            or display_info.overrideAtlas ~= nil
                            or display_info.overrideTooltipWidgetSetID ~= nil then
                            return "bad_ongoing_display_info"
                        end
                    elseif next(display_info) ~= nil then
                        return "changed_nonretail_ongoing_display_info"
                    end
                end
                for _, event in ipairs(scheduled) do
                    local display_info = event.displayInfo
                    if expect_retail_display_info then
                        if type(display_info) ~= "table"
                            or type(display_info.hideDescription) ~= "boolean"
                            or display_info.hideDescription ~= false
                            or type(display_info.hideTimeLeft) ~= "boolean"
                            or display_info.hideTimeLeft ~= false
                            or display_info.overrideAtlas ~= nil
                            or display_info.overrideTooltipWidgetSetID ~= nil then
                            return "bad_scheduled_display_info"
                        end
                    elseif next(display_info) ~= nil then
                        return "changed_nonretail_scheduled_display_info"
                    end
                end
                if type(scheduled[1].eventID) ~= "number" or scheduled[1].eventID ~= 2001
                    or type(scheduled[2].eventID) ~= "number" or scheduled[2].eventID ~= 2002 then
                    return "bad_scheduled_event_ids"
                end
                local first_event_id = scheduled[1].eventID
                local second_event_id = scheduled[2].eventID
                C_EventScheduler.RequestEvents()
                local refreshed_scheduled = C_EventScheduler.GetScheduledEvents()
                if refreshed_scheduled[1].eventID ~= first_event_id
                    or refreshed_scheduled[2].eventID ~= second_event_id then
                    return "unstable_scheduled_event_ids"
                end
                local refreshed_display_info = refreshed_scheduled[1].displayInfo
                if expect_retail_display_info then
                    if type(refreshed_display_info) ~= "table"
                        or type(refreshed_display_info.hideDescription) ~= "boolean"
                        or refreshed_display_info.hideDescription ~= false
                        or type(refreshed_display_info.hideTimeLeft) ~= "boolean"
                        or refreshed_display_info.hideTimeLeft ~= false
                        or refreshed_display_info.overrideAtlas ~= nil
                        or refreshed_display_info.overrideTooltipWidgetSetID ~= nil then
                        return "unstable_scheduled_display_info"
                    end
                elseif next(refreshed_display_info) ~= nil then
                    return "changed_nonretail_refreshed_display_info"
                end
                if not C_EventScheduler.CanShowEvents() then
                    return "not_visible"
                end
                C_EventScheduler.SetReminder("warsong-gulch")
                if not C_EventScheduler.HasSavedReminders() then
                    return "missing_reminder"
                end
                C_EventScheduler.ClearReminder("warsong-gulch")
                if C_EventScheduler.HasSavedReminders() then
                    return "stale_reminder"
                end
                if C_EventScheduler.GetEventZoneName(1001) ~= "Warsong Gulch" then
                    return "bad_zone_name"
                end
                if C_EventScheduler.GetEventUiMapID(1004) == 0 then
                    return "bad_map_id"
                end
                return "ok"
                "#,
            )
            .expect("event scheduler probe should run");

        assert_eq!(result, "ok");
    }
}
