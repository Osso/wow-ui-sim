//! WM93 bounded PTR 70668 contract applied to pinned PTR 70493 UI inputs.
#[cfg(any(feature = "client-ptr", feature = "client-retail"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
fn supplied_result(env: &WowLuaEnv, key: i32, in_range: bool, available: bool) {
    env.state()
        .borrow_mut()
        .action_combat_range
        .supplied_results
        .insert(
            key,
            wow_ui_sim::c_api::action_combat_range_check::SuppliedCombatRangeResult {
                in_range,
                available,
            },
        );
}

#[cfg(feature = "client-ptr")]
fn listen(env: &WowLuaEnv) {
    env.exec(r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        CombatRangeEvents = {}
        local f = CreateFrame('Frame')
        f:RegisterEvent('ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE')
        f:SetScript('OnEvent', function(_, event, ...)
            assert(event == 'ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE')
            assert(select('#', ...) == 3)
            local slot, inRange, available = ...
            assert(type(slot) == 'number' and type(inRange) == 'boolean' and type(available) == 'boolean')
            table.insert(CombatRangeEvents, {slot, inRange, available})
        end)
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_combat_range_counts_truthiness_and_hover_independence() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_range_visualization.stored_slot = Some(4);
        sim.spell_range_visualization.value = 71;
        sim.spell_range_visualization.enabled = true;
    }
    let hover = env.state().borrow().spell_range_visualization.clone();
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        local enable = C_ActionBar.EnableActionCombatActionRangeCheck
        for _, value in ipairs({true, 0, '', {}, function() end}) do
            assert(select('#', enable('5.75', value, 'ignored')) == 0)
        end
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().action_combat_range.registrations[&4].references,
        5
    );
    env.exec("C_ActionBar.EnableActionCombatActionRangeCheck(5, false)")
        .unwrap();
    assert_eq!(
        env.state().borrow().action_combat_range.registrations[&4].references,
        4
    );
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', false)
        C_ActionBar.EnableActionCombatActionRangeCheck(5, false)
        C_ActionBar.EnableActionCombatActionRangeCheck(6, true)
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().action_combat_range.registrations[&4].references,
        4
    );
    assert!(
        !env.state()
            .borrow()
            .action_combat_range
            .registrations
            .contains_key(&5)
    );
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        for i = 1, 4 do C_ActionBar.EnableActionCombatActionRangeCheck(5, false) end
    "#,
    )
    .unwrap();
    assert!(
        !env.state()
            .borrow()
            .action_combat_range
            .registrations
            .contains_key(&4)
    );
    assert_eq!(env.state().borrow().spell_range_visualization, hover);
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_combat_range_parser_required_enable_and_noncanonical_slots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        local query = rawget(C_ActionBar, 'EnableActionCombatActionRangeCheck')
        assert(type(query) == 'function')
        local usage = 'Usage: C_ActionBar.EnableActionCombatActionRangeCheck(actionID, enable)'
        for _, call in ipairs({function() query() end, function() query(nil, true) end,
                               function() query(5) end, function() query(5, nil) end}) do
            local ok, err = pcall(call)
            assert(not ok and err:find(usage, 1, true))
        end
        local ok, err = pcall(query, 5, nil)
        assert(not ok and err:find('bad argument #2', 1, true))
        for _, value in ipairs({-1, math.huge, 4294967296, 'invalid', false, {}}) do
            ok, err = pcall(query, value, true)
            assert(not ok and err:find('outside of expected range 0 to 4294967295', 1, true))
            assert(err:find('bad argument #1', 1, true))
        end
        for _, value in ipairs({0, '409', 0.25, 408.9}) do
            assert(select('#', query(value, true)) == 0)
        end
    "#,
    )
    .unwrap();
    let sim = env.state().borrow();
    for key in [-1, 408, 0, 407] {
        assert_eq!(sim.action_combat_range.registrations[&key].references, 1);
    }
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_combat_range_secret_access_errors_are_not_parser_errors() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        A_Admin.SetGameRule('ActionCombatSpellHitIndicationEnabled', true)
        local query = C_ActionBar.EnableActionCombatActionRangeCheck
        local slot, enable = secretwrap(5), secretwrap(true)
        assert(select('#', query(slot, enable)) == 0, 'EMULATED AllowedWhenUntainted')
        local function probe()
            for _, args in ipairs({{slot, true}, {5, enable}}) do
                local ok, err = pcall(query, unpack(args))
                assert(not ok and err:find('Secret values', 1, true))
                assert(not err:find('outside of expected range', 1, true))
            end
            assert(debug.getstacktaint() == 'WM93Probe')
        end
        debug.setobjecttaint(probe, 'WM93Probe')
        probe()
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state().borrow().action_combat_range.registrations[&4].references,
        1
    );
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_combat_range_monitor_initial_change_and_balanced_cleanup_events() {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    supplied_result(&env, 4, true, true);
    env.exec(r#"
        local query = C_ActionBar.EnableActionCombatActionRangeCheck
        query(5, true); query(5, true)
        assert(#CombatRangeEvents == 0, 'registration must not notify')
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 1)
        assert(CombatRangeEvents[1][1] == 5 and CombatRangeEvents[1][2] == true and CombatRangeEvents[1][3] == true)
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 1, 'unchanged initialized result')
    "#).unwrap();
    supplied_result(&env, 4, false, true);
    env.exec(
        r#"
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 2)
        assert(CombatRangeEvents[2][2] == false and CombatRangeEvents[2][3] == true)
    "#,
    )
    .unwrap();
    supplied_result(&env, 4, true, false);
    env.exec(r#"
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 3)
        assert(CombatRangeEvents[3][2] == false and CombatRangeEvents[3][3] == false, 'unavailable retains cached range')
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 3)
        C_ActionBar.EnableActionCombatActionRangeCheck(5, false)
        assert(#CombatRangeEvents == 3, 'partial disable is silent')
    "#).unwrap();
    assert_eq!(
        env.state().borrow().action_combat_range.registrations[&4].references,
        1
    );
    supplied_result(&env, 4, true, true);
    env.exec(r#"
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 4 and CombatRangeEvents[4][2] == true and CombatRangeEvents[4][3] == true)
        C_ActionBar.EnableActionCombatActionRangeCheck(5, false)
        assert(#CombatRangeEvents == 4, 'final disable has no final notification')
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 4)
        C_ActionBar.EnableActionCombatActionRangeCheck(5, true)
        assert(#CombatRangeEvents == 4)
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 5, 'balanced removal discarded cached initialization')
        A_Admin.SweepActionCombatRangeChecks(true)
        assert(#CombatRangeEvents == 6, 'explicit force input')
    "#).unwrap();
}

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_tooltip_combat_range_monitor_requires_supplied_results_and_reports_invalid_slots_unavailable()
 {
    let env = WowLuaEnv::new().unwrap();
    listen(&env);
    env.exec(
        r#"
        C_ActionBar.EnableActionCombatActionRangeCheck(0, true)
        C_ActionBar.EnableActionCombatActionRangeCheck(409, true)
        A_Admin.SweepActionCombatRangeChecks()
        assert(#CombatRangeEvents == 0, 'no fabricated range results')
    "#,
    )
    .unwrap();
    for (key, public_slot) in [(-1, 0), (408, 409)] {
        supplied_result(&env, key, true, true);
        env.exec("CombatRangeEvents = {}; A_Admin.SweepActionCombatRangeChecks()")
            .unwrap();
        env.exec(&format!(
            r#"
            assert(#CombatRangeEvents == 1)
            assert(CombatRangeEvents[1][1] == {public_slot})
            assert(CombatRangeEvents[1][3] == false, 'converted slot outside 0..407 is unavailable')
        "#
        ))
        .unwrap();
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn tooltip_combat_range_api_and_event_are_absent_on_retail() {
    let env = WowLuaEnv::new().unwrap();
    assert!(!wow_ui_sim::event::is_registerable_event(
        "ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE"
    ));
    assert!(!wow_ui_sim::event::is_valid_event(
        "ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE"
    ));
    env.exec(
        r#"
        assert(rawget(C_ActionBar, 'EnableActionCombatActionRangeCheck') == nil)
        assert(C_ActionBar.EnableActionCombatActionRangeCheck == nil)
        local frame = CreateFrame('Frame')
        assert(not pcall(frame.RegisterEvent, frame, 'ACTION_COMBAT_ACTION_RANGE_CHECK_UPDATE'))
    "#,
    )
    .unwrap();
}
