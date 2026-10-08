//! Documented calendar ordering, not native security/invalid-date parity.
use wow_ui_sim::lua_api::WowLuaEnv;

pub(crate) fn assert_calendar_ordering(env: &WowLuaEnv) {
    env.exec(
        r#"
        local compare = rawget(C_DateAndTime, 'CompareCalendarTime')
        assert(type(compare) == 'function', 'raw calendar comparison producer missing')
        local function date(year, month, day, hour, minute, weekday)
            return { year=year, month=month, monthDay=day, hour=hour,
                     minute=minute, weekday=weekday or 1 }
        end
        local early = date(2024, 2, 29, 23, 59, 5)
        local later = date(2024, 3, 1, 0, 0, 6)
        -- Blizzard documentation orders rhs against lhs.
        assert(compare(early, later) == 1)
        assert(compare(later, early) == -1)
        assert(compare(early, date(2024, 2, 29, 23, 59, 7)) == 0)
        local boundaries = {
            {date(2023, 12, 31, 23, 59), date(2024, 1, 1, 0, 0)},
            {date(2024, 6, 30, 23, 59), date(2024, 7, 1, 0, 0)},
            {date(2024, 7, 1, 11, 59), date(2024, 7, 1, 12, 0)},
            {date(2024, 7, 1, 12, 0), date(2024, 7, 1, 12, 1)},
        }
        for _, pair in ipairs(boundaries) do
            assert(compare(pair[1], pair[2]) == 1)
            assert(compare(pair[2], pair[1]) == -1)
        end
        assert(early.year == 2024 and early.month == 2 and early.monthDay == 29)
        assert(early.hour == 23 and early.minute == 59 and early.weekday == 5)
        assert(not pcall(compare, nil, later), 'missing input must fail')
        assert(not pcall(compare, {}, later), 'missing calendar fields must fail')
        "#,
    )
    .expect("compare supplied calendar fields without changing inputs");
}

#[test]
fn patch_8_1_0_calendar_ordering() {
    let env = WowLuaEnv::new().expect("initialize environment");
    assert_calendar_ordering(&env);
}

prefork_full_ui_case! {
fn patch_8_1_0_cached_calendar_ordering(env: &WowLuaEnv) {
    assert_calendar_ordering(env);
}
}
