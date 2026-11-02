use wow_ui_sim::c_api::c_calendar::day_event_record::{CalendarDayEventRecord, CalendarTimeRecord};
use wow_ui_sim::lua_api::WowLuaEnv;

pub const KEY: (i32, u32) = (0, 15);

fn start_time() -> CalendarTimeRecord {
    CalendarTimeRecord {
        month_day: 15,
        month: 10,
        weekday: 5,
        year: 2026.0,
        hour: 18.0,
        minute: 30.0,
    }
}

fn end_time() -> CalendarTimeRecord {
    CalendarTimeRecord {
        month_day: 16,
        month: 10,
        weekday: 6,
        year: 2026.0,
        hour: 1.0,
        minute: 45.0,
    }
}

pub fn record() -> CalendarDayEventRecord {
    CalendarDayEventRecord {
        event_id: "9007199254740997".into(),
        category_flags: 0,
        invite_role_markers: 0,
        primary_identity: String::new(),
        alternate_identity: None,
        holiday: None,
        raid_id: 0,
        raid_name: None,
        club_id: "9007199254740995".into(),
        title: "Host raid".into(),
        is_custom_title: true,
        start_time: start_time(),
        end_time: end_time(),
        calendar_type: "GUILD_EVENT".into(),
        sequence_type: "START".into(),
        event_type: 1,
        icon_texture: Some(136116),
        mod_status: "CREATOR".into(),
        has_pending_invite: false,
        invite_status: 4,
        invited_by: "Host-Realm".into(),
        difficulty: 7.5,
        invite_type: 1,
        sequence_index: 2,
        num_sequence_days: 2.5,
        difficulty_name: "Host difficulty".into(),
        dont_display_banner: false,
        dont_display_end: true,
        is_locked: false,
    }
}

pub fn supply(env: &WowLuaEnv, key: (i32, u32), rows: Vec<CalendarDayEventRecord>) {
    env.state()
        .borrow_mut()
        .calendar_day_events
        .insert(key, rows);
}

pub fn erase(env: &WowLuaEnv) {
    env.state().borrow_mut().calendar_day_events.remove(&KEY);
}

pub const ASSERTIONS: &str = r#"
    assert(type(C_Calendar) == 'table', 'Calendar namespace')
    assert(type(C_Calendar.GetDayEvent) == 'function', 'GetDayEvent stub/function guard')
    assert(type(C_Calendar.GetNumDayEvents) == 'function', 'GetNumDayEvents stub/function guard')

    function CalendarDayExpected(title, icon)
        return {
            eventID='9007199254740997', clubID='9007199254740995', title=title,
            isCustomTitle=true,
            startTime={monthDay=15, month=10, weekday=5, year=2026, hour=18, minute=30},
            endTime={monthDay=16, month=10, weekday=6, year=2026, hour=1, minute=45},
            calendarType='GUILD_EVENT', sequenceType='START', eventType=1,
            iconTexture=icon, modStatus='CREATOR', inviteStatus=4,
            invitedBy='Host-Realm', difficulty=7.5, inviteType=1,
            sequenceIndex=2, numSequenceDays=2.5, difficultyName='Host difficulty',
            dontDisplayBanner=false, dontDisplayEnd=true, isLocked=false,
        }
    end

    function CalendarDayEqual(actual, expected)
        assert(type(actual) == 'table', 'MissingSuppliedRecord: expected owned DTO')
        local count, expectedCount = 0, 0
        for key, value in pairs(expected) do
            expectedCount = expectedCount + 1
            if type(value) == 'table' then
                CalendarDayEqual(actual[key], value)
            else
                assert(type(actual[key]) == type(value), 'field type: ' .. key)
                assert(actual[key] == value, 'field value: ' .. key)
            end
        end
        for key in pairs(actual) do
            assert(expected[key] ~= nil, 'unexpected field: ' .. tostring(key))
            count = count + 1
        end
        assert(count == expectedCount, 'exact field inventory')
    end

    function CalendarDayOne(expected, ...)
        assert(type((...)) == 'table', 'MissingSuppliedRecord: expected owned DTO')
        assert(select('#', ...) == 1, 'one supplied DTO')
        CalendarDayEqual((...), expected)
    end

    function CalendarDayMissing(...)
        -- Explicit simulator model policy; MayReturnNothing does not prove native arity.
        assert(select('#', ...) == 1 and (...) == nil, 'model miss: one nil')
    end

    function CalendarDayCount(expected, ...)
        assert(select('#', ...) == 1, 'one count')
        assert(type((...)) == 'number' and (...) == expected, 'exact day count')
    end
"#;

pub fn assert_lua(env: &WowLuaEnv, body: &str) {
    env.exec(&format!("{ASSERTIONS}\n{body}"))
        .expect("public Calendar day behavior");
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "Calendar day Lua errors: {errors:?}");
}

pub fn supplied_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("bare Calendar environment");
    supply(&env, KEY, vec![record()]);
    // Always before consumer loads, security setup, or absence/type-only assertions.
    assert_lua(
        &env,
        "CalendarDayOne(CalendarDayExpected('Host raid', 136116), C_Calendar.GetDayEvent(0,15,1))",
    );
    env
}
