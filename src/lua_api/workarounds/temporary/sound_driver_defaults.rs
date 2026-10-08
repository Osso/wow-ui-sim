//! Temporary silent sound-driver defaults.
//!
//! The simulator does not model selectable audio devices. These defaults keep
//! options UI probes inert while making the unsupported device list explicit.

const SOUND_DRIVER_DEFAULTS_LUA: &str = r#"
if type(C_CombatAudioAlert) ~= "table" then
    C_CombatAudioAlert = {}
end
if type(C_Sound) ~= "table" then
    C_Sound = {}
end
if type(GetBuildInfo) == "function" and select(4, GetBuildInfo()) >= 120100 then
    if rawget(C_CombatAudioAlert, "SpeakText") == nil then
        function C_CombatAudioAlert.SpeakText()
            return 0
        end
    end
end
if MuteSoundFile == nil then
    function MuteSoundFile()
        return true
    end
end
if UnmuteSoundFile == nil then
    function UnmuteSoundFile()
        return true
    end
end
if rawget(C_Sound, "GetSoundScaledVolume") == nil then
    function C_Sound.GetSoundScaledVolume()
        return 1
    end
end
if rawget(C_Sound, "IsPlaying") == nil then
    function C_Sound.IsPlaying()
        return false
    end
end
if rawget(C_Sound, "PlayItemSound") == nil then
    function C_Sound.PlayItemSound() end
end
if rawget(C_Sound, "PlaySoundFile") == nil then
    function C_Sound.PlaySoundFile() end
end
if rawget(C_Sound, "PlayVocalErrorSound") == nil then
    function C_Sound.PlayVocalErrorSound() end
end
if Sound_GameSystem_GetNumOutputDrivers == nil then
    function Sound_GameSystem_GetNumOutputDrivers() return 1 end
end
if Sound_GameSystem_GetOutputDriverNameByIndex == nil then
    function Sound_GameSystem_GetOutputDriverNameByIndex(index)
        if index == 0 then
            return "Silent Output Device"
        end
        return nil
    end
end
if Sound_GameSystem_GetNumInputDrivers == nil then
    function Sound_GameSystem_GetNumInputDrivers() return 1 end
end
if Sound_GameSystem_GetInputDriverNameByIndex == nil then
    function Sound_GameSystem_GetInputDriverNameByIndex(index)
        if index == 0 then
            return "Silent Input Device"
        end
        return nil
    end
end
if Sound_ChatSystem_GetNumOutputDrivers == nil then
    function Sound_ChatSystem_GetNumOutputDrivers() return 1 end
end
if Sound_ChatSystem_GetOutputDriverNameByIndex == nil then
    function Sound_ChatSystem_GetOutputDriverNameByIndex(index)
        if index == 0 then
            return "Silent Voice Output Device"
        end
        return nil
    end
end
if Sound_ChatSystem_GetNumInputDrivers == nil then
    function Sound_ChatSystem_GetNumInputDrivers() return 1 end
end
if Sound_ChatSystem_GetInputDriverNameByIndex == nil then
    function Sound_ChatSystem_GetInputDriverNameByIndex(index)
        if index == 0 then
            return "Silent Voice Input Device"
        end
        return nil
    end
end
if Sound_GameSystem_RestartSoundSystem == nil then
    function Sound_GameSystem_RestartSoundSystem() end
end
"#;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(SOUND_DRIVER_DEFAULTS_LUA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn installs_silent_sound_driver_defaults() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        let result: String = env
            .eval(
                r#"
                if Sound_GameSystem_GetNumOutputDrivers() ~= 1 then return "game_output_count" end
                if type(C_CombatAudioAlert) ~= "table" then return "combat_audio_alert" end
                if type(C_Sound) ~= "table" then return "sound_namespace" end
                if C_Sound.GetSoundScaledVolume() ~= 1 then return "sound_volume" end
                if C_Sound.IsPlaying() ~= false then return "sound_playing" end
                if MuteSoundFile(123) ~= true then return "mute" end
                if UnmuteSoundFile(123) ~= true then return "unmute" end
                C_Sound.PlayItemSound(1)
                C_Sound.PlaySound(1)
                C_Sound.PlaySoundFile("silent.ogg")
                C_Sound.PlayVocalErrorSound(1)
                if Sound_GameSystem_GetOutputDriverNameByIndex(0) ~= "Silent Output Device" then return "game_output_name" end
                if Sound_GameSystem_GetOutputDriverNameByIndex(1) ~= nil then return "game_output_extra" end
                if Sound_GameSystem_GetNumInputDrivers() ~= 1 then return "game_input_count" end
                if Sound_GameSystem_GetInputDriverNameByIndex(0) ~= "Silent Input Device" then return "game_input_name" end
                if Sound_ChatSystem_GetNumOutputDrivers() ~= 1 then return "chat_output_count" end
                if Sound_ChatSystem_GetOutputDriverNameByIndex(0) ~= "Silent Voice Output Device" then return "chat_output_name" end
                if Sound_ChatSystem_GetNumInputDrivers() ~= 1 then return "chat_input_count" end
                if Sound_ChatSystem_GetInputDriverNameByIndex(0) ~= "Silent Voice Input Device" then return "chat_input_name" end
                Sound_GameSystem_RestartSoundSystem()
                return "ok"
                "#,
            )
            .expect("sound defaults should be callable");

        assert_eq!(result, "ok");
    }

    #[test]
    fn preserves_existing_sound_driver_members() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            C_CombatAudioAlert = { Existing = true }
            C_Sound = { Existing = true, IsPlaying = function() return true end }
            function MuteSoundFile() return "muted" end
            function Sound_GameSystem_GetNumOutputDrivers() return 4 end
            function Sound_ChatSystem_GetOutputDriverNameByIndex() return "Existing Voice" end
            "#,
        )
        .expect("fixture should install existing sound members");

        {
            let mut lua = env.lua.borrow_mut();
            super::apply_bootstrap(&mut lua).expect("sound defaults should apply");
        }

        let result: String = env
            .eval(
                r#"
                if Sound_GameSystem_GetNumOutputDrivers() ~= 4 then return "overwrote_count" end
                if C_CombatAudioAlert.Existing ~= true then return "overwrote_combat_audio" end
                if C_Sound.Existing ~= true then return "overwrote_sound_namespace" end
                if C_Sound.IsPlaying() ~= true then return "overwrote_sound_method" end
                if MuteSoundFile(1) ~= "muted" then return "overwrote_mute" end
                if UnmuteSoundFile(1) ~= true then return "missing_unmute" end
                if type(C_Sound.PlayVocalErrorSound) ~= "function" then return "missing_vocal_error" end
                if Sound_ChatSystem_GetOutputDriverNameByIndex(0) ~= "Existing Voice" then return "overwrote_name" end
                if type(Sound_GameSystem_GetInputDriverNameByIndex) ~= "function" then return "missing_game_input" end
                if type(Sound_GameSystem_RestartSoundSystem) ~= "function" then return "missing_restart" end
                return "ok"
                "#,
            )
            .expect("sound preservation probe should run");

        assert_eq!(result, "ok");
    }
}
