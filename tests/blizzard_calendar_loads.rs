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

fn calendar_test_record(id: &str) -> wow_ui_sim::c_api::c_calendar::day_event_record::CalendarDayEventRecord {
    use wow_ui_sim::c_api::c_calendar::day_event_record::{CalendarDayEventRecord, CalendarTimeRecord};
    let time = CalendarTimeRecord { month_day: 15, month: 4, weekday: 4, year: 2026.0, hour: 12.0, minute: 0.0 };
    CalendarDayEventRecord {
        event_id: id.into(), category_flags: 0, invite_role_markers: 0,
        primary_identity: "Player-1-OTHER".into(), alternate_identity: None,
        holiday: None, raid_id: 0, raid_name: None,
        title: id.into(), is_custom_title: false,
        start_time: time.clone(), end_time: time.clone(), calendar_type: "GUILD_EVENT".into(),
        sequence_type: "".into(), event_type: 0, icon_texture: None, mod_status: "CREATOR".into(),
        has_pending_invite: false, invite_status: 0, invited_by: "".into(), difficulty: 0.0, invite_type: 0,
        sequence_index: 0, num_sequence_days: 1.0, difficulty_name: "".into(),
        dont_display_banner: false, dont_display_end: false, club_id: "".into(), is_locked: false,
    }
}

prefork_full_ui_case! {
fn calendar_selection_indices_follow_day_event_identity(env: &WowLuaEnv) {
    env.state().borrow_mut().calendar_day_events.insert((0, 15), vec![calendar_test_record("one"), calendar_test_record("two")]);
    env.exec(r#"
        assert(select('#', C_Calendar.GetEventIndex()) == 0)
        assert(select('#', C_Calendar.ContextMenuGetEventIndex()) == 0)
        C_Calendar.OpenEvent(0, 15, 2)
        C_Calendar.ContextMenuSelectEvent(0, 15, 1)
        local a, b = C_Calendar.GetEventIndex(), C_Calendar.ContextMenuGetEventIndex()
        assert(a.offsetMonths == 0 and a.monthDay == 15 and a.eventIndex == 2)
        assert(b.offsetMonths == 0 and b.monthDay == 15 and b.eventIndex == 1)
    "#).unwrap();
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap().swap(0, 1);
    env.exec(r#"
        assert(C_Calendar.GetEventIndex().eventIndex == 1)
        assert(C_Calendar.ContextMenuGetEventIndex().eventIndex == 2)
        C_Calendar.CloseEvent()
        assert(select('#', C_Calendar.GetEventIndex()) == 0)
        assert(C_Calendar.ContextMenuGetEventIndex().eventIndex == 2)
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_pending_invite_uses_first_flagged_day_event(env: &WowLuaEnv) {
    let mut second = calendar_test_record("two");
    second.has_pending_invite = true;
    env.state().borrow_mut().calendar_day_events.insert((0, 15), vec![calendar_test_record("one"), second]);
    env.exec(r#"
        assert(select('#', C_Calendar.GetFirstPendingInvite(0, 15)) == 1)
        assert(C_Calendar.GetFirstPendingInvite(0, 15) == 2)
        assert(C_Calendar.GetFirstPendingInvite(0, 31) == nil)
        assert(C_Calendar.GetFirstPendingInvite(2, 15) == nil)
    "#).unwrap();
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap()[0].has_pending_invite = true;
    env.exec("assert(C_Calendar.GetFirstPendingInvite(0, 15) == 1)").unwrap();
    env.state().borrow_mut().calendar_day_events.clear();
    env.exec("assert(C_Calendar.GetFirstPendingInvite(0, 15) == nil)").unwrap();
}
}

prefork_full_ui_case! {
fn calendar_month_navigation_projects_relative_month(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::DisplayedMonth;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_displayed_month = DisplayedMonth { year: 2026, month: 12 };
    env.exec("CalendarFrame_OffsetMonth(1)").unwrap();
    let shown = env.state().borrow().calendar_displayed_month;
    assert_eq!((shown.year, shown.month), (2027, 1));
    env.exec("CalendarFrame_OffsetMonth(-2)").unwrap();
    let shown = env.state().borrow().calendar_displayed_month;
    assert_eq!((shown.year, shown.month), (2026, 11));
    env.exec("C_Calendar.SetMonth(-10000)").unwrap();
    let shown = env.state().borrow().calendar_displayed_month;
    assert_eq!((shown.year, shown.month), (2004, 11));
    env.exec("C_Calendar.SetMonth(10000)").unwrap();
    let shown = env.state().borrow().calendar_displayed_month;
    assert_eq!(shown.year, 2030);
}
}

prefork_full_ui_case! {
fn calendar_context_menu_reflects_supplied_permissions(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let mut row = calendar_test_record("context");
    row.calendar_type = "PLAYER".into();
    row.invite_role_markers = 0x2;
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.complaints_enabled = true;
        state.calendar_day_events.insert((0, 15), vec![row]);
    }
    env.exec(r#"
        function CalendarContextMenuProbe()
            local root = { buttons = {} }
            function root:SetTag(_) end
            function root:CreateButton(label) self.buttons[label] = true end
            function root:QueueDivider() end
            function root:ClearQueuedDescriptions() end
            function root:HasElements() return next(self.buttons) ~= nil end
            function root:AddMenuAcquiredCallback(_) end
            function root:AddMenuReleasedCallback(_) end
            GenerateDayContextMenu(nil, root, 2, {day=15, monthOffset=0}, {eventIndex=1})
            return root.buttons[CALENDAR_COPY_EVENT] == true,
                   root.buttons[CALENDAR_DELETE_EVENT] == true,
                   root.buttons[REPORT_CALENDAR] == true
        end
    "#).unwrap();
    let allowed: (bool, bool, bool) = env.eval("return CalendarContextMenuProbe()").unwrap();
    assert_eq!(allowed, (true, true, true));
    {
        let mut state = env.state().borrow_mut();
        let row = &mut state.calendar_day_events.get_mut(&(0, 15)).unwrap()[0];
        row.invite_role_markers = 0;
        row.category_flags = 0x8;
    }
    let rejected: (bool, bool, bool) = env.eval("return CalendarContextMenuProbe()").unwrap();
    assert_eq!(rejected, (false, false, false));
    {
        use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
        let mut state = env.state().borrow_mut();
        state.world.guild_id = 7;
        state.calendar_context_policy.guild_club_id = Some(42);
        state.calendar_context_policy.guild_rank_rights = 0x100000;
        state.calendar_context_policy.edit_subsystem_available = true;
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            category_flags: 0x800, associated_id: Some(42), ..Default::default()
        });
        let row = &mut state.calendar_day_events.get_mut(&(0, 15)).unwrap()[0];
        row.category_flags = 0x800;
    }
    let guild: (bool, bool, bool) = env.eval("return CalendarContextMenuProbe()").unwrap();
    assert_eq!(guild, (true, false, false));
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap()[0].invite_role_markers = 0x4;
    let creator: (bool, bool, bool) = env.eval("return CalendarContextMenuProbe()").unwrap();
    assert_eq!(creator, (true, true, false));
    {
        use wow_ui_sim::lua_api::state::SocialFriend;
        let mut state = env.state().borrow_mut();
        let row = &mut state.calendar_day_events.get_mut(&(0, 15)).unwrap()[0];
        row.category_flags = 0;
        row.primary_identity = "Player-1-FRIEND".into();
        state.social_friends = vec![SocialFriend {
            name: "Friend".into(), level: 80, area: String::new(), class_name: String::new(),
            note: String::new(), is_online: true, guid: "Player-1-FRIEND".into(),
        }];
        state.cvars.register("legacyFriendSystemEnabledClient", Some("0"));
        assert!(state.cvars.set("legacyFriendSystemEnabledClient", "2"));
    }
    let friend: (bool, bool, bool) = env.eval("return CalendarContextMenuProbe()").unwrap();
    assert_eq!(friend, (true, true, false));
}
}

prefork_full_ui_case! {
fn calendar_holiday_view_uses_supplied_day_event(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::day_event_record::CalendarHolidayRecord;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let mut row = calendar_test_record("event");
    row.calendar_type = "HOLIDAY".into();
    row.holiday = Some(CalendarHolidayRecord {
        name: Some("Harvest festival".into()),
        description: Some("Gather the harvest".into()), texture: 77, multi_day: false,
    });
    env.state().borrow_mut().calendar_day_events.insert((0, 15), vec![row]);
    env.exec(r#"
        C_Calendar.OpenEvent(0, 15, 1)
        CalendarViewHolidayFrame_Update()
        assert(CalendarViewHolidayFrame.ScrollingFont:GetFontString():GetText() == 'Gather the harvest')
    "#).unwrap();
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap()[0].holiday = None;
    env.exec("assert(select('#', C_Calendar.GetHolidayInfo(0, 15, 1)) == 0)").unwrap();
}
}

prefork_full_ui_case! {
fn calendar_raid_view_uses_supplied_day_event(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let mut row = calendar_test_record("event");
    row.calendar_type = "RAID_LOCKOUT".into();
    row.raid_id = 42;
    row.raid_name = Some("Sunken Vault".into());
    row.difficulty_name = "Heroic".into();
    env.state().borrow_mut().calendar_day_events.insert((0, 15), vec![row]);
    env.exec(r#"
        C_Calendar.OpenEvent(0, 15, 1)
        CalendarViewRaidFrame_Update()
        assert(CalendarViewRaidFrame.ScrollingFont:GetFontString():GetText():find('Sunken Vault', 1, true))
    "#).unwrap();
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap()[0].calendar_type = "PLAYER".into();
    env.exec(r#"
        local raid = C_Calendar.GetRaidInfo(0, 15, 1)
        assert(raid and raid.raidID == 42 and raid.name == 'Sunken Vault')
        CalendarViewRaidFrame.ScrollingFont:SetText('unchanged')
        CalendarViewRaidFrame_Update()
        assert(CalendarViewRaidFrame.ScrollingFont:GetFontString():GetText() == 'unchanged')
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_guild_event_navigation_rematches_current_day_order(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarSelection;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.exec("CalendarFrame:Show()").unwrap();
    {
        let mut state = env.state().borrow_mut();
        let mut rows: Vec<_> = (1..=6).map(|i| calendar_test_record(&format!("event-{i}"))).collect();
        for row in &mut rows { row.sequence_type = "ONGOING".into(); }
        state.calendar_day_events.insert((0, 15), rows);
        state.calendar_guild_selections = vec![CalendarSelection { event_id: "event-5".into(), month: 2026 * 12 + 3, month_day: 15 }];
    }
    env.exec(r#"
        CalendarFrame_OpenToGuildEventIndex(1)
        local info = C_Calendar.GetEventIndex()
        assert(info and info.monthDay == 15 and info.eventIndex == 5)
    "#).unwrap();
    env.state().borrow_mut().calendar_day_events.get_mut(&(0, 15)).unwrap().swap(4, 5);
    env.exec(r#"
        CalendarFrame_OpenToGuildEventIndex(1)
        local info = C_Calendar.GetEventIndex()
        assert(info and info.monthDay == 15 and info.eventIndex == 6)
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_context_clipboard_controls_paste_menu(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CivilDateTime, DisplayedMonth};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_now = CivilDateTime { year: 2024, month: 1, day: 1, hour: 12, minute: 0 };
        state.calendar_displayed_month = DisplayedMonth { year: 2024, month: 1 };
        state.calendar_clipboard = Some(calendar_test_record("copied"));
    }
    env.exec(r#"
        function CalendarPasteMenuProbe()
            local root = { buttons = {} }
            function root:SetTag(_) end
            function root:CreateButton(label) self.buttons[label] = true end
            function root:QueueDivider() end
            function root:HasElements() return next(self.buttons) ~= nil end
            function root:AddMenuAcquiredCallback(_) end
            function root:AddMenuReleasedCallback(_) end
            GenerateDayContextMenu(nil, root, 0, {day=1, monthOffset=0}, nil)
            return root.buttons[CALENDAR_PASTE_EVENT] == true
        end
        assert(CalendarPasteMenuProbe())
    "#).unwrap();
    env.state().borrow_mut().calendar_clipboard = None;
    env.exec("assert(not CalendarPasteMenuProbe())").unwrap();
}
}

prefork_full_ui_case! {
fn calendar_context_type_selects_delete_confirmation(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let mut row = calendar_test_record("context");
    row.calendar_type = "GUILD_EVENT".into();
    env.state().borrow_mut().calendar_day_events.insert((0, 15), vec![row]);
    env.exec(r#"
        assert(select('#', C_Calendar.ContextMenuEventGetCalendarType()) == 1)
        assert(C_Calendar.ContextMenuEventGetCalendarType() == nil)
        C_Calendar.ContextMenuSelectEvent(0, 15, 1)
        CalendarDayContextMenu_DeleteEvent()
        local popup = StaticPopup_FindVisible('CALENDAR_DELETE_EVENT')
        assert(popup and popup:IsShown())
        assert(popup:GetTextFontString():GetText():find(CALENDAR_DELETE_GUILD_EVENT_CONFIRM, 1, true))
    "#).unwrap();
}
}
