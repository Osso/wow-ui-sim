//! Retail AudioAssist registrant selects its voice-connection wait path.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn audio_assist_selects_connection_or_tts_wait_from_supplied_state(env: &WowLuaEnv) {
    env.exec(r#"
        local suffix = 'Blizzard_SettingsDefinitions_Shared/AudioAssist.lua'
        for _, callback in ipairs(SettingsRegistrar.registrants) do
            local info = debug.getinfo(callback, 'S')
            local path = info.source:gsub('\\', '/')
            if path:sub(-#suffix) == suffix then
                assert(not AudioAssistOriginal)
                AudioAssistOriginal = callback
            end
        end
        assert(AudioAssistOriginal, 'runtime-selected AudioAssist registrant missing')
        local oldVertical = Settings.RegisterVerticalLayoutCategory
        local oldRegister = Settings.RegisterCategory
        local oldOnce = EventUtil.RegisterOnceFrameEventAndCallback
        local oldVoices = C_VoiceChat.GetRemoteTtsVoices
        function ObserveAudioAssist()
            local observed
            Settings.RegisterVerticalLayoutCategory = function() return {}, {} end
            Settings.RegisterCategory = function() end
            EventUtil.RegisterOnceFrameEventAndCallback = function(event) observed = event end
            C_VoiceChat.GetRemoteTtsVoices = function() return {} end
            local ok, err = pcall(AudioAssistOriginal)
            Settings.RegisterVerticalLayoutCategory = oldVertical
            Settings.RegisterCategory = oldRegister
            EventUtil.RegisterOnceFrameEventAndCallback = oldOnce
            C_VoiceChat.GetRemoteTtsVoices = oldVoices
            assert(ok, tostring(err))
            return observed
        end
        assert(ObserveAudioAssist() == 'VOICE_CHAT_CONNECTION_SUCCESS')
    "#).unwrap();
    env.state().borrow_mut().voice_chat.connection_ready = true;
    env.exec("assert(ObserveAudioAssist() == 'VOICE_CHAT_TTS_VOICES_UPDATE')").unwrap();
    env.state().borrow_mut().voice_chat.connection_excluded = true;
    env.exec("assert(ObserveAudioAssist() == 'VOICE_CHAT_CONNECTION_SUCCESS')").unwrap();
}
}
