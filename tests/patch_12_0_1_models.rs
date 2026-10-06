#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn patch_12_0_1_audio_category_settings_are_independent() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        assert(C_CombatAudioAlert.SetCategoryVoice(0, 4))
        assert(C_CombatAudioAlert.SetCategoryVoice(1, 7))
        assert(C_CombatAudioAlert.SetCategoryVolume(0, 0.25))
        assert(C_CombatAudioAlert.SetCategoryVolume(1, 0.75))
        assert(C_CombatAudioAlert.GetCategoryVoice(0) == 4)
        assert(C_CombatAudioAlert.GetCategoryVoice(1) == 7)
        assert(C_CombatAudioAlert.GetCategoryVolume(0) == 0.25)
        assert(C_CombatAudioAlert.GetCategoryVolume(1) == 0.75)
        assert(not pcall(C_CombatAudioAlert.SetCategoryVoice, 'bad', 1))
        assert(not pcall(C_CombatAudioAlert.SetCategoryVolume, 0, 0/0))
        assert(C_CombatAudioAlert.GetCategoryVolume(0) == 0.25)
    "#).unwrap();
}

#[test]
fn patch_12_0_1_warning_visibility_and_sound_settings_roundtrip() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        C_EncounterWarnings.SetWarningsShown(false)
        C_EncounterWarnings.SetPlayCustomSoundsWhenHidden(true)
        assert(not C_EncounterWarnings.GetWarningsShown())
        assert(C_EncounterWarnings.GetPlayCustomSoundsWhenHidden())
        C_EncounterWarnings.SetWarningsShown(true)
        C_EncounterWarnings.SetPlayCustomSoundsWhenHidden(false)
        assert(C_EncounterWarnings.GetWarningsShown())
        assert(not C_EncounterWarnings.GetPlayCustomSoundsWhenHidden())
    "#).unwrap();
}
