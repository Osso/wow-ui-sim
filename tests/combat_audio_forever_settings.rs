//! Forever 1.60.1 ships a CombatAudioAlertDocumentation.lua byte-identical to
//! Retail 12.1.5 and the original AudioAssist settings (Blizzard_SettingsDefinitions_Shared)
//! read and write these values through proxy settings.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn forever_game() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Forever Lua environment");
    crate::common::load_game_startup_inventory(&env);
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    let game = wow_ui_sim::screen::ScreenKind::Game;
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, game);
    env
}

const PROXY_ROUND_TRIPS: &str = r#"
    local volume = Settings.GetSetting("PROXY_CAA_VOLUME")
    local speed = Settings.GetSetting("PROXY_CAA_SPEED")
    local throttle = Settings.GetSetting("PROXY_CAA_PLAYER_HEALTH_THROTTLE")
    local targeted = Settings.GetSetting("PROXY_CAA_SAY_IF_TARGETED")
    assert(volume and speed and throttle and targeted, "AudioAssist registrant ran")
    local general = Enum.CombatAudioAlertCategory.General
    local health = Enum.CombatAudioAlertThrottle.PlayerHealth
    local sayTargeted = Enum.CombatAudioAlertSpecSetting.SayIfTargeted
    for _, value in ipairs({ 35, 80 }) do
        volume:SetValue(value)
        assert(C_CombatAudioAlert.GetCategoryVolume(general) == value, "volume write")
        speed:SetValue(value / 10)
        assert(C_CombatAudioAlert.GetSpeakerSpeed() == value / 10, "speed write")
        throttle:SetValue(value / 5)
        assert(C_CombatAudioAlert.GetThrottle(health) == value / 5, "throttle write")
    end
    for _, value in ipairs({ 1, 0 }) do
        C_CombatAudioAlert.SetSpecSetting(sayTargeted, value)
        assert(targeted:GetValue() == (value == 1), "spec setting read")
        C_CombatAudioAlert.SetCategoryVolume(general, value + 10)
        assert(volume:GetValue() == value + 10, "volume read")
    end
"#;

#[test]
fn forever_audio_assist_settings_round_trip_combat_audio_state() {
    crate::common::with_timeout(120, || {
        let env = forever_game();
        env.exec(PROXY_ROUND_TRIPS)
            .expect("AudioAssist proxy settings follow C_CombatAudioAlert state");
    });
}
