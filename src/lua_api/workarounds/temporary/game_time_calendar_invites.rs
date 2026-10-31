//! Temporary GameTime calendar defaults.
//!
//! Calendar invite state is not modeled yet, but Blizzard
//! startup expects these fields/globals before real calendar data has loaded.

use crate::lua_api::WowLuaEnv;

const GAME_TIME_BOOTSTRAP_LUA: &str = r#"
local function __wow_normalize_time_table(dateTable)
    if type(dateTable) ~= "table" then
        return dateTable
    end

    local normalized = {}
    for key, value in pairs(dateTable) do
        if key == "sec" or key == "min" or key == "hour" or key == "day" or key == "month" or key == "year" then
            normalized[key] = tonumber(value) or value
        else
            normalized[key] = value
        end
    end
    return normalized
end

if time == nil then
    function time(dateTable)
        if os and type(os.time) == "function" then
            if type(dateTable) == "table" then
                dateTable = {
                    year = tonumber(dateTable.year) or 1970,
                    month = tonumber(dateTable.month) or 1,
                    day = tonumber(dateTable.day or dateTable.mday) or 1,
                    hour = tonumber(dateTable.hour) or 0,
                    min = tonumber(dateTable.min) or 0,
                    sec = tonumber(dateTable.sec) or 0,
                    isdst = dateTable.isdst,
                }
            end
            return os.time(__wow_normalize_time_table(dateTable))
        end
        return math.floor(GetTime())
    end
end

if GetTimePreciseSec == nil then
    function GetTimePreciseSec()
        return GetTime()
    end
end

if GameTime_GetTime == nil then
    function GameTime_GetTime(_useLocalTime)
        return "12:00"
    end
end

if GetQuestResetTime == nil then
    function GetQuestResetTime()
        if C_DateAndTime and type(C_DateAndTime.GetSecondsUntilDailyReset) == "function" then
            return C_DateAndTime.GetSecondsUntilDailyReset()
        end
        return 86400
    end
end
"#;

const GAME_TIME_CALENDAR_INVITES_LUA: &str = r#"
if type(GameTimeFrame) == "table" and GameTimeFrame.pendingCalendarInvites == nil then
    GameTimeFrame.pendingCalendarInvites = 0
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(GAME_TIME_BOOTSTRAP_LUA)?;
    Ok(())
}

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(GAME_TIME_CALENDAR_INVITES_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installs_missing_game_time_clock_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("GameTime_GetTime = nil")
            .expect("game time fixture should clear global");

        {
            let mut lua = env.lua.borrow_mut();
            apply_bootstrap(&mut lua).expect("game time bootstrap should apply");
        }

        let time: String = env
            .eval("return GameTime_GetTime(true)")
            .expect("game time default should be callable");

        assert_eq!(time, "12:00");
    }

    #[test]
    fn installs_missing_game_time_globals() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("time = nil")
            .expect("game time fixture should clear globals");

        {
            let mut lua = env.lua.borrow_mut();
            apply_bootstrap(&mut lua).expect("game time bootstrap should apply");
        }

        let result: String = env
            .eval(
                r#"
                local hour, minute = GetGameTime()
                if hour ~= 12 or minute ~= 0 then return "clock" end
                if GetQuestResetTime() ~= C_DateAndTime.GetSecondsUntilDailyReset() then return "quest_reset" end
                if type(time) ~= "function" then return "time_missing" end
                local value = time({ year = "2024", month = "1", day = "2", hour = "3", min = "4", sec = "5" })
                if type(value) ~= "number" then return "time_value" end
                return "ok"
                "#,
            )
            .expect("game time globals should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_game_time_clock_default() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(r#"GameTime_GetTime = function() return "03:14" end"#)
            .expect("game time fixture should install existing global");

        {
            let mut lua = env.lua.borrow_mut();
            apply_bootstrap(&mut lua).expect("game time bootstrap should apply");
        }

        let time: String = env
            .eval("return GameTime_GetTime(true)")
            .expect("game time default should be callable");

        assert_eq!(time, "03:14");
    }

    #[test]
    fn seeds_missing_pending_calendar_invites() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("GameTimeFrame = {}")
            .expect("game time frame fixture should install");

        patch(&env);

        let pending_invites: i64 = env
            .eval("return GameTimeFrame.pendingCalendarInvites")
            .expect("pending calendar invites should be readable");

        assert_eq!(pending_invites, 0);
    }

    #[test]
    fn preserves_existing_pending_calendar_invites() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec("GameTimeFrame = { pendingCalendarInvites = 3 }")
            .expect("game time frame fixture should install");

        patch(&env);

        let pending_invites: i64 = env
            .eval("return GameTimeFrame.pendingCalendarInvites")
            .expect("pending calendar invites should be readable");

        assert_eq!(pending_invites, 3);
    }
}
