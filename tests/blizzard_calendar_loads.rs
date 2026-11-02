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
        CalendarDayContextMenu_DeleteEvent()
        StaticPopup_Hide('CALENDAR_DELETE_EVENT')
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

prefork_full_ui_case! {
fn calendar_edit_button_reflects_supplied_dirty_state(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess::default());
    env.exec(r#"
        CalendarCreateEventFrame.mode = 'edit'
        CalendarCreateEventCreateButton_Update()
        assert(not CalendarCreateEventCreateButton:IsEnabled())
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().dirty = true;
    env.exec(r#"
        CalendarCreateEventCreateButton_Update()
        assert(CalendarCreateEventCreateButton:IsEnabled())
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = None;
    env.exec(r#"
        CalendarCreateEventCreateButton_Update()
        assert(not CalendarCreateEventCreateButton:IsEnabled())
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_invite_button_projects_supplied_invitee(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let invite = CalendarInviteRecord { invite_id: "first".into(), name: Some("Alma".into()), invite_status: Some(0), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { invites: vec![invite], ..Default::default() });
    env.exec(r#"
        CalendarCreateEventFrame.mode = 'create'
        calendarInviteProbe = CreateFrame('Button', 'CalendarInviteProbe', UIParent, 'CalendarCreateEventInviteListButtonTemplate')
        CalendarCreateEventInviteList_InitButton(calendarInviteProbe, {index = 1})
        assert(calendarInviteProbe.Name:GetText() == 'Alma')
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().invites[0].name = Some("Bora".into());
    env.exec(r#"
        CalendarCreateEventInviteList_InitButton(calendarInviteProbe, {index = 1})
        assert(calendarInviteProbe.Name:GetText() == 'Bora')
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_invite_button_tracks_selected_identity(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let invite = CalendarInviteRecord { invite_id: "first".into(), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { invites: vec![invite], selected_invite_id: Some("first".into()), ..Default::default() });
    env.exec(r#"
        calendarSelectedInviteProbe = CreateFrame('Button', 'CalendarSelectedInviteProbe', UIParent, 'CalendarCreateEventInviteListButtonTemplate')
        local info = {name='Alma', className='', inviteStatus=0, modStatus=''}
        CalendarEventInviteList_InitButtonShared(calendarSelectedInviteProbe, 1, info)
        assert(CalendarCreateEventFrame.selectedInvite == calendarSelectedInviteProbe)
        CalendarCreateEventFrame_SetSelectedInvite(nil)
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().selected_invite_id = None;
    env.exec(r#"
        local info = {name='Alma', className='', inviteStatus=0, modStatus=''}
        CalendarEventInviteList_InitButtonShared(calendarSelectedInviteProbe, 1, info)
        assert(CalendarCreateEventFrame.selectedInvite == nil)
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_invite_sort_controls_reflect_supplied_criterion(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { sort_criterion: "class".into(), ..Default::default() });
    env.exec(r#"
        CalendarEventInviteList_UpdateSortButtons(CalendarCreateEventInviteList)
        assert(CalendarCreateEventInviteListClassSortButtonDirection:IsShown())
        assert(not CalendarCreateEventInviteListStatusSortButtonDirection:IsShown())
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().sort_criterion = "status".into();
    env.exec(r#"
        CalendarEventInviteList_UpdateSortButtons(CalendarCreateEventInviteList)
        assert(not CalendarCreateEventInviteListClassSortButtonDirection:IsShown())
        assert(CalendarCreateEventInviteListStatusSortButtonDirection:IsShown())
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_checkbox_updates_auto_approve(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { local_edit_credential: Some(1), ..Default::default() });
    env.exec("CalendarCreateEventAutoApproveCheck:SetChecked(true); CalendarCreateEvent_SetAutoApprove()").unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert!(event.auto_approve && event.dirty);
    }
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().dirty = false;
    env.exec("CalendarCreateEventAutoApproveCheck:SetChecked(false); CalendarCreateEvent_SetAutoApprove()").unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert!(!event.auto_approve && event.dirty);
}
}

prefork_full_ui_case! {
fn calendar_checkbox_updates_locked(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { local_edit_credential: Some(1), ..Default::default() });
    env.exec("CalendarCreateEventLockEventCheck:SetChecked(true); CalendarCreateEvent_SetLockEvent()").unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert!(event.locked && event.dirty);
    }
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().dirty = false;
    env.exec("CalendarCreateEventLockEventCheck:SetChecked(false); CalendarCreateEvent_SetLockEvent()").unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert!(!event.locked && event.dirty);
}
}

prefork_full_ui_case! {
fn calendar_title_edit_updates_supplied_event(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            local_edit_credential: Some(123),
            ..Default::default()
        });
    }
    env.exec(r#"
        CalendarCreateEventFrame.mode = 'edit'
        CalendarCreateEventTitleEdit:SetText('Raid|night\\nignored')
        CalendarCreateEventTitleEdit_OnTextChanged(CalendarCreateEventTitleEdit, true)
    "#).unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.title, b"Raidnight");
        assert!(event.dirty);
    }
    {
        let mut state = env.state().borrow_mut();
        let event = state.calendar_context_policy.editable.as_mut().unwrap();
        event.dirty = false;
        event.local_edit_credential = None;
    }
    env.exec(r#"
        CalendarCreateEventTitleEdit:SetText('Denied')
        CalendarCreateEventTitleEdit_OnTextChanged(CalendarCreateEventTitleEdit, true)
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.title, b"Raidnight");
    assert!(!event.dirty);
}
}

prefork_full_ui_case! {
fn calendar_description_edit_updates_supplied_event(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess {
        local_edit_credential: Some(123),
        ..Default::default()
    });
    env.exec(r#"
        local box = CalendarCreateEventDescriptionContainer.ScrollingEditBox
        box:SetText('First|line\nSecond')
        box:TriggerEvent('OnTextChanged', box, true)
    "#).unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.description, b"Firstline\nSecond");
        assert!(event.dirty);
    }
    {
        let mut state = env.state().borrow_mut();
        let event = state.calendar_context_policy.editable.as_mut().unwrap();
        event.dirty = false;
        event.local_edit_credential = None;
    }
    env.exec(r#"
        local box = CalendarCreateEventDescriptionContainer.ScrollingEditBox
        box:SetText('Denied')
        box:TriggerEvent('OnTextChanged', box, true)
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.description, b"Firstline\nSecond");
    assert!(!event.dirty);
}
}

prefork_full_ui_case! {
fn calendar_create_events_report_native_notifications(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::club_model::Club;
    env.exec(r#"
        calendarCreateEvents = {}
        local observer = CreateFrame('Frame')
        for _, name in ipairs({'CALENDAR_UPDATE_INVITE_LIST', 'CALENDAR_UPDATE_ERROR', 'CALENDAR_ACTION_PENDING'}) do
            observer:RegisterEvent(name)
        end
        observer:SetScript('OnEvent', function(_, name, ...)
            table.insert(calendarCreateEvents, {name = name, count = select('#', ...), value = ...})
        end)
        assert(select('#', C_Calendar.CreatePlayerEvent()) == 0)
        assert(#calendarCreateEvents == 1)
        assert(calendarCreateEvents[1].name == 'CALENDAR_UPDATE_INVITE_LIST')
        assert(calendarCreateEvents[1].count == 1 and calendarCreateEvents[1].value == true)
        calendarCreateEvents = {}
        assert(select('#', C_Calendar.CreateGuildSignUpEvent()) == 0)
        assert(#calendarCreateEvents == 2)
        assert(calendarCreateEvents[1].name == 'CALENDAR_UPDATE_ERROR' and calendarCreateEvents[1].count == 1 and calendarCreateEvents[1].value == 'ERR_GUILD_PLAYER_NOT_IN_GUILD')
        assert(calendarCreateEvents[2].name == 'CALENDAR_ACTION_PENDING' and calendarCreateEvents[2].count == 1 and calendarCreateEvents[2].value == false)
    "#).unwrap();
    for id in ["123", "456"] {
        {
            let mut state = env.state().borrow_mut();
            state.clubs.clubs.clear();
            state.clubs.clubs.insert(id.into(), Club {
                id: id.into(), name: id.into(), description: String::new(),
                club_type: 2, avatar_id: 0, members: Vec::new(), candidates: Default::default(),
                invitations: Vec::new(), streams: Vec::new(),
            });
        }
        env.exec(r#"
            calendarCreateEvents = {}
            assert(select('#', C_Calendar.CreateGuildSignUpEvent()) == 0)
            assert(C_Calendar.EventGetCalendarType() == 'GUILD_EVENT')
            assert(#calendarCreateEvents == 0)
        "#).unwrap();
        assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().club_id, id.parse::<u64>().unwrap());
        env.exec(r#"
            assert(select('#', C_Calendar.CreateGuildAnnouncementEvent()) == 0)
            assert(C_Calendar.EventGetCalendarType() == 'GUILD_ANNOUNCEMENT')
            assert(C_Calendar.GetNumInvites() == 0)
            assert(select('#', C_Calendar.CreateCommunitySignUpEvent()) == 0)
            assert(C_Calendar.EventGetCalendarType() == 'COMMUNITY_EVENT')
        "#).unwrap();
    }
}
}

prefork_full_ui_case! {
fn calendar_create_events_follow_supplied_local_player(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::club_model::Club;
    env.state().borrow_mut().clubs.clubs.insert("77".into(), Club {
        id: "77".into(), name: "Guild".into(), description: String::new(),
        club_type: 2, avatar_id: 0, members: Vec::new(), candidates: Default::default(),
        invitations: Vec::new(), streams: Vec::new(),
    });
    for (level, invite_type, create) in [(37, 0, "CreatePlayerEvent"), (80, 1, "CreateGuildSignUpEvent"), (80, 1, "CreateCommunitySignUpEvent")] {
        {
            let mut state = env.state().borrow_mut();
            state.player.present = true;
            state.player.level = level;
        }
        env.exec(&format!(r#"
            C_Calendar.{create}()
            assert(C_Calendar.GetNumInvites() == 1)
            local invite = C_Calendar.EventGetInvite(1)
            assert(invite.inviteStatus == 3 and invite.modStatus == 'CREATOR')
            assert(invite.type == {invite_type} and invite.level == {level} and invite.notes == '')
            assert(invite.guid == UnitGUID('player') and invite.name == nil)
        "#)).unwrap();
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.creator_identity, "Player-1-00000001");
        assert_eq!(event.invites[0].invite_id, "0");
    }
    env.state().borrow_mut().player.present = false;
    for create in ["CreatePlayerEvent", "CreateGuildSignUpEvent", "CreateCommunitySignUpEvent", "CreateGuildAnnouncementEvent"] {
        env.exec(&format!("C_Calendar.{create}(); assert(C_Calendar.GetNumInvites() == 0)")).unwrap();
        assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().creator_identity, "");
    }
    env.state().borrow_mut().player.present = true;
    env.exec("C_Calendar.CreateGuildAnnouncementEvent(); assert(C_Calendar.GetNumInvites() == 0)").unwrap();
    for (create, flags, calendar_type) in [
        ("CreatePlayerEvent", 0x1, "PLAYER"),
        ("CreateGuildSignUpEvent", 0xc00, "GUILD_EVENT"),
        ("CreateGuildAnnouncementEvent", 0x40, "GUILD_ANNOUNCEMENT"),
        ("CreateCommunitySignUpEvent", 0x400, "COMMUNITY_EVENT"),
    ] {
        let shown: String = env.eval(&format!("C_Calendar.{create}(); return C_Calendar.EventGetCalendarType()")).unwrap();
        assert_eq!(shown, calendar_type);
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.category_flags, flags);
        assert_eq!(event.event_id, "0");
        assert_eq!((event.event_type, event.texture_id), (0, Some(4_294_967_295)));
        assert!(event.title.is_empty() && event.description.is_empty());
        assert!(!event.locked && !event.auto_approve);
        assert_eq!(event.date, Some((-1, -1, -1)));
        // The model stores time components as unsigned 32-bit words; native -1 is all ones.
        assert_eq!(event.time, Some((u32::MAX, u32::MAX)));
    }
}
}

prefork_full_ui_case! {
fn calendar_create_player_event_replaces_editable_record(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, DisplayedMonth};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            title: b"old event".to_vec(),
            local_edit_credential: Some(123),
            ..Default::default()
        });
        state.calendar_displayed_month = DisplayedMonth { year: 2026, month: 4 };
        state.calendar_day_events.insert((0, 15), vec![calendar_test_record("old-event")]);
    }
    env.exec(r#"
        C_Calendar.OpenEvent(0, 15, 1)
        C_Calendar.ContextMenuSelectEvent(0, 15, 1)
        local day = CreateFrame('Button')
        day:SetID(3)
        day:SetHighlightTexture(day:CreateTexture())
        day.day = 15
        day.monthOffset = 0
        CalendarDayContextMenu_CreateEvent(day)
        assert(CalendarCreateEventFrame:IsShown())
        assert(C_Calendar.GetEventIndex() == nil)
        assert(C_Calendar.ContextMenuGetEventIndex().eventIndex == 1)
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().expect("new editable event");
    assert_ne!(event.title, b"old event");
    assert_eq!(event.date, Some((3, 14, 26)));
}
}

prefork_full_ui_case! {
fn calendar_texture_picker_sets_editable_event_type(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarTextureRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            local_edit_credential: Some(123),
            ..Default::default()
        });
        state.calendar_texture_lists[0] = vec![CalendarTextureRecord { id: 42, icon_texture: 111, ..Default::default() }];
        state.calendar_texture_lists[1] = vec![CalendarTextureRecord { id: 84, icon_texture: 222, ..Default::default() }];
    }
    for (event_type, texture_id) in [(1_u8, 84), (0_u8, 42)] {
        env.exec(&format!("CalendarCreateEvent_SetSelectedIndex(1, {event_type})")).unwrap();
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.event_type, event_type);
        assert_eq!(event.texture_id, Some(texture_id));
    }
    {
        let mut state = env.state().borrow_mut();
        let event = state.calendar_context_policy.editable.as_mut().unwrap();
        event.event_type = 5;
        event.dirty = false;
    }
    env.exec("C_Calendar.EventSetType(1)").unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.event_type, 5);
        assert!(!event.dirty);
    }
    env.exec("C_Calendar.EventSetType(261); C_Calendar.EventSetTextureID(0.5)").unwrap();
    {
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.event_type, 5);
        assert_eq!(event.texture_id, Some(84));
        assert!(event.dirty);
    }
    env.exec("C_Calendar.EventSetType(-255); C_Calendar.EventSetTextureID(200)").unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.event_type, 5);
    assert_eq!(event.texture_id, Some(84));
}
}

prefork_full_ui_case! {
fn calendar_create_event_projects_selected_date(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, DisplayedMonth};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            local_edit_credential: Some(123),
            ..Default::default()
        });
        state.calendar_displayed_month = DisplayedMonth { year: 2026, month: 4 };
    }
    env.exec(r#"
        local day = CreateFrame('Button')
        day:SetID(3)
        day.day = 15
        day.monthOffset = 0
        CalendarCreateEventFrame.dayButton = day
        CalendarCreateEventFrame.mode = 'create'
        CalendarCreateEventFrame_Update()
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.date, Some((3, 14, 26)));
    assert!(event.dirty);
    drop(state);
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable.as_mut().unwrap().dirty = false;
        state.calendar_context_policy.action_pending = true;
    }
    env.exec(r#"
        dateErrors = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('CALENDAR_UPDATE_ERROR')
        observer:RegisterEvent('CALENDAR_ACTION_PENDING')
        observer:SetScript('OnEvent', function(_, name, value) table.insert(dateErrors, {name, value}) end)
        CalendarCreateEventFrame.dayButton.day = 32
        CalendarCreateEventFrame_Update()
        assert(#dateErrors == 2)
        assert(dateErrors[1][1] == 'CALENDAR_UPDATE_ERROR' and dateErrors[1][2] == 'CALENDAR_ERROR_INVALID_DATE')
        assert(dateErrors[2][1] == 'CALENDAR_ACTION_PENDING' and dateErrors[2][2] == false)
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.date, Some((3, 14, 26)));
    assert!(event.dirty);
    assert!(!state.calendar_context_policy.action_pending);
}
}

prefork_full_ui_case! {
fn calendar_event_time_control_updates_supplied_event(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess {
        local_edit_credential: Some(123),
        ..Default::default()
    });
    for (hour, minute) in [(3, 17), (21, 45)] {
        env.exec(&format!("CalendarFrame.militaryTime = true; CalendarCreateEventFrame.selectedHour = {hour}; CalendarCreateEventFrame.selectedMinute = {minute}; CalendarCreateEvent_SetEventTime()" )).unwrap();
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.time, Some((hour, minute)));
        assert!(event.dirty);
    }
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.action_pending = true;
        state.calendar_context_policy.editable.as_mut().unwrap().dirty = false;
    }
    env.exec(r#"
        timeErrors = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('CALENDAR_UPDATE_ERROR')
        observer:RegisterEvent('CALENDAR_ACTION_PENDING')
        observer:SetScript('OnEvent', function(_, name, value) table.insert(timeErrors, {name, value}) end)
        CalendarCreateEventFrame.selectedHour = 24
        CalendarCreateEventFrame.selectedMinute = 12
        CalendarCreateEvent_SetEventTime()
        assert(#timeErrors == 2)
        assert(timeErrors[1][1] == 'CALENDAR_UPDATE_ERROR' and timeErrors[1][2] == 'CALENDAR_ERROR_INVALID_TIME')
        assert(timeErrors[2][1] == 'CALENDAR_ACTION_PENDING' and timeErrors[2][2] == false)
    "#).unwrap();
    let state = env.state().borrow();
    let event = state.calendar_context_policy.editable.as_ref().unwrap();
    assert_eq!(event.time, Some((21, 45)));
    assert!(event.dirty);
    assert!(!state.calendar_context_policy.action_pending);
    drop(state);
    env.exec(r#"
        CalendarCreateEventFrame.selectedHour = '23.9'
        CalendarCreateEventFrame.selectedMinute = '59.9'
        CalendarCreateEvent_SetEventTime()
    "#).unwrap();
    assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().time, Some((23, 59)));
    env.exec(r#"
        CalendarCreateEventFrame.selectedHour = 2147483648
        CalendarCreateEventFrame.selectedMinute = 4294967295
        CalendarCreateEvent_SetEventTime()
    "#).unwrap();
    assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().time, Some((2147483648, 4294967295)));
    env.exec(r#"
        CalendarCreateEventFrame.selectedHour = -1
        assert(not pcall(CalendarCreateEvent_SetEventTime))
    "#).unwrap();
    assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().time, Some((2147483648, 4294967295)));
}
}

prefork_full_ui_case! {
fn calendar_invite_status_menu_uses_supplied_options(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let name: String = env.eval("return UnitName('player')").unwrap();
    let invite = CalendarInviteRecord { invite_id: "one".into(), name: Some(name), invite_status: Some(0), mod_status: Some("CREATOR".into()), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { local_edit_credential: Some(1), invites: vec![invite], ..Default::default() });
    env.exec(r#"
        CalendarCreateEventFrame.mode = 'edit'
        calendarStatusProbe = CreateFrame('Button', 'CalendarStatusProbe', UIParent)
        calendarStatusProbe.inviteIndex = 1
        CalendarCreateEventInviteListButton_OnClick(calendarStatusProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        local options = C_Calendar.EventGetStatusOptions(1)
        local expected = {'ACCEPTED', 'DECLINED', 'CONFIRMED', 'OUT', 'STANDBY', 'TENTATIVE'}
        assert(#options == 6)
        for i, status in ipairs({1, 2, 3, 4, 5, 8}) do
            assert(options[i].status == status and options[i].statusString == 'CALENDAR_STATUS_' .. expected[i])
        end
        Menu.GetManager():CloseMenus()
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().invites[0].invite_status = Some(1);
    env.exec(r#"
        CalendarCreateEventInviteListButton_OnClick(calendarStatusProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        local options = C_Calendar.EventGetStatusOptions(1)
        assert(#options == 5 and options[1].status == 2)
        Menu.GetManager():CloseMenus()
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        let event = state.calendar_context_policy.editable.as_mut().unwrap();
        event.category_flags = 0x400;
        event.invites[0].invite_status = Some(4);
    }
    env.exec(r#"
        CalendarCreateEventInviteListButton_OnClick(calendarStatusProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        local options = C_Calendar.EventGetStatusOptions(1)
        assert(#options == 3 and options[1].status == 3 and options[2].status == 5 and options[3].status == 8)
        Menu.GetManager():CloseMenus()
        assert(select('#', C_Calendar.EventGetStatusOptions(99)) == 1 and #C_Calendar.EventGetStatusOptions(99) == 0)
    "#).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.action_pending = true;
        state.calendar_context_policy.editable.as_mut().unwrap().local_edit_credential = None;
    }
    env.exec(r#"
        wm27events = {}
        wm27observer = CreateFrame('Frame')
        wm27observer:RegisterEvent('CALENDAR_UPDATE_ERROR')
        wm27observer:RegisterEvent('CALENDAR_ACTION_PENDING')
        wm27observer:SetScript('OnEvent', function(_, event, value) table.insert(wm27events, {event, value}) end)
        assert(#C_Calendar.EventGetStatusOptions(99) == 0)
        assert(#wm27events == 2)
        assert(wm27events[1][1] == 'CALENDAR_UPDATE_ERROR' and wm27events[1][2] == 'CALENDAR_ERROR_PERMISSIONS')
        assert(wm27events[2][1] == 'CALENDAR_ACTION_PENDING' and wm27events[2][2] == false)
    "#).unwrap();
    assert!(!env.state().borrow().calendar_context_policy.action_pending);
    env.state().borrow_mut().calendar_context_policy.editable = None;
    env.exec("wm27events = {}; assert(select('#', C_Calendar.EventGetStatusOptions(1)) == 1 and #C_Calendar.EventGetStatusOptions(1) == 0); assert(#wm27events == 0)").unwrap();
}
}

prefork_full_ui_case! {
fn calendar_view_invite_menu_tracks_pending_event_identity(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let invite = CalendarInviteRecord { invite_id: "one".into(), invite_status: Some(0), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { event_id: "event-a".into(), local_edit_credential: Some(1), invites: vec![invite], ..Default::default() });
    env.exec(r#"
        calendarPendingProbe = CreateFrame('Button', 'CalendarPendingProbe', UIParent)
        calendarPendingProbe.inviteIndex = 1
        CalendarViewEventFrame.myInviteIndex = 1
        Menu.GetManager():CloseMenus()
        CalendarViewEventInviteListButton_OnClick(calendarPendingProbe, 'RightButton')
        assert(not Menu.GetManager():GetOpenMenu())
    "#).unwrap();
    env.state().borrow_mut().calendar_pending_invite_event_ids.insert("event-a".into());
    env.exec(r#"
        CalendarViewEventInviteListButton_OnClick(calendarPendingProbe, 'RightButton')
        assert(Menu.GetManager():GetOpenMenu())
        Menu.GetManager():CloseMenus()
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_invite_autocomplete_tracks_editable_event_type(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { category_flags: 0x800, ..Default::default() });
    env.exec(r#"
        local edit = CalendarCreateEventInviteEdit
        edit:GetScript('OnTextChanged')(edit, true)
        assert(edit.autoCompleteParams[1] == AUTOCOMPLETE_LIST.CALENDARGUILDEVENT.include)
        assert(edit.autoCompleteParams[2] == AUTOCOMPLETE_LIST.CALENDARGUILDEVENT.exclude)
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().category_flags = 0x40;
    env.exec(r#"
        local edit = CalendarCreateEventInviteEdit
        edit:GetScript('OnTextChanged')(edit, true)
        assert(edit.autoCompleteParams[1] == AUTOCOMPLETE_LIST.CALENDAREVENT.include)
        assert(edit.autoCompleteParams[2] == AUTOCOMPLETE_LIST.CALENDAREVENT.exclude)
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_invite_hover_uses_supplied_response_time(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::{CalendarEditableAccess, CalendarInviteRecord, day_event_record::CalendarTimeRecord};
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let invite = CalendarInviteRecord { response_time: Some(CalendarTimeRecord { year: 2026.0, month: 4, month_day: 15, weekday: 4, hour: 12.0, minute: 30.0 }), ..Default::default() };
    env.state().borrow_mut().calendar_context_policy.editable = Some(CalendarEditableAccess { invites: vec![invite], ..Default::default() });
    env.exec(r#"
        calendarResponseProbe = CreateFrame('Button', 'CalendarResponseProbe', UIParent)
        calendarResponseProbe.inviteIndex = 1
        GameTooltip:Hide()
        CalendarEventInviteListButton_OnEnter(calendarResponseProbe)
        assert(GameTooltip:IsShown())
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().invites[0].response_time = None;
    env.exec(r#"
        GameTooltip:Hide()
        CalendarEventInviteListButton_OnEnter(calendarResponseProbe)
        assert(not GameTooltip:IsShown())
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_texture_picker_uses_supplied_texture_list(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarTextureRecord;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    env.state().borrow_mut().calendar_texture_lists[0] = vec![CalendarTextureRecord { id: 0, title: "Test Vault".into(), icon_texture: 136116, expansion_level: 0, difficulty_id: 1, map_id: 42, is_lfr: false }];
    env.exec(r#"
        CalendarTexturePickerFrame:Hide()
        CalendarTexturePickerFrame_Show(0)
        assert(CalendarTexturePickerFrame:IsShown())
        assert(CalendarTexturePickerFrame.ScrollBox:GetDataProviderSize() > 0)
        CalendarTexturePickerFrame_Hide()
        CalendarTexturePickerFrame_Show(1)
        assert(not CalendarTexturePickerFrame:IsShown())
    "#).unwrap();
}
}

prefork_full_ui_case! {
fn calendar_community_dropdown_edits_supplied_club_id(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    use wow_ui_sim::c_api::club_model::Club;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess {
            local_edit_credential: Some(123),
            ..Default::default()
        });
        state.clubs.initialized = true;
        for id in ["0x0020000000000000", "0x0020000000000001"] {
            state.clubs.clubs.insert(id.into(), Club {
                id: id.into(), name: id.into(), description: String::new(),
                club_type: 1, avatar_id: 0, members: Vec::new(), candidates: Default::default(),
                invitations: Vec::new(), streams: Vec::new(),
            });
        }
    }
    for (id, expected) in [("0x0020000000000000", 9_007_199_254_740_992_u64), ("0x0020000000000001", 9_007_199_254_740_993_u64)] {
        env.exec(&format!(r#"
            CalendarCreateEventFrame.selectedEventType = 0
            local dropdown = CalendarCreateEventFrame.CommunityDropdown
            dropdown:GenerateMenu()
            for _, entry in dropdown:GetMenuDescription():EnumerateElementDescriptions() do
                if entry:GetData() == '{id}' then entry:Pick({{}}); return end
            end
            error('community entry not found')
        "#)).unwrap();
        let state = env.state().borrow();
        let event = state.calendar_context_policy.editable.as_ref().unwrap();
        assert_eq!(event.club_id, expected);
        assert!(event.dirty);
    }
    env.exec("C_Calendar.EventSetClubId(nil)").unwrap();
    assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().club_id, 0);
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().local_edit_credential = None;
    env.exec("C_Calendar.EventSetClubId('0x0020000000000000')").unwrap();
    assert_eq!(env.state().borrow().calendar_context_policy.editable.as_ref().unwrap().club_id, 0);
}
}

prefork_full_ui_case! {
fn calendar_community_dropdown_tracks_supplied_club_id(env: &WowLuaEnv) {
    use wow_ui_sim::c_api::c_calendar::CalendarEditableAccess;
    use wow_ui_sim::c_api::club_model::Club;
    load_addon(&env.loader_env(), &calendar_toc()).unwrap();
    let club_id = 9_007_199_254_740_992_u64;
    {
        let mut state = env.state().borrow_mut();
        state.calendar_context_policy.editable = Some(CalendarEditableAccess { club_id, ..Default::default() });
        state.clubs.initialized = true;
        state.clubs.clubs.insert("0x0020000000000000".into(), Club {
            id: "0x0020000000000000".into(), name: "Test community".into(), description: String::new(),
            club_type: 1, avatar_id: 0, members: Vec::new(), candidates: Default::default(), invitations: Vec::new(), streams: Vec::new(),
        });
    }
    env.exec(r#"
        CalendarCreateEventFrame.selectedEventType = 0
        function CalendarCommunitySelected()
            local dropdown = CalendarCreateEventFrame.CommunityDropdown
            dropdown:GenerateMenu()
            local root = dropdown:GetMenuDescription()
            for _, entry in root:EnumerateElementDescriptions() do
                if entry:GetData() == '0x0020000000000000' then return entry:IsSelected() end
            end
            error('community entry not found')
        end
        assert(CalendarCommunitySelected())
    "#).unwrap();
    env.state().borrow_mut().calendar_context_policy.editable.as_mut().unwrap().club_id = 0;
    env.exec("assert(not CalendarCommunitySelected())").unwrap();
}
}

