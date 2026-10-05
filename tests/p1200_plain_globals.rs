#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1200_removed_plain_globals_are_not_simulator_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({
            'CombatLogAddFilter', 'CombatLogAdvanceEntry', 'CombatLogClearEntries',
            'CombatLogGetCurrentEntry', 'CombatLogGetCurrentEventInfo',
            'CombatLogGetNumEntries', 'CombatLogGetRetentionTime', 'CombatLogResetFilter',
            'CombatLogSetCurrentEntry', 'CombatLogSetRetentionTime', 'CombatLogShowCurrentEntry',
            'CombatLog_Object_IsA', 'CombatTextSetActiveUnit', 'DeathRecap_GetEvents',
            'DeathRecap_HasEvents', 'GetBattlegroundInfo', 'GetCurrentCombatTextEventInfo',
            'GetDeathRecapLink', 'SetPortraitToTexture'
        }) do
            assert(rawget(_G, name) == nil, name .. ' must not be simulator-published')
        end
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_cloak_helm_transitions_are_independent_and_reversible() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local cloak, helm = ShowingCloak(), ShowingHelm()
        ShowCloak(false); ShowHelm(true)
        assert(not ShowingCloak() and ShowingHelm())
        ShowCloak(true); ShowHelm(false)
        assert(ShowingCloak() and not ShowingHelm())
        ShowCloak(cloak); ShowHelm(helm)
        assert(ShowingCloak() == cloak and ShowingHelm() == helm)
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_player_queries_read_host_inputs() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.plain_global_inputs.collapsing_star_cost = 125.5;
        sim.plain_global_inputs.raid_marker_system_enabled = false;
    }
    assert_eq!(
        env.eval::<(f64, bool)>("return GetCollapsingStarCost(), IsRaidMarkerSystemEnabled()")
            .unwrap(),
        (125.5, false)
    );
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .raid_marker_system_enabled = true;
    assert!(
        env.eval::<bool>("return IsRaidMarkerSystemEnabled()")
            .unwrap()
    );
}

#[test]
fn p1200_secret_helpers_use_vm_wrappers_and_calling_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(canaccesssecrets())
        local secret = secretwrap(42)
        assert(not hasanysecretvalues())
        assert(not hasanysecretvalues(nil, 42, 'public', {secret}))
        assert(hasanysecretvalues(nil, secret, 'public'))
        local f = function() return canaccesssecrets(), hasanysecretvalues(secret) end
        debug.setobjecttaint(f, 'PlainGlobalAddon')
        local access, contains = f()
        assert(not access and contains)
        assert(canaccesssecrets())
    "#,
    )
    .unwrap();
}

#[test]
fn p1200_unit_roles_and_threat_use_resolved_guids() {
    let env = WowLuaEnv::new().unwrap();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.plain_global_inputs
            .lieutenant_guids
            .insert(guid.clone());
        sim.plain_global_inputs.minion_guids.insert(guid.clone());
        sim.plain_global_inputs
            .npc_as_player_guids
            .insert(guid.clone());
        sim.plain_global_inputs.threat_lead.insert(
            (guid.clone(), guid.clone()),
            wow_ui_sim::lua_api::globals::real::publication_12_0_0::ThreatLeadSnapshot {
                is_first: true,
                lead_state: 1,
            },
        );
    }
    env.exec("assert(UnitIsLieutenant('player')); assert(UnitIsMinion('player')); assert(UnitIsNPCAsPlayer('player')); assert(not UnitIsNPCAsPlayer(nil)); assert(not UnitIsLieutenant('nonexistent')); assert(UnitThreatLeadSituation('player', 'player') == 1); assert(UnitThreatLeadSituation('player', 'nonexistent') == nil)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.plain_global_inputs
            .threat_lead
            .get_mut(&(guid.clone(), guid))
            .unwrap()
            .is_first = false;
        sim.plain_global_inputs.lieutenant_guids.clear();
    }
    env.exec("assert(not UnitIsLieutenant('player')); assert(UnitThreatLeadSituation('player', 'player') == 3)").unwrap();
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .threat_state_restricted = true;
    env.exec("assert(issecretvalue(UnitThreatLeadSituation('player','player')))")
        .unwrap();
}

#[test]
fn p1200_unit_cast_class_and_stage_durations_are_snapshot_backed() {
    use wow_ui_sim::lua_api::state::{CastTargetSnapshot, CastingState, EmpowerTiming};
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(not UnitShouldDisplaySpellTargetName('player')); assert(UnitSpellTargetClass('player') == nil); assert(select('#', UnitEmpoweredStageDurations('player')) == 0)").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.plain_global_inputs
            .cast_target_classes
            .insert("Player-1-123".into(), "PRIEST".into());
        sim.casting = Some(CastingState {
            spell_id: 19750,
            spell_name: "Flash of Light".into(),
            icon_path: String::new(),
            start_time: 100.0,
            end_time: 105.0,
            cast_id: 1,
            delay_time: 0.0,
            empower: None,
            target: Some(CastTargetSnapshot {
                guid: "Player-1-123".into(),
                name: "Recipient".into(),
                is_player: true,
            }),
        });
        sim.channeling = Some(CastingState {
            spell_id: 361469,
            spell_name: "Living Flame".into(),
            icon_path: String::new(),
            start_time: 100.0,
            end_time: 105.0,
            cast_id: 2,
            delay_time: 0.0,
            target: None,
            empower: Some(EmpowerTiming {
                stage_durations: vec![2.0, 3.0],
                hold_at_max: 1.5,
            }),
        });
    }
    env.exec(
        r#"
        assert(UnitShouldDisplaySpellTargetName('player'))
        local class = UnitSpellTargetClass('player')
        assert(issecretvalue(class) and secretunwrap(class) == 'PRIEST')
        local stages = UnitEmpoweredStageDurations('player')
        assert(#stages == 3)
        assert(stages[1]:GetStartTime() == 100 and stages[1]:GetTotalDuration() == 2)
        assert(stages[2]:GetStartTime() == 102 and stages[2]:GetTotalDuration() == 3)
        assert(stages[3]:GetStartTime() == 105 and stages[3]:GetTotalDuration() == 1.5)
        assert(select('#', UnitEmpoweredStageDurations('target')) == 0)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().casting = None;
    env.exec("assert(not UnitShouldDisplaySpellTargetName('player')); assert(UnitSpellTargetClass('player') == nil)").unwrap();
}

#[test]
fn p1200_cursor_position_consumes_one_gamepad_grant_for_addons() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        SetCursorPosition(120, 230)
        local x,y = GetCursorPosition()
        assert(x == 120 and y == 230)
        CursorAddon = function(x,y) SetCursorPosition(x,y) end
        debug.setobjecttaint(CursorAddon, 'GamepadAddon')
        CursorAddon(10,20)
        x,y = GetCursorPosition()
        assert(x == 120 and y == 230)
    "#,
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .plain_global_inputs
        .gamepad_cursor_input_available = true;
    env.exec(
        r#"
        CursorAddon(10,20)
        CursorAddon(30,40)
        local x,y = GetCursorPosition()
        assert(x == 10 and y == 20)
    "#,
    )
    .unwrap();
    assert!(
        !env.state()
            .borrow()
            .plain_global_inputs
            .gamepad_cursor_input_available
    );
}
