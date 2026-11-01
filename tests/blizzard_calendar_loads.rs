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
