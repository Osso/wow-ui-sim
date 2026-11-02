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
