//! Temporary `C_DateAndTime` deterministic calendar/server-time defaults.
//!
//! The simulator does not model server calendar state yet. These fixed table
//! shapes keep Blizzard callers stable until a real date/time model exists.

const DATE_AND_TIME_DEFAULTS_LUA: &str = r#"
C_DateAndTime = C_DateAndTime or __wow_namespace()

local MINUTES_PER_DAY = 24 * 60

local function calendarTimeFromParts(monthDay, hour, minute)
    local current = __wow_current_calendar_time()
    return {
        year = current.year,
        month = current.month,
        monthDay = monthDay,
        weekday = current.weekday,
        hour = hour,
        minute = minute,
    }
end

local function calendarTimeFromOffsets(dayOffset, minuteOffset)
    local current = __wow_current_calendar_time()
    local totalMinutes = current.hour * 60 + current.minute + minuteOffset
    local dayDelta = math.floor(totalMinutes / MINUTES_PER_DAY)
    local minuteOfDay = totalMinutes - dayDelta * MINUTES_PER_DAY
    return calendarTimeFromParts(
        current.monthDay + dayOffset + dayDelta,
        math.floor(minuteOfDay / 60),
        minuteOfDay % 60
    )
end

local function numberField(tableValue, key, fallback)
    if type(tableValue) == "table" and type(tableValue[key]) == "number" then
        return tableValue[key]
    end

    return fallback
end

local function calendarTimeArg(tableValue)
    local current = __wow_current_calendar_time()
    return {
        monthDay = numberField(tableValue, "monthDay", current.monthDay),
        hour = numberField(tableValue, "hour", current.hour),
        minute = numberField(tableValue, "minute", current.minute),
    }
end

if rawget(C_DateAndTime, "GetServerTimeLocal") == nil then
    function C_DateAndTime.GetServerTimeLocal()
        return 0
    end
end

if rawget(C_DateAndTime, "AdjustTimeByDays") == nil then
    function C_DateAndTime.AdjustTimeByDays(calendarTime, deltaDays)
        local time = calendarTimeArg(calendarTime)
        return calendarTimeFromParts(
            time.monthDay + (tonumber(deltaDays) or 0),
            time.hour,
            time.minute
        )
    end
end

if rawget(C_DateAndTime, "AdjustTimeByMinutes") == nil then
    function C_DateAndTime.AdjustTimeByMinutes(calendarTime, deltaMinutes)
        local time = calendarTimeArg(calendarTime)
        local baseMinutes = time.hour * 60 + time.minute + (tonumber(deltaMinutes) or 0)
        local dayDelta = math.floor(baseMinutes / MINUTES_PER_DAY)
        local minuteOfDay = baseMinutes - dayDelta * MINUTES_PER_DAY
        return calendarTimeFromParts(
            time.monthDay + dayDelta,
            math.floor(minuteOfDay / 60),
            minuteOfDay % 60
        )
    end
end

if rawget(C_DateAndTime, "GetCalendarTimeFromEpoch") == nil then
    function C_DateAndTime.GetCalendarTimeFromEpoch(epoch)
        local seconds = tonumber(epoch) or 0
        if seconds > 1000000000000 then
            seconds = seconds / 1000000
        end

        local totalMinutes = math.floor(seconds / 60)
        local rawDayOffset = math.floor(totalMinutes / MINUTES_PER_DAY)
        local dayOffset = rawDayOffset % 30
        local minuteOffset = totalMinutes % MINUTES_PER_DAY
        return calendarTimeFromOffsets(dayOffset, minuteOffset)
    end
end

if rawget(C_DateAndTime, "GetWeeklyResetStartTime") == nil then
    function C_DateAndTime.GetWeeklyResetStartTime()
        return 0
    end
end

if rawget(C_DateAndTime, "GetSecondsUntilDailyReset") == nil then
    function C_DateAndTime.GetSecondsUntilDailyReset()
        return 86400
    end
end

if rawget(C_DateAndTime, "GetSecondsUntilWeeklyReset") == nil then
    function C_DateAndTime.GetSecondsUntilWeeklyReset()
        return 604800
    end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(DATE_AND_TIME_DEFAULTS_LUA)?;
    #[cfg(feature = "client-wrath")]
    lua.exec(
        r#"
        -- Missing system: server calendar clock and pending-invitation model.
        -- Project the existing temporary namespace sources; do not seed UI fields.
        -- Retire with those defaults when supplied calendar state is implemented.
        function CalendarGetDate()
            local date = __wow_current_calendar_time()
            return date.weekday, date.month, date.monthDay, date.year
        end
        function CalendarGetNumPendingInvites()
            return C_Calendar.GetNumPendingInvites()
        end
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_deterministic_calendar_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, i32, i32, i32, i32, i32, i32) = env
            .eval(
                r#"
                local current = __wow_current_calendar_time()
                local serverTime = C_DateAndTime.GetServerTimeLocal()
                local weeklyStart = C_DateAndTime.GetWeeklyResetStartTime()
                return current.year,
                       current.month,
                       current.monthDay,
                       current.weekday,
                       current.hour,
                       current.minute,
                       serverTime,
                       weeklyStart
                "#,
            )
            .expect("date/time defaults should be callable");

        assert_eq!(result, (2026, 4, 14, 3, 12, 0, 0, 0));
    }

    #[test]
    fn adjusts_calendar_time_across_day_boundaries() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: (i32, i32, i32, i32, i32, i32, i32) = env
            .eval(
                r#"
                local base = __wow_current_calendar_time()
                local previous = C_DateAndTime.AdjustTimeByMinutes(base, -13 * 60)
                local next = C_DateAndTime.AdjustTimeByMinutes(base, 13 * 60)
                local tomorrow = C_DateAndTime.AdjustTimeByDays(base, 1)
                return previous.monthDay,
                       previous.hour,
                       previous.minute,
                       next.monthDay,
                       next.hour,
                       next.minute,
                       tomorrow.monthDay
                "#,
            )
            .expect("date/time adjustment defaults should be callable");

        assert_eq!(result, (13, 23, 0, 15, 1, 0, 15));
    }

    #[test]
    fn preserves_existing_date_and_time_provider() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_DateAndTime = C_DateAndTime or __wow_namespace()

            function C_DateAndTime.GetServerTimeLocal()
                return 99
            end

            function C_DateAndTime.GetSecondsUntilDailyReset()
                return 12
            end
            "#,
        )
        .expect("fixture should install existing date/time provider");

        super::apply_bootstrap(&mut env.rilua_mut()).expect("workaround should apply");

        let result: (i32, i32) = env
            .eval(
                r#"
                return C_DateAndTime.GetServerTimeLocal(),
                       C_DateAndTime.GetSecondsUntilDailyReset()
                "#,
            )
            .expect("existing date/time provider should remain callable");

        assert_eq!(result, (99, 12));
    }
}
