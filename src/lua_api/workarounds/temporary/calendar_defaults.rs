//! Temporary calendar defaults.
//!
//! Host-supplied day records and event positions are modeled in `c_api::c_calendar`;
//! native catalogs, invite, and holiday state remain unmodeled. Expose the remaining
//! inert compatibility surface without replacing modeled Calendar functions.

const CALENDAR_DEFAULTS_LUA: &str = r#"
C_Calendar = C_Calendar or __wow_namespace()

local function emptyDateInfo()
    return {
        year = 2024,
        month = 1,
        monthDay = 1,
        weekday = 1,
        hour = 0,
        minute = 0,
    }
end

if rawget(C_Calendar, "GetMonthInfo") == nil then
    function C_Calendar.GetMonthInfo(_offset)
        return {
            month = 1,
            year = 2024,
            numDays = 31,
            firstWeekday = 1,
        }
    end
end
if rawget(C_Calendar, "GetDefaultGuildFilter") == nil then
    function C_Calendar.GetDefaultGuildFilter()
        return {
            minLevel = 1,
            maxLevel = GetMaxLevelForLatestExpansion(),
            rank = 1,
        }
    end
end
if rawget(C_Calendar, "GetMaxCreateDate") == nil then
    function C_Calendar.GetMaxCreateDate()
        return emptyDateInfo()
    end
end
if rawget(C_Calendar, "GetMinDate") == nil then
    function C_Calendar.GetMinDate()
        return emptyDateInfo()
    end
end
if rawget(C_Calendar, "EventGetTypesDisplayOrdered") == nil then
    function C_Calendar.EventGetTypesDisplayOrdered()
        return {}
    end
end
if rawget(C_Calendar, "GetClubCalendarEvents") == nil then
    function C_Calendar.GetClubCalendarEvents()
        return {}
    end
end
if rawget(C_Calendar, "GetNumGuildEvents") == nil then
    function C_Calendar.GetNumGuildEvents()
        return 0
    end
end
if rawget(C_Calendar, "GetNumInvites") == nil then
    function C_Calendar.GetNumInvites()
        return 0
    end
end
if rawget(C_Calendar, "GetNumPendingInvites") == nil then
    function C_Calendar.GetNumPendingInvites()
        return 0
    end
end
if rawget(C_Calendar, "AreNamesReady") == nil then
    function C_Calendar.AreNamesReady()
        return true
    end
end
if rawget(C_Calendar, "IsActionPending") == nil then
    function C_Calendar.IsActionPending()
        return false
    end
end
if rawget(C_Calendar, "CanAddEvent") == nil then
    function C_Calendar.CanAddEvent()
        return false
    end
end
if rawget(C_Calendar, "CanSendInvite") == nil then
    function C_Calendar.CanSendInvite()
        return false
    end
end
if rawget(C_Calendar, "OpenCalendar") == nil then
    function C_Calendar.OpenCalendar()
    end
end
if rawget(C_Calendar, "CloseEvent") == nil then
    function C_Calendar.CloseEvent()
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(CALENDAR_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_empty_calendar_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        let month: (i32, i32) = env
            .eval("local m = C_Calendar.GetMonthInfo(0); return m.month, m.year")
            .unwrap();
        let expected =
            if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
                (4, 2026)
            } else {
                (1, 2024)
            };
        assert_eq!(month, expected);

        let result: String = env
            .eval(
                r#"
                local filter = C_Calendar.GetDefaultGuildFilter()
                if filter.minLevel ~= 1 then return "filter-min" end
                if filter.maxLevel ~= GetMaxLevelForLatestExpansion() then return "filter-max" end
                if filter.rank ~= 1 then return "filter-rank" end
                if #C_Calendar.EventGetTypesDisplayOrdered() ~= 0 then return "event-types" end
                if #C_Calendar.GetClubCalendarEvents() ~= 0 then return "club-events" end
                if C_Calendar.GetNumDayEvents(0, 1) ~= 0 then return "day-events" end
                if C_Calendar.GetNumGuildEvents() ~= 0 then return "guild-events" end
                if C_Calendar.GetNumInvites() ~= 0 then return "invites" end
                if C_Calendar.GetNumPendingInvites() ~= 0 then return "pending" end
                if C_Calendar.AreNamesReady() ~= true then return "names-ready" end
                if C_Calendar.IsActionPending() ~= false then return "action-pending" end
                if C_Calendar.CanAddEvent() ~= false then return "add-event" end
                if C_Calendar.CanSendInvite() ~= false then return "send-invite" end
                C_Calendar.OpenCalendar()
                C_Calendar.CloseEvent()
                return "ok"
                "#,
            )
            .expect("calendar defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_calendar_functions() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            function C_Calendar.GetNumInvites()
                return 4
            end
            function C_Calendar.AreNamesReady()
                return false
            end
            "#,
        )
        .expect("fixture should install existing functions");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: String = env
            .eval(
                "return C_Calendar.GetNumInvites() .. ':' .. tostring(C_Calendar.AreNamesReady())",
            )
            .expect("existing calendar functions should remain callable");

        assert_eq!(result, "4:false");
    }
}
