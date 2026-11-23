#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::paths::default_blizzard_ui_addons_path;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
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
fn blizzard_class_trial_top_level_frames_are_defined(env: &WowLuaEnv) {

    let frames_present: bool = env
        .eval(
            "return _G.ClassTrialThanksForPlayingDialog ~= nil \
                and _G.ExpansionTrialThanksForPlayingDialog ~= nil \
                and _G.ClassTrialTimerDisplay ~= nil",
        )
        .expect("frame query should succeed");
    assert!(
        frames_present,
        "ClassTrialThanksForPlayingDialog, ExpansionTrialThanksForPlayingDialog, ClassTrialTimerDisplay should be defined after load"
    );

    let mixins_present: bool = env
        .eval(
            "return type(ClassTrialDialogMixin) == 'table' \
                and type(ExpansionTrialDialogMixin) == 'table' \
                and type(ClassTrialTimerDisplayMixin) == 'table'",
        )
        .expect("mixin query should succeed");
    assert!(
        mixins_present,
        "Blizzard_ClassTrial mixin tables should be defined after load"
    );
}
}

prefork_full_ui_case! {
fn class_trial_timer_display_show_and_hide_run_without_errors(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("ClassTrialTimerDisplay:Show(); ClassTrialTimerDisplay:Hide()")
        .expect("ClassTrialTimerDisplay Show/Hide should run");

    let unexpected_errors: Vec<String> = env.state().borrow().lua_errors.clone();
    assert!(
        unexpected_errors.is_empty(),
        "ClassTrialTimerDisplay Show/Hide emitted Lua errors:\n  {}",
        unexpected_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn class_trial_timer_displays_supplied_deadline(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetManualClock(10)
        A_Admin.SetClassTrialState(false, 0)
        ClassTrialTimerDisplay:CheckShowTimer()
        assert(not ClassTrialTimerDisplay:IsShown())

        A_Admin.SetClassTrialState(true, 130)
        ClassTrialTimerDisplay:CheckShowTimer()
        assert(ClassTrialTimerDisplay:IsShown())
        assert(ClassTrialTimerDisplay.TimerText:GetText() ==
            CLASS_TRIAL_TIMER_DIALOG_TEXT_HAS_REMAINING_TIME:format(
                SecondsToTime(120, false, true, 1, true)))

        A_Admin.SetClassTrialState(true, 250)
        ClassTrialTimerDisplay:CheckShowTimer()
        assert(ClassTrialTimerDisplay.TimerText:GetText() ==
            CLASS_TRIAL_TIMER_DIALOG_TEXT_HAS_REMAINING_TIME:format(
                SecondsToTime(240, false, true, 1, true)))

        A_Admin.AdvanceTime(10)
        assert(ClassTrialTimerDisplay.TimerText:GetText() ==
            CLASS_TRIAL_TIMER_DIALOG_TEXT_HAS_REMAINING_TIME:format(
                SecondsToTime(230, false, true, 1, true)))
        "#,
    )
    .expect("class-trial timer should use the supplied logout deadline");
}
}
