//! Target-bound Forever combo points; native lifecycle choices remain simulator guesses.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn env_with_two_targets() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec("TargetUnit('enemy1')").unwrap();
    {
        let mut state = env.state().borrow_mut();
        let mut first = state.current_target.clone().unwrap();
        first.guid = "Creature-0-0-0-0-448-000101".into();
        let mut second = first.clone();
        second.name = "Second enemy".into();
        second.guid = "Creature-0-0-0-0-448-000102".into();
        state.current_target = Some(first.clone());
        state.enemy_pool = vec![first, second];
    }
    env
}

#[test]
fn combo_points_read_updates_and_zero_from_the_player_power_input() {
    let env = env_with_two_targets();
    env.exec(
        r#"
        assert(select('#', GetComboPoints('player', 'target')) == 1)
        assert(GetComboPoints('player', 'target') == 0)
        for _, count in ipairs({3, 5, 1, 0}) do
            A_Admin.SetPlayerPower(count, 5, Enum.PowerType.ComboPoints)
            assert(GetComboPoints('player', 'target') == count)
            assert(UnitPower('player', Enum.PowerType.ComboPoints) == count)
        end
        A_Admin.SetPlayerPower(4, 5, Enum.PowerType.ComboPoints)
        A_Admin.SetPlayerPower(73, 100, Enum.PowerType.Energy)
        assert(GetComboPoints('player', 'target') == 4)
        assert(UnitPower('player') == 73)
        assert(GetComboPoints('absent', 'target') == 0)
        assert(GetComboPoints('player', 'absent') == 0)
        assert(not pcall(GetComboPoints))
        assert(not pcall(GetComboPoints, 'player'))
        "#,
    )
    .expect("the legacy result follows the existing combo power pool, not primary energy");
}

#[test]
fn combo_points_target_change_does_not_reuse_the_stale_unit_power_snapshot() {
    let env = env_with_two_targets();
    env.exec(
        r#"
        A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
        local seen = {}
        local frame = CreateFrame('Frame')
        frame:RegisterEvent('PLAYER_TARGET_CHANGED')
        frame:SetScript('OnEvent', function()
            seen[#seen + 1] = {
                GetComboPoints('player', 'target'),
                UnitPower('player', Enum.PowerType.ComboPoints),
            }
        end)
        TargetUnit('enemy2')
        assert(#seen == 1)
        assert(seen[1][1] == 0 and seen[1][2] == 3,
            'target-change callback must not display the previous target count')
        assert(GetComboPoints('player', 'enemy1') == 3)
        A_Admin.SetPlayerPower(2, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 2)
        assert(GetComboPoints('player', 'enemy1') == 0,
            'a new power input replaces the single target assignment')
        ClearTarget()
        assert(GetComboPoints('player', 'target') == 0)
        assert(UnitPower('player', Enum.PowerType.ComboPoints) == 2)
        "#,
    )
    .expect("target identity changes before the next power update");
}

#[test]
fn combo_points_compare_target_guids_and_keep_queries_read_only() {
    let env = env_with_two_targets();
    env.exec(
        r#"
        A_Admin.SetPlayerPower(4, 5, Enum.PowerType.ComboPoints)
        FocusUnit('target')
        TargetUnit('enemy2')
        assert(GetComboPoints('player', 'focus') == 4)
        assert(GetComboPoints('player', 'target') == 0)
        TargetUnit('enemy1')
        assert(GetComboPoints('self', 'target') == 4)
        FocusUnit('player')
        assert(GetComboPoints('focus', 'target') == 4)
        A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 0)
        "#,
    )
    .expect("aliases share a GUID and changing selection does not mutate point storage");
}

#[test]
fn combo_points_without_a_target_do_not_attach_to_a_later_selection() {
    let env = env_with_two_targets();
    env.exec(
        r#"
        ClearTarget()
        A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 0)
        TargetUnit('enemy1')
        assert(GetComboPoints('player', 'target') == 0)
        assert(UnitPower('player', Enum.PowerType.ComboPoints) == 3)
        A_Admin.SetPlayerPower(2, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 2)
        "#,
    )
    .expect("an unassigned snapshot is not implicitly credited to a new target");
}

#[test]
fn combo_points_report_unmodeled_nonplayer_ownership_instead_of_faking_a_count() {
    let env = env_with_two_targets();
    env.exec(
        r#"
        assert(GetComboPoints('target', 'absent') == 0)
        local ok, message = pcall(GetComboPoints, 'target', 'player')
        assert(not ok)
        assert(string.find(message, 'player owner', 1, true), message)
        "#,
    )
    .expect("other resolved units have no modeled combo-point ownership");
}

#[test]
fn combo_points_native_frame_updates_with_the_cvar_enabled() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    env.exec("SetCVar('comboPointLocation', '1')").unwrap();
    crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_AuraContainer", "Blizzard_UnitFrame"],
        &[],
    );
    env.exec(
        r#"
        assert(ComboFrame and ComboFrame.maxComboPoints)
        assert(ComboFrame:IsEventRegistered('PLAYER_TARGET_CHANGED'))
        TargetUnit('enemy1')
        A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
        ComboFrame_Update(ComboFrame)
        assert(ComboFrame:IsShown())
        assert(COMBO_FRAME_LAST_NUM_POINTS == 3)
        A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)
        ComboFrame_Update(ComboFrame)
        assert(not ComboFrame:IsShown())
        assert(COMBO_FRAME_LAST_NUM_POINTS == 0)
        "#,
    )
    .expect("unchanged native ComboFrame consumes zero and nonzero modeled values");
    assert!(env.state().borrow().lua_errors.is_empty());
}
