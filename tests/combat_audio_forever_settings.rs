//! Forever 1.60.1 ships a CombatAudioAlertDocumentation.lua byte-identical to
//! Retail 12.1.5, so it publishes the same setting model. The original
//! AudioAssist registrant (Blizzard_SettingsDefinitions_Shared/AudioAssist.lua)
//! registers the `PROXY_CAA_*` settings only after `C_VoiceChat.IsVoiceChatConnected`
//! (:1346) and a non-empty `C_VoiceChat.GetRemoteTtsVoices` (:1330).
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

const ROUND_TRIPS: &str = r#"
    local general = Enum.CombatAudioAlertCategory.General
    local health = Enum.CombatAudioAlertCategory.PlayerHealth
    local throttle = Enum.CombatAudioAlertThrottle.PlayerHealth
    local targeted = Enum.CombatAudioAlertSpecSetting.SayIfTargeted
    for _, value in ipairs({ 35, 80 }) do
        assert(C_CombatAudioAlert.SetCategoryVolume(general, value))
        assert(C_CombatAudioAlert.SetCategoryVoice(general, value + 1))
        C_CombatAudioAlert.SetSpeakerSpeed(value / 10)
        C_CombatAudioAlert.SetThrottle(throttle, value / 5)
        assert(C_CombatAudioAlert.SetSpecSetting(targeted, value % 2))
        assert(C_CombatAudioAlert.GetCategoryVolume(general) == value)
        assert(C_CombatAudioAlert.GetCategoryVoice(general) == value + 1)
        assert(C_CombatAudioAlert.GetCategoryVolume(health) == 0)
        assert(C_CombatAudioAlert.GetSpeakerSpeed() == value / 10)
        assert(C_CombatAudioAlert.GetThrottle(throttle) == value / 5)
        assert(C_CombatAudioAlert.GetSpecSetting(targeted) == value % 2)
        assert(Settings.GetValue("PROXY_CAA_VOLUME") == value)
        assert(Settings.GetValue("PROXY_CAA_SPEED") == value / 10)
    end
    assert(C_CombatAudioAlert.GetSpeakerVolume == nil, "Forever docs drop SpeakerVolume")
"#;

const PROXY_WRITES: &str = r#"
    for _, value in ipairs({ 20, 65 }) do
        Settings.SetValue("PROXY_CAA_VOLUME", value)
        Settings.SetValue("PROXY_CAA_SAY_IF_TARGETED", value % 2)
        local general = Enum.CombatAudioAlertCategory.General
        local targeted = Enum.CombatAudioAlertSpecSetting.SayIfTargeted
        assert(C_CombatAudioAlert.GetCategoryVolume(general) == value)
        assert(C_CombatAudioAlert.GetSpecSetting(targeted) == value % 2)
    end
"#;

fn forever_game(connected: bool, voices: &str) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.exec(&format!(
        "A_Admin.SetVoiceChatConnected({connected}); A_Admin.SetRemoteTtsVoices({voices})"
    ))
    .expect("supply the voice service state");
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    let game = wow_ui_sim::screen::ScreenKind::Game;
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
    env
}

fn proxy_registered(env: &WowLuaEnv) -> bool {
    env.eval("return Settings.GetSetting('PROXY_CAA_VOLUME') ~= nil")
        .expect("look up the proxy setting")
}

#[test]
fn forever_combat_audio_settings_follow_their_inputs() {
    crate::common::with_timeout(120, || {
        let env = forever_game(true, "{ { voiceID = 1, name = 'Remote One' } }");
        assert!(proxy_registered(&env), "AudioAssist registered PROXY_CAA_*");
        env.exec(ROUND_TRIPS)
            .expect("combat audio settings follow their inputs");
        env.exec(PROXY_WRITES)
            .expect("AudioAssist proxy settings write the combat audio model");
    });
}

#[test]
fn forever_audio_assist_waits_for_the_voice_connection() {
    crate::common::with_timeout(120, || {
        let env = forever_game(false, "{ { voiceID = 7, name = 'Remote Seven' } }");
        assert!(!proxy_registered(&env), "registration waits for the connection");
        let (connected, voice): (bool, String) = env
            .eval("return C_VoiceChat.IsVoiceChatConnected(), C_VoiceChat.GetRemoteTtsVoices()[1].name")
            .expect("read the supplied voice state");
        assert!(!connected);
        assert_eq!(voice, "Remote Seven");
        let env = forever_game(true, "{}");
        assert!(!proxy_registered(&env), "registration waits for remote voices");
    });
}
