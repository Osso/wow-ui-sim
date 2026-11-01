#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn calendar_toc() -> PathBuf {
    wow_ui_sim::loader::find_toc_file(&blizzard_ui_dir().join("Blizzard_Calendar"))
        .expect("Blizzard_Calendar TOC must resolve")
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

prefork_full_ui_case! {
fn blizzard_calendar_loads_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &calendar_toc())
        .expect("Blizzard_Calendar should load via Rust loader");

    let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        load_errors.is_empty(),
        "Blizzard_Calendar emitted Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let frames_present: bool = env
        .eval(
            "return CalendarFrame ~= nil \
             and CalendarTodayFrame ~= nil \
             and CalendarViewHolidayFrame ~= nil \
             and CalendarViewRaidFrame ~= nil \
             and CalendarViewEventFrame ~= nil \
             and CalendarCreateEventFrame ~= nil \
             and CalendarMassInviteFrame ~= nil \
             and CalendarEventPickerFrame ~= nil \
             and CalendarTexturePickerFrame ~= nil",
        )
        .expect("frame query should succeed");
    assert!(
        frames_present,
        "Calendar frames should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn calendar_show_projects_supplied_civil_month(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CivilDateTime, DisplayedMonth};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.exec("calendarUpdateCount = 0; calendarObserver = CreateFrame('Frame'); calendarObserver:RegisterEvent('CALENDAR_UPDATE_EVENT_LIST'); calendarObserver:SetScript('OnEvent', function() calendarUpdateCount = calendarUpdateCount + 1 end)").unwrap();
    for (year, month, day) in [(2026, 4, 14), (2028, 2, 29)] {
        {
            let mut state = env.state().borrow_mut();
            state.calendar_now = CivilDateTime { year, month, day, hour: 12, minute: 0 };
            state.calendar_displayed_month = DisplayedMonth { year: 2025, month: 1 };
        }
        env.exec("calendarUpdateCount = 0; CalendarFrame_OnShow(CalendarFrame)").unwrap();
        let (shown_year, shown_month, events): (i32, i32, i32) = env.eval("local m = C_Calendar.GetMonthInfo(); return m.year, m.month, calendarUpdateCount").unwrap();
        assert_eq!((shown_year, shown_month), (year, i32::from(month)));
        assert!(events > 0, "calendar update event should fire");
    }
}
}

prefork_full_ui_case! {
fn calendar_month_and_club_inputs_follow_supplied_state(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CivilDateTime;
    env.state().borrow_mut().calendar_now = CivilDateTime { year: 2026, month: 4, day: 14, hour: 12, minute: 0 };
    env.exec(r#"
        assert(select('#', C_Calendar.SetAbsMonth(13, 2000)) == 0)
        local m = C_Calendar.GetMonthInfo(); assert(m.month == 1 and m.year == 2021 and m.numDays == 31)
        C_Calendar.SetAbsMonth(0, 2035)
        m = C_Calendar.GetMonthInfo(); assert(m.month == 2 and m.year == 2030 and m.numDays == 28)
        local ok, err = pcall(C_Calendar.SetAbsMonth, nil, 2026)
        assert(not ok and err:find('Usage: C_Calendar.SetAbsMonth', 1, true))
        assert(select('#', C_Calendar.SetNextClubId(0)) == 0)
        assert(C_Calendar.GetNextClubId() == 0 and C_Calendar.GetNextClubId() == 0)
        C_Calendar.SetNextClubId(43.9); assert(C_Calendar.GetNextClubId() == 43)
        C_Calendar.SetNextClubId('bad'); assert(C_Calendar.GetNextClubId() == 0)
        assert(not pcall(C_Calendar.SetNextClubId, -1))
        assert(not pcall(C_Calendar.SetNextClubId, 0/0))
        assert(not pcall(C_Calendar.SetNextClubId, 9007199254740992))
        assert(not pcall(C_Calendar.SetNextClubId, '0x0000000000000001'))
        C_Calendar.SetNextClubId('0x0020000000000000')
    "#).unwrap();
    assert_eq!(env.state().borrow().calendar_next_club_id, Some(9_007_199_254_740_992));
    env.exec("C_Calendar.SetNextClubId(); assert(C_Calendar.GetNextClubId() == nil)").unwrap();
}
}

prefork_full_ui_case! {
fn calendar_hide_clears_supplied_club_selection(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_next_club_id = Some(0);
    env.exec("CalendarFrame_OnHide(CalendarFrame)").unwrap();
    let cleared: bool = env.eval("return C_Calendar.GetNextClubId() == nil").unwrap();
    assert!(cleared, "the original OnHide consumer clears the selected club");
}
}

prefork_full_ui_case! {
fn calendar_show_and_hide_run_without_errors(env: &WowLuaEnv) {

    load_addon(&env.loader_env(), &calendar_toc()).expect("Blizzard_Calendar should load");

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("CalendarFrame:Show(); CalendarFrame:Hide()")
        .expect("CalendarFrame Show/Hide should run");

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        errors.is_empty(),
        "CalendarFrame Show/Hide emitted Lua errors:\n  {}",
        errors.join("\n  ")
    );
}
}
