//! Forever 1.60.1 ships a CombatAudioAlertDocumentation.lua byte-identical to
//! Retail 12.1.5, so it publishes the same setting model. Its AudioAssist
//! settings consumer cannot run here yet: the Forever registrants stop on
//! unmodeled Settings methods and `C_VoiceChat.IsVoiceChatConnected`.
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
    end
    assert(C_CombatAudioAlert.GetSpeakerVolume == nil, "Forever docs drop SpeakerVolume")
"#;

#[test]
fn forever_combat_audio_settings_follow_their_inputs() {
    let env = WowLuaEnv::new().expect("Lua environment");
    env.exec(ROUND_TRIPS)
        .expect("combat audio settings follow their inputs");
}
