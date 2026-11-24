//! Original manager targeting flow with supplied unit-target identities.
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::{WowLuaEnv, state::TargetInfo};

fn target(guid: &str) -> TargetInfo {
    TargetInfo {
        unit_id: "target".into(), name: "Fixture".into(), class_index: 1, level: 80,
        health: 100, health_max: 100, power: 0, power_max: 0, power_type: 1,
        power_type_name: "RAGE".into(), is_player: false, is_enemy: true,
        guid: guid.into(), classification: "elite".into(), creature_type: "Humanoid".into(),
        reaction: 2, has_vehicle_ui: false, interaction: Default::default(),
    }
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn manager_announces_once_and_rearms_after_target_changes(env: &WowLuaEnv) {
    {
        let mut sim = env.state().borrow_mut();
        sim.current_target = Some(target("Creature-0-0-0-0-123-1"));
        sim.unit_target_guids.insert("target".into(), "Player-1-00000001".into());
    }
    env.exec(r#"
        assert(UnitExists('targettarget') and UnitIsUnit('targettarget', 'player'))
        C_CombatAudioAlert.SetSpecSetting(Enum.CombatAudioAlertSpecSetting.SayIfTargeted, 1)
        local method = CombatAudioAlertManagerMixin.ProcessUnitTargetChanged
        local addon
        for i = 1, 20 do
            local key, value = debug.getupvalue(method, i)
            if not key then break end
            if key == 'addonTable' then addon = value end
        end
        assert(addon, 'original manager addon table missing')
        originalSpeak = addon.SpeakText
        announcements = {}
        addon.SpeakText = function(text) announcements[#announcements + 1] = text end
        method(CombatAudioAlertManager, 'target')
        method(CombatAudioAlertManager, 'target')
        assert(#announcements == 1, 'announce first targeting only')
    "#).unwrap();
    env.state().borrow_mut().unit_target_guids.insert("target".into(), "Creature-0-0-0-0-888-1".into());
    env.exec(r#"
        assert(UnitExists('targettarget') and not UnitIsUnit('targettarget', 'player'))
        CombatAudioAlertManagerMixin.ProcessUnitTargetChanged(CombatAudioAlertManager, 'target')
        assert(#announcements == 1)
    "#).unwrap();
    env.state().borrow_mut().unit_target_guids.insert("target".into(), "Player-1-00000001".into());
    env.exec(r#"
        CombatAudioAlertManagerMixin.ProcessUnitTargetChanged(CombatAudioAlertManager, 'target')
        assert(#announcements == 2, 'removal rearms announcement')
    "#).unwrap();
}
}

fn submit_playsound_slash(env: &WowLuaEnv, sound_enum: &str) -> f64 {
    let script = format!(
        r#"
        local soundEnum = {sound_enum}
        ChatFrame1EditBox:SetText(SLASH_CAA1 .. " " .. SLASH_CAA_PLAY_SOUND .. " " .. soundEnum)
        ChatEdit_SendText(ChatFrame1EditBox, false)
        local queuedID, count = nil, 0
        for id, queuedEnum in pairs(CombatAudioAlertManager.queuedSounds) do
            count = count + 1
            if queuedEnum == soundEnum then queuedID = tonumber(id) end
        end
        if count ~= 1 then return nil end
        return queuedID
        "#
    );
    env.eval(&script).expect("CAA slash command should queue its utterance")
}

fn sound_kit_for(env: &WowLuaEnv, sound_enum: &str) -> i32 {
    env.eval(&format!("return CooldownViewerUtil.GetSoundTypeSoundKit({sound_enum})"))
        .expect("sound enum should resolve to a sound kit")
}

fn assert_recorded_speech_request(env: &WowLuaEnv, index: usize, sound_enum: &str) {
    let expected_text: String = env
        .eval(&format!("return CooldownViewerUtil.GetSoundTypeText({sound_enum})"))
        .expect("sound enum should resolve to its speech text");
    let expected_category: f64 = env
        .eval("return Enum.CombatAudioAlertCategory.General")
        .expect("general category should be a numeric enum");
    let request = env.state().borrow().combat_audio_speak_requests[index].clone();
    assert_eq!(request.text, expected_text);
    assert_eq!(request.category, expected_category);
    assert!(request.allow_overlap);
}

fn finish_playsound(env: &WowLuaEnv, expected_sound_kit: i32) {
    env.process_timers().expect("process later timer tick");
    let cleared: bool = env
        .eval("return next(CombatAudioAlertManager.queuedSounds) == nil")
        .expect("consumer queue should be readable");
    assert!(cleared, "completion should consume the queued sound enum");
    let sim = env.state();
    let state = sim.borrow();
    let played = state.last_sound_request.as_ref().map(|request| request.sound_kit_id);
    assert_eq!(played, Some(expected_sound_kit as u32));
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn combat_audio_speak_text_completes_two_play_sound_slashes(env: &WowLuaEnv) {
    let first_kit = sound_kit_for(env, "Enum.CooldownViewerSound.War3Bell");
    let second_kit = sound_kit_for(env, "Enum.CooldownViewerSound.War3CrunchyBell");
    assert_ne!(first_kit, second_kit);
    let request_count = env.state().borrow().combat_audio_speak_requests.len();
    assert_eq!(request_count, 0, "fresh environment has no prior CAA speech");
    let first_id = submit_playsound_slash(env, "Enum.CooldownViewerSound.War3Bell");
    assert_eq!(first_id, 1.0);
    assert_recorded_speech_request(env, request_count, "Enum.CooldownViewerSound.War3Bell");
    finish_playsound(env, first_kit);
    let second_id = submit_playsound_slash(env, "Enum.CooldownViewerSound.War3CrunchyBell");
    assert_eq!(second_id, 2.0);
    assert_ne!(first_id, second_id);
    assert_recorded_speech_request(env, request_count + 1, "Enum.CooldownViewerSound.War3CrunchyBell");
    finish_playsound(env, second_kit);
}
}
