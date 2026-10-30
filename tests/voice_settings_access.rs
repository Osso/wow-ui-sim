//! Supplied access through the pinned Audio.lua polling callback, not a UI rewrite.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn original_audio_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_SettingsDefinitions_Shared")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(toc.files.iter().any(|file| file.ends_with("Audio.xml")));
    assert!(
        discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
            .iter()
            .any(|(name, _)| name == "Blizzard_SettingsDefinitions_Shared")
    );
    let env = new_blizzard_addon_env(&ui);
    let expected = if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) { Some(true) } else { None };
    assert_eq!(env.state().borrow().voice_chat.can_access_settings, expected);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SettingsDefinitions_Shared"], &[]);
    env.exec(include_str!("voice_settings_access/original_callback.lua"))
        .expect("selected original Audio registration must create its polling callback");
    env
}

fn assert_original_branch(env: &WowLuaEnv, supplied: bool) {
    env.state().borrow_mut().voice_chat.can_access_settings = Some(supplied);
    let enters_initialization: bool = env
        .eval("return ObserveOriginalVoiceSettings()")
        .expect("original polling callback must consume the supplied observation");
    assert_eq!(enters_initialization, supplied);
    let (arity, actual): (i32, bool) = env
        .eval("local function pack(...) return select('#', ...), ... end; return pack(C_VoiceChat.CanAccessSettings())")
        .unwrap();
    assert_eq!((arity, actual), (1, supplied));
}

fn exercise_original_branches(initial: bool) {
    if ACTIVE == ClientProfile::Wrath {
        let env = WowLuaEnv::new().unwrap();
        assert!(
            env.eval::<bool>("return rawget(C_VoiceChat, 'CanAccessSettings') == nil")
                .unwrap()
        );
        return;
    }
    let first = original_audio_env();
    assert_original_branch(&first, initial);
    assert_original_branch(&first, !initial);
    let second = original_audio_env();
    assert_original_branch(&second, !initial);
    assert_original_branch(&first, initial);
    assert_original_branch(&second, !initial);
    first.state().borrow_mut().voice_chat.can_access_settings = None;
    let error = first
        .eval::<bool>("return ObserveOriginalVoiceSettings()")
        .expect_err("unknown access must not select either original branch");
    assert!(
        error
            .to_string()
            .contains("C_VoiceChat.CanAccessSettings: settings access has not been supplied"),
        "{error}"
    );
    assert_original_branch(&first, false);
    assert_original_branch(&first, true);
    assert_original_branch(&second, !initial);
}

#[test]
fn original_audio_denied_then_allowed_isolation_and_unset_recovery() {
    exercise_original_branches(false);
}

#[test]
fn original_audio_allowed_then_denied_isolation_and_unset_recovery() {
    exercise_original_branches(true);
}

#[test]
fn original_audio_initially_unset_then_supplied() {
    if ACTIVE == ClientProfile::Wrath {
        return;
    }
    let env = original_audio_env();
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        assert!(env.eval::<bool>("return ObserveOriginalVoiceSettings()").unwrap());
    }
    env.state().borrow_mut().voice_chat.can_access_settings = None;
    let error = env
        .eval::<bool>("return ObserveOriginalVoiceSettings()")
        .expect_err("unset observation must be explicitly unknown");
    assert!(
        error
            .to_string()
            .contains("C_VoiceChat.CanAccessSettings: settings access has not been supplied"),
        "{error}"
    );
    assert_original_branch(&env, false);
    assert_original_branch(&env, true);
}
