//! Temporary encounter timeline/event state surface.
//!
//! Encounter timeline and event customization are seeded startup fixtures. Keep
//! them explicit until the simulator has a real encounter-event model.

#[cfg(not(feature = "retail-12-1-5"))]
const TIMELINE_DEMO_LUA: &str = r#"
if type(C_EncounterTimeline) ~= "table" then
    C_EncounterTimeline = {}
end
if type(C_EncounterEvents) ~= "table" then
    C_EncounterEvents = {}
end

if rawget(C_EncounterTimeline, "IsFeatureAvailable") == nil then
    function C_EncounterTimeline.IsFeatureAvailable()
        return true
    end
end
if rawget(C_EncounterTimeline, "IsFeatureEnabled") == nil then
    function C_EncounterTimeline.IsFeatureEnabled()
        return true
    end
end
if rawget(C_EncounterTimeline, "GetEventList") == nil then
    function C_EncounterTimeline.GetEventList()
        return { 1 }
    end
end
if rawget(C_EncounterTimeline, "GetEventInfo") == nil then
    function C_EncounterTimeline.GetEventInfo(eventID)
        if eventID ~= 1 then
            return nil
        end
        return {
            spellID = 19750,
            spellName = "Flash of Light",
        }
    end
end
if rawget(C_EncounterTimeline, "GetEventTimer") == nil then
    function C_EncounterTimeline.GetEventTimer(eventID)
        if eventID ~= 1 then
            return nil
        end
        local timer = { remaining = 12.5 }
        function timer:GetRemainingDuration()
            return self.remaining
        end
        return timer
    end
end
if rawget(C_EncounterTimeline, "GetEventTrack") == nil then
    function C_EncounterTimeline.GetEventTrack(eventID)
        if eventID ~= 1 then
            return nil, nil
        end
        return Enum.EncounterTimelineTrack.Short, 1
    end
end
if rawget(C_EncounterTimeline, "HasActiveEvents") == nil then
    function C_EncounterTimeline.HasActiveEvents()
        return true
    end
end
if rawget(C_EncounterTimeline, "HasVisibleEvents") == nil then
    function C_EncounterTimeline.HasVisibleEvents()
        return true
    end
end
if rawget(C_EncounterTimeline, "AddEditModeEvents") == nil then
    function C_EncounterTimeline.AddEditModeEvents()
        return 30
    end
end
if rawget(C_EncounterTimeline, "CancelEditModeEvents") == nil then
    function C_EncounterTimeline.CancelEditModeEvents()
    end
end

"#;

const ENCOUNTER_STATE_LUA: &str = r#"
if type(C_EncounterEvents) ~= "table" then C_EncounterEvents = {} end
local state = rawget(C_EncounterEvents, "_state")
if type(state) ~= "table" then
    state = {
        events = {
            [1] = {
                encounterEventID = 1,
                name = "Default Encounter Event",
                color = nil,
                sounds = {},
            },
        },
        nextSoundHandle = 1,
    }
    C_EncounterEvents._state = state
end
if type(state.events) ~= "table" then
    state.events = {}
end
if type(state.events[1]) ~= "table" then
    state.events[1] = {
        encounterEventID = 1,
        name = "Default Encounter Event",
        color = nil,
        sounds = {},
    }
end
if type(state.events[1].sounds) ~= "table" then
    state.events[1].sounds = {}
end
if type(state.nextSoundHandle) ~= "number" then
    state.nextSoundHandle = 1
end

local function EncounterEvent(eventID)
    eventID = tonumber(eventID)
    return eventID and C_EncounterEvents._state.events[eventID] or nil
end

if rawget(C_EncounterEvents, "GetEventList") == nil then
    function C_EncounterEvents.GetEventList()
        return { 1 }
    end
end
if rawget(C_EncounterEvents, "HasEventInfo") == nil then
    function C_EncounterEvents.HasEventInfo(eventID)
        return EncounterEvent(eventID) ~= nil
    end
end
if rawget(C_EncounterEvents, "GetEventInfo") == nil then
    function C_EncounterEvents.GetEventInfo(eventID)
        local event = EncounterEvent(eventID)
        if not event then
            return nil
        end
        local info = {
            encounterEventID = event.encounterEventID,
            name = event.name,
        }
        if event.color ~= nil then
            info.color = {
                r = event.color.r,
                g = event.color.g,
                b = event.color.b,
                a = event.color.a,
            }
        end
        return info
    end
end
if rawget(C_EncounterEvents, "SetEventColor") == nil then
    function C_EncounterEvents.SetEventColor(eventID, color)
        local event = EncounterEvent(eventID)
        if not event then
            return
        end
        if color == nil then
            event.color = nil
            return
        end
        event.color = {
            r = tonumber(color.r) or 0,
            g = tonumber(color.g) or 0,
            b = tonumber(color.b) or 0,
            a = tonumber(color.a) or 1,
        }
    end
end
if rawget(C_EncounterEvents, "GetEventColor") == nil then
    function C_EncounterEvents.GetEventColor(eventID)
        local event = EncounterEvent(eventID)
        if not event or event.color == nil then
            return nil
        end
        return {
            r = event.color.r,
            g = event.color.g,
            b = event.color.b,
            a = event.color.a,
        }
    end
end
if rawget(C_EncounterEvents, "SetEventSound") == nil then
    function C_EncounterEvents.SetEventSound(eventID, triggerID, sound)
        local event = EncounterEvent(eventID)
        triggerID = tonumber(triggerID)
        if not event or triggerID == nil then
            return
        end
        if type(event.sounds) ~= "table" then
            event.sounds = {}
        end
        if sound == nil then
            event.sounds[triggerID] = nil
            return
        end
        event.sounds[triggerID] = {
            file = tonumber(sound.file) or 0,
            channel = tostring(sound.channel or ""),
            volume = tonumber(sound.volume) or 0,
        }
    end
end
if rawget(C_EncounterEvents, "GetEventSound") == nil then
    function C_EncounterEvents.GetEventSound(eventID, triggerID)
        local event = EncounterEvent(eventID)
        triggerID = tonumber(triggerID)
        if not event or triggerID == nil or type(event.sounds) ~= "table" then
            return nil
        end
        local sound = event.sounds[triggerID]
        if sound == nil then
            return nil
        end
        return {
            file = sound.file,
            channel = sound.channel,
            volume = sound.volume,
        }
    end
end
if rawget(C_EncounterEvents, "PlayEventSound") == nil then
    function C_EncounterEvents.PlayEventSound(eventID, triggerID)
        local sound = C_EncounterEvents.GetEventSound(eventID, triggerID)
        if sound == nil then
            return nil
        end
        local handle = C_EncounterEvents._state.nextSoundHandle
        C_EncounterEvents._state.nextSoundHandle = handle + 1
        return handle
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    #[cfg(not(feature = "retail-12-1-5"))]
    lua.exec(TIMELINE_DEMO_LUA)?;
    lua.exec(ENCOUNTER_STATE_LUA)?;
    Ok(())
}

#[cfg(all(test, not(feature = "retail-12-1-5")))]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_timeline_and_event_customization_state() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if not C_EncounterTimeline.IsFeatureAvailable() or not C_EncounterTimeline.IsFeatureEnabled() then
                    return "feature_disabled"
                end
                if C_EncounterTimeline.GetEventList()[1] ~= 1 then
                    return "bad_timeline_list"
                end
                if C_EncounterTimeline.GetEventInfo(1).spellName ~= "Flash of Light" then
                    return "bad_timeline_info"
                end
                if C_EncounterTimeline.GetEventTimer(1):GetRemainingDuration() ~= 12.5 then
                    return "bad_timer"
                end
                local track, index = C_EncounterTimeline.GetEventTrack(1)
                if track ~= Enum.EncounterTimelineTrack.Short or index ~= 1 then
                    return "bad_track"
                end
                if not C_EncounterEvents.HasEventInfo(1) or C_EncounterEvents.HasEventInfo(99) then
                    return "bad_has_info"
                end
                local eventInfo = C_EncounterEvents.GetEventInfo(1)
                if eventInfo.encounterEventID ~= 1 then
                    return "bad_event_info"
                end
                C_EncounterEvents.SetEventColor(1, { r = 0.1, g = 0.2, b = 0.3, a = 0.4 })
                local color = C_EncounterEvents.GetEventColor(1)
                if color.r ~= 0.1 or color.g ~= 0.2 or color.b ~= 0.3 or color.a ~= 0.4 then
                    return "bad_color"
                end
                C_EncounterEvents.SetEventSound(1, 5, { file = 123, channel = "Master", volume = 0.5 })
                local sound = C_EncounterEvents.GetEventSound(1, 5)
                if sound.file ~= 123 or sound.channel ~= "Master" or sound.volume ~= 0.5 then
                    return "bad_sound"
                end
                if C_EncounterEvents.PlayEventSound(1, 5) ~= 1 then
                    return "bad_first_handle"
                end
                if C_EncounterEvents.PlayEventSound(1, 5) ~= 2 then
                    return "bad_second_handle"
                end
                C_EncounterEvents.SetEventColor(1, nil)
                C_EncounterEvents.SetEventSound(1, 5, nil)
                if C_EncounterEvents.GetEventColor(1) ~= nil or C_EncounterEvents.GetEventSound(1, 5) ~= nil then
                    return "bad_clear"
                end
                return "ok"
                "#,
            )
            .expect("encounter state probe should run");

        assert_eq!(result, "ok");
    }
}
