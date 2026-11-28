#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn load_calendar_addon(env: &WowLuaEnv) {
    let addon_dir = blizzard_ui_dir().join("Blizzard_Calendar");
    let toc = wow_ui_sim::loader::find_toc_file(&addon_dir)
        .expect("Blizzard_Calendar TOC should resolve");
    wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).expect("Blizzard_Calendar should load");
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Lua environment should initialize");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn club_calendar_tooltip_renders_only_in_range_events() {
    let env = load_full_game_ui();
    env.exec(
        r#"
        A_Admin.SetCalendarClubEvents('[{"clubId":"guild-0","title":"In range one","inviteStatus":0,"startTime":{"year":2026,"month":4,"monthDay":15,"weekday":4,"hour":13,"minute":0}},{"clubId":"guild-0","title":"In range two","inviteStatus":0,"startTime":{"year":2026,"month":4,"monthDay":17,"weekday":6,"hour":13,"minute":0}},{"clubId":"guild-0","title":"Outside range","inviteStatus":0,"startTime":{"year":2026,"month":4,"monthDay":20,"weekday":2,"hour":13,"minute":0}}]')
        CommunitiesFrame:Show()
        CommunitiesFrame:SelectClub('guild-0')
        CommunitiesFrame.CommunitiesCalendarButton:OnEnter()
        local lines = {}
        for index = 1, GameTooltip:NumLines() do
            local line = _G['GameTooltipTextLeft' .. index]
            if line and line:GetText() then table.insert(lines, line:GetText()) end
        end
        local text = table.concat(lines, '\n')
        assert(text:find('In range one', 1, true), text)
        assert(text:find('In range two', 1, true), text)
        assert(not text:find('Outside range', 1, true), text)
        "#,
    )
    .expect("CommunitiesCalendar's real tooltip should render supplied in-range events");
}

#[test]
fn calendar_create_event_dropdown_renders_supplied_types() {
    let env = load_full_game_ui();
    load_calendar_addon(&env);
    env.exec(
        r#"
        A_Admin.SetCalendarEventTypes('[{"displayString":"CALENDAR_TYPE_PVP","eventType":2},{"displayString":"CALENDAR_TYPE_MEETING","eventType":3}]')
        CalendarCreateEventFrame:Show()
        local dropdown = CalendarCreateEventFrame.EventTypeDropdown
        dropdown:Show()
        dropdown:GenerateMenu()
        dropdown:OpenMenu()
        local menu = Menu.GetManager():GetOpenMenu()
        assert(menu, 'calendar event type menu should open')
        local labels = {}
        for _, child in ipairs({menu:GetChildren()}) do
            if child.fontString then table.insert(labels, child.fontString:GetText()) end
        end
        local text = table.concat(labels, '\n')
        assert(text:find(CALENDAR_TYPE_PVP, 1, true), text)
        assert(text:find(CALENDAR_TYPE_MEETING, 1, true), text)
        "#,
    )
    .expect("the real create-event dropdown should render supplied type labels");
}

#[test]
fn calendar_event_type_names_follow_supplied_catalog() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(#C_Calendar.EventGetTypes() == 0)
        A_Admin.SetCalendarEventTypes('[{"displayString":"Meeting","eventType":3},{"displayString":"Raid","eventType":1}]')
        local types = C_Calendar.EventGetTypes()
        assert(#types == 2 and types[1] == "Meeting" and types[2] == "Raid")
        types[1] = "Modified"
        assert(C_Calendar.EventGetTypes()[1] == "Meeting")
        A_Admin.SetCalendarEventTypes('[{"displayString":"Dungeon","eventType":2}]')
        local types = C_Calendar.EventGetTypes()
        assert(#types == 1 and types[1] == "Dungeon")
    "#).unwrap();
}
