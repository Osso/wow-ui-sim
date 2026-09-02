//! `AlertContainerMixin:UpdateAnchors` (AlertFrames.lua:416) stacks its
//! subsystems and advances the anchor past a frame only when
//! `not frame.IsInDefaultPosition or frame:IsInDefaultPosition()`: absence
//! of the method means "in default position". The simulator gave EVERY frame
//! an `IsInDefaultPosition` answering false without edit-mode `systemInfo`,
//! so with text-to-speech enabled the TTS button and the Quick Join toast
//! both anchored to the container's bottom-left and overlapped; the client
//! stacks Quick Join above the TTS button.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_settled_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    for (name, toc_path) in &discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game) {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("Failed to load Blizzard addon {name}: {err}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env
}

#[test]
fn quick_join_toast_stacks_above_the_shown_text_to_speech_button() {
    let env = load_settled_game_ui();
    let (tts_shown, relative_to, point): (bool, String, String) = env
        .eval(
            r#"
            SetCVar("textToSpeech", "1")
            TextToSpeechButtonFrame.Button:UpdateVisibleState()
            ChatAlertFrame:UpdateAnchors()
            local point, relativeTo = QuickJoinToastButton:GetPoint(1)
            return TextToSpeechButtonFrame.Button:IsShown(),
                relativeTo and relativeTo:GetName() or "nil", point or "nil"
            "#,
        )
        .expect("alert container probe");
    assert!(tts_shown, "the TTS button shows once the CVar is on");
    assert_eq!(relative_to, "TextToSpeechButtonFrame", "Quick Join anchors above the TTS button ({point})");
}
