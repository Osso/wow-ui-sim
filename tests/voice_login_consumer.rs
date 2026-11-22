//! The loaded Blizzard voice toggle reads the supplied voice login state.

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, with_blizzard_addon_startup_shape};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

const VOICE_TOGGLE_LOGIN_CASE: &str = r#"
    local button = CreateFrame("Button", nil, UIParent, "ToggleVoiceMuteButtonTemplate")
    assert(C_VoiceChat.IsLoggedIn() == false, "default login state")
    assert(button:UpdateVisibleState() == false, "default button visibility")
    assert(not button:IsShown(), "default button is hidden")

    A_Admin.SetVoiceChatLoggedIn(false)
    assert(C_VoiceChat.IsLoggedIn() == false, "supplied logged-out state")
    assert(button:UpdateVisibleState() == false, "logged-out visibility")
    assert(not button:IsShown(), "logged-out button is hidden")

    A_Admin.SetVoiceChatLoggedIn(true)
    assert(C_VoiceChat.IsLoggedIn() == true, "supplied logged-in state")
    assert(C_VoiceChat.CanAccessSettings() == false, "readiness remains independent")
    assert(button:UpdateVisibleState() == true, "logged-in visibility")
    assert(button:IsShown(), "logged-in button is shown")

    A_Admin.SetVoiceChatLoggedIn(false)
    assert(button:UpdateVisibleState() == false, "updated logged-out visibility")
    assert(not button:IsShown(), "updated logged-out button is hidden")
"#;

fn assert_voice_addon_selected() {
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_VoiceToggleButton"))
        .expect("voice toggle TOC must be present");
    let toc = TocFile::from_file(&toc_path).expect("voice toggle TOC must parse");
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(
        discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
            .iter()
            .any(|(name, _)| name == "Blizzard_VoiceToggleButton"),
        "Blizzard_VoiceToggleButton must be selected for the game screen"
    );
}

fn assert_loaded_button_tracks_login(env: &WowLuaEnv) {
    env.exec(VOICE_TOGGLE_LOGIN_CASE)
        .expect("loaded Blizzard voice toggle must follow supplied login state");

    let state = env.state();
    let state = state.borrow();
    assert!(!state.voice_chat.connection_ready);
    assert_eq!(state.voice_chat.can_access_settings, Some(false));
}

#[test]
fn original_voice_toggle_buttons_follow_supplied_login_state() {
    assert!(matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::WowForever
    ));
    assert_voice_addon_selected();

    with_blizzard_addon_startup_shape(
        &["Blizzard_SharedXML", "Blizzard_VoiceToggleButton"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_SharedXML"));
            assert!(
                loaded
                    .iter()
                    .any(|name| name == "Blizzard_VoiceToggleButton")
            );
            assert_loaded_button_tracks_login(env);
        },
    );
}
