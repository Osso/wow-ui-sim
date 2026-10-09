#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn behavioral_messaging_dependency_chain() -> Vec<(&'static str, PathBuf)> {
    let ui = blizzard_ui_dir();
    vec![
        (
            "Blizzard_StatusTrayManager",
            ui.join("Blizzard_StatusTrayManager/Blizzard_StatusTrayManager.toc"),
        ),
        (
            "Blizzard_StatusUI",
            ui.join("Blizzard_StatusUI/Blizzard_StatusUI.toc"),
        ),
        (
            "Blizzard_GMChatUI",
            ui.join("Blizzard_GMChatUI/Blizzard_GMChatUI.toc"),
        ),
        (
            "Blizzard_BehavioralMessaging",
            ui.join("Blizzard_BehavioralMessaging/Blizzard_BehavioralMessaging.toc"),
        ),
    ]
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

fn load_behavioral_messaging_chain(env: &WowLuaEnv) {
    for (name, toc_path) in behavioral_messaging_dependency_chain() {
        load_addon(&env.loader_env(), &toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
}

#[test]
fn c_behavioral_messaging_namespace_is_registered() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let registered: bool = env
        .eval(
            "return type(C_BehavioralMessaging) == 'table' \
             and type(C_BehavioralMessaging.SendNotificationReceipt) == 'function'",
        )
        .expect("namespace probe should succeed");
    assert!(
        registered,
        "C_BehavioralMessaging.SendNotificationReceipt should be registered"
    );
}

#[test]
fn behavioral_messaging_global_strings_exist() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    let strings_present: bool = env
        .eval(
            "return type(BEHAVIORAL_NOTIFICATION_OPEN) == 'string' \
             and type(BEHAVIORAL_NOTIFICATION_WARNING) == 'string' \
             and type(BEHAVIORAL_DETAILS_TITLE_BAR) == 'string' \
             and type(BEHAVIORAL_DETAILS_CLOSE_BUTTON) == 'string'",
        )
        .expect("global strings probe should succeed");
    assert!(
        strings_present,
        "BEHAVIORAL_* global strings should be defined"
    );
}

prefork_full_ui_case! {
    fn blizzard_behavioral_messaging_loads_without_errors(env: &WowLuaEnv) {
        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }

        load_behavioral_messaging_chain(&env);

        let load_errors: Vec<String> = env.state().borrow().lua_errors.clone();
        assert!(
            load_errors.is_empty(),
            "Blizzard_BehavioralMessaging emitted Lua errors during load:\n  {}",
            load_errors.join("\n  ")
        );

        let frames_present: bool = env
            .eval(
                "return BehavioralMessagingTray ~= nil \
                 and BehavioralMessagingDetails ~= nil \
                 and type(BehavioralMessagingTrayMixin) == 'table' \
                 and type(BehavioralMessagingNotificationMixin) == 'table' \
                 and type(BehavioralMessagingDetailsMixin) == 'table'",
            )
            .expect("frame/mixin query should succeed");
        assert!(
            frames_present,
            "BehavioralMessaging frames and mixins should be defined after load"
        );
    }
}

prefork_full_ui_case! {
    fn behavioral_notification_event_does_not_error(env: &WowLuaEnv) {
        load_behavioral_messaging_chain(&env);

        {
            let mut state = env.state().borrow_mut();
            state.lua_errors.clear();
            state.lua_error_records.clear();
            state.lua_error_counts.clear();
        }

        env.exec(
            "BehavioralMessagingTray:OnEvent('BEHAVIORAL_NOTIFICATION', 'ComplaintWarning_Social', 1)",
        )
        .expect("OnEvent dispatch should run");

        let errors: Vec<String> = env.state().borrow().lua_errors.clone();
        assert!(
            errors.is_empty(),
            "BehavioralMessagingTray:OnEvent emitted Lua errors:\n  {}",
            errors.join("\n  ")
        );
    }
}
