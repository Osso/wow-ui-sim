//! Known-target identity and restriction cases complement the manager consumer.
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

#[test]
fn known_target_tokens_are_reresolved_before_identity_comparisons() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.current_target = Some(target("Creature-0-0-0-0-123-1"));
        sim.current_focus = Some(target("Creature-0-0-0-0-123-1"));
    }
    env.exec(r#"
        assert(C_CombatAudioAlert.AddToKnownTargetingList('target') == true)
        assert(C_CombatAudioAlert.AddToKnownTargetingList('focus') == false)
        assert(C_CombatAudioAlert.AddToKnownTargetingList('missing') == false)
        assert(C_CombatAudioAlert.AddToKnownTargetingList(42) == false)
        assert(not pcall(C_CombatAudioAlert.AddToKnownTargetingList, nil))
        assert(not pcall(C_CombatAudioAlert.RemoveFromKnownTargetingList, {}))
    "#).unwrap();
    env.state().borrow_mut().current_target = Some(target("Creature-0-0-0-0-124-2"));
    env.exec(r#"
        assert(C_CombatAudioAlert.AddToKnownTargetingList('focus') == true)
        assert(C_CombatAudioAlert.RemoveFromKnownTargetingList('target') == true)
        assert(C_CombatAudioAlert.RemoveFromKnownTargetingList('target') == false)
        assert(C_CombatAudioAlert.RemoveFromKnownTargetingList('focus') == true)
        assert(C_CombatAudioAlert.RemoveFromKnownTargetingList('focus') == false)
    "#).unwrap();
}

#[test]
fn tainted_calls_are_refused_before_invalid_arguments() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local reports = {}
        local listener = CreateFrame('Frame')
        listener:RegisterEvent('ADDON_ACTION_FORBIDDEN')
        listener:SetScript('OnEvent', function(_, _, source, name) reports[#reports + 1] = source .. '|' .. name end)
        local function addon()
            assert(select('#', C_CombatAudioAlert.AddToKnownTargetingList(nil)) == 0)
            assert(select('#', C_CombatAudioAlert.RemoveFromKnownTargetingList({})) == 0)
        end
        debug.setobjecttaint(addon, 'AudioFixture')
        addon()
        assert(reports[1] == 'AudioFixture|AddToKnownTargetingList()')
        assert(reports[2] == 'AudioFixture|RemoveFromKnownTargetingList()')
    "#).unwrap();
}
