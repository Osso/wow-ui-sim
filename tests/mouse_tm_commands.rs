#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn marker_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        TargetUnit('player')
        markerEvents = 0
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('RAID_TARGET_UPDATE')
        frame:SetScript('OnEvent', function(_, event)
            assert(event == 'RAID_TARGET_UPDATE')
            markerEvents = markerEvents + 1
        end)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().events.drain();
    env
}

#[test]
fn tm_condition_uses_current_combat_state_and_dispatches_marker_events() {
    let env = marker_env();
    env.state().borrow_mut().player.in_combat = true;
    env.exec("C_Macro.RunMacroText('/tm [combat] 1')").unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        1.0
    );
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 1.0);
    assert_eq!(env.state().borrow().events.pending()[0].name, "RAID_TARGET_UPDATE");

    env.state().borrow_mut().player.in_combat = false;
    env.exec("C_Macro.RunMacroText('/tm [combat] 2')").unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        1.0
    );
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 1.0);
    assert_eq!(env.state().borrow().events.pending().len(), 1);
    env.exec("C_Macro.RunMacroText('/tm [nocombat] 3')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        3.0
    );
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 2.0);
}

#[test]
fn tm_unconditional_assignment_and_zero_clear_are_observable() {
    let env = marker_env();
    env.exec("C_Macro.RunMacroText('/tm 8\\n/tm 2')").unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('player')")
            .unwrap(),
        2.0
    );
    env.exec("C_Macro.RunMacroText('/tm 0')").unwrap();
    assert!(
        env.eval::<bool>("return GetRaidTargetIndex('target') == nil")
            .unwrap()
    );
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 3.0);
}

#[test]
fn tm_reuses_conjunctions_modifiers_and_first_matching_clause() {
    let env = marker_env();
    env.state().borrow_mut().player.in_combat = true;
    env.state().borrow_mut().modifier_keys.shift = false;
    env.exec("C_Macro.RunMacroText('/tm [combat,mod:shift] 4; [combat] 5; 6')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        5.0
    );
    env.state().borrow_mut().modifier_keys.shift = true;
    env.exec("C_Macro.RunMacroText('/tm [combat,mod:shift] 4; [combat] 5; 6')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        4.0
    );
    env.state().borrow_mut().player.in_combat = false;
    env.exec("C_Macro.RunMacroText('/tm [combat,mod:shift] 4; [combat] 5; 6')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('target')")
            .unwrap(),
        6.0
    );
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 3.0);
}

#[test]
fn tm_selected_unit_is_used_for_conditions_and_marker_destination() {
    let env = marker_env();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let mut focus = sim.current_target.clone().unwrap();
        focus.guid = "Creature-0-1-2-3-448-000002".into();
        focus.name = "Focus fixture".into();
        sim.current_focus = Some(focus);
    }
    env.exec("C_Macro.RunMacroText('/tm [@focus,exists] 5')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('focus')")
            .unwrap(),
        5.0
    );
    assert!(
        env.eval::<bool>("return GetRaidTargetIndex('target') == nil")
            .unwrap()
    );
    env.exec("C_Macro.RunMacroText('/tm [target=focus,exists] 6')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('focus')")
            .unwrap(),
        6.0
    );
    env.exec("C_Macro.RunMacroText('/tm [unit=focus,exists] 6')")
        .unwrap();
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('focus')")
            .unwrap(),
        6.0
    );
    env.state().borrow_mut().current_focus = None;
    env.exec("C_Macro.RunMacroText('/tm [@focus,exists] 7')")
        .unwrap();
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 3.0);
}

#[test]
fn tm_invalid_or_unselected_input_preserves_state_and_valid_command_recovers() {
    let env = marker_env();
    env.exec(
        r#"
        SetRaidTarget('target', 2)
        C_Macro.RunMacroText('/tm [unknown] 1\n/tm [combat 1\n/tm bad\n/tm 9\n/tm -1\n/tm 1.5\n/tm')
        assert(GetRaidTargetIndex('target') == 2)
        assert(markerEvents == 1)
        C_Macro.RunMacroText('/tm [exists] 7')
        assert(GetRaidTargetIndex('target') == 7)
        assert(markerEvents == 2)
    "#,
    )
    .unwrap();
    env.exec("ClearTarget(); C_Macro.RunMacroText('/tm [noexists] 3')")
        .unwrap();
    assert_eq!(env.eval::<f64>("return markerEvents").unwrap(), 2.0);
    assert_eq!(
        env.eval::<f64>("return GetRaidTargetIndex('player')")
            .unwrap(),
        7.0
    );
}
