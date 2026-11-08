//! Supplied access through the pinned Audio.lua polling callback, not a UI rewrite.
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::lua_api::WowLuaEnv;

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

pub(crate) fn original_audio_env() -> WowLuaEnv {
    original_audio_env_with_roots(&[])
}

/// The original Audio registrant schedules its polling ticker on this clock, so a
/// later `AdvanceTime` reaches the five-second deadline deterministically.
pub(crate) fn original_audio_env_with_manual_clock() -> WowLuaEnv {
    original_audio_env_inner(&[], true)
}

/// Extra roots load before the Audio closure, as the client loads them at startup.
pub(crate) fn original_audio_env_with_roots(extra_roots: &[&str]) -> WowLuaEnv {
    original_audio_env_inner(extra_roots, false)
}

fn original_audio_env_inner(extra_roots: &[&str], manual_clock: bool) -> WowLuaEnv {
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
    if manual_clock {
        env.exec("A_Admin.SetManualClock(0)")
            .expect("manual clock must be enabled before the Audio registrant runs");
    }
    let expected = match ACTIVE {
        ClientProfile::Retail
        | ClientProfile::Ptr
        | ClientProfile::WowForever
        | ClientProfile::Mists
        | ClientProfile::Era
        | ClientProfile::Anniversary => Some(false),
        ClientProfile::Wrath => None,
    };
    assert_eq!(env.state().borrow().voice_chat.can_access_settings, expected);
    let roots: Vec<&str> = extra_roots.iter().copied().chain(["Blizzard_SettingsDefinitions_Shared"]).collect();
    load_blizzard_addon_closure_into_env(&env, &ui, &roots, &[]);
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
    // The initial observation is false on every declaring profile: the voice
    // service is never ready, so the original polling loop stays in its wait branch.
    assert!(!first.eval::<bool>("return ObserveOriginalVoiceSettings()").unwrap());
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

/// The pinned Audio.lua polls `CanAccessSettings` from a five-second ticker and
/// expects a boolean while the voice proxy is not ready. Crossing that deadline on
/// a profile without a voice proxy must continue polling, not raise a Lua error.
#[test]
fn original_audio_ticker_past_polling_deadline_does_not_raise_unsupplied_access() {
    if !matches!(
        ACTIVE,
        ClientProfile::Ptr | ClientProfile::Mists | ClientProfile::Era | ClientProfile::Anniversary
    ) {
        return;
    }
    let env = original_audio_env_with_manual_clock();
    env.state().borrow_mut().lua_errors.clear();
    env.exec("A_Admin.AdvanceTime(5)")
        .expect("advancing to the original polling deadline must not fail");
    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let access_errors: Vec<&String> = errors
        .iter()
        .filter(|error| error.contains("CanAccessSettings"))
        .collect();
    assert!(
        access_errors.is_empty(),
        "original polling ticker raised unsupplied settings access: {access_errors:?}"
    );
    assert_eq!(env.state().borrow().voice_chat.can_access_settings, Some(false));
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
    // Initially false: the original consumer keeps polling instead of initializing.
    let enters_initialization: bool = env.eval("return ObserveOriginalVoiceSettings()").unwrap();
    assert!(!enters_initialization);
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
