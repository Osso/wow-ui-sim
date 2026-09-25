//! Retail movement and player-owned combo-power queries.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn retail_speed_reads_movement_capabilities_and_player_aliases() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local current, run, flight, swim = GetUnitSpeed('player')
        assert(current == 0 and run == 7 and flight == 7 and swim == 4.722222)
        MoveForwardStart()
        assert(select(1, GetUnitSpeed('self')) == 7)
        A_Admin.SetFlying(true)
        assert(select(1, GetUnitSpeed('player')) == 7)
        A_Admin.SetSwimming(true)
        assert(select(1, GetUnitSpeed('player')) == 4.722222)
        A_Admin.SetSwimming(false)
        assert(select(1, GetUnitSpeed('player')) == 7)
        MoveForwardStop()
        assert(select(1, GetUnitSpeed('player')) == 0)
        TargetUnit('player')
        MoveForwardStart()
        assert(select(1, GetUnitSpeed('target')) == 7)
        A_Admin.SetTarget('Other', 70, 2, false)
        current, run, flight, swim = GetUnitSpeed('target')
        assert(current == 0 and run == 0 and flight == 0 and swim == 0)
        assert(select(1, GetUnitSpeed('unknown')) == 0)
        assert(not pcall(GetUnitSpeed))
        "#,
    )
    .unwrap();
}

#[test]
fn retail_speed_uses_configured_capabilities() {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        state.player.movement_speeds.run = 9.0;
        state.player.movement_speeds.flight = 15.0;
        state.player.movement_speeds.swim = 5.0;
    }
    env.exec(
        r#"
        local current, run, flight, swim = GetUnitSpeed('player')
        assert(current == 0 and run == 9 and flight == 15 and swim == 5)
        A_Admin.SetMoving(true)
        assert(select(1, GetUnitSpeed('player')) == 9)
        A_Admin.SetFlying(true)
        assert(select(1, GetUnitSpeed('player')) == 15)
        A_Admin.SetSwimming(true)
        assert(select(1, GetUnitSpeed('player')) == 5)
        "#,
    )
    .unwrap();
}

#[test]
fn retail_combo_points_follow_player_power_across_target_changes() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        TargetUnit('enemy1')
        assert(select('#', GetComboPoints('player', 'target')) == 1)
        assert(GetComboPoints('player', 'target') == 0)
        A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 3)
        TargetUnit('enemy2')
        assert(GetComboPoints('player', 'target') == 3)
        ClearTarget()
        assert(GetComboPoints('player', 'target') == 3)
        assert(GetComboPoints('self', 'unknown') == 3)
        assert(GetComboPoints('unknown', 'target') == 0)
        A_Admin.SetPlayerPower(72, 100, Enum.PowerType.Energy)
        assert(GetComboPoints('player', 'target') == 3)
        A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)
        assert(GetComboPoints('player', 'target') == 0)
        assert(not pcall(GetComboPoints))
        assert(not pcall(GetComboPoints, 'player'))
        "#,
    )
    .unwrap();
}

#[test]
fn retail_native_aura_button_preserves_wrapped_duration_arguments() {
    let ui = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui);
    crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_AuraContainer"],
        &[],
    );
    env.exec(
        r#"
        assert(type(AuraButtonPrivateMixin.UpdateAuraDuration) == 'function')
        local duration = C_DurationUtil.CreateDuration()
        local button = { auraDuration = duration,
            auraData = { expirationTime = 50, duration = 30, timeMod = 2 } }
        AuraButtonPrivateMixin.UpdateAuraDuration(button)
        assert(duration:HasSecretValues())
        assert(duration:GetEndTime() == 50 and duration:GetTotalDuration() == 15)
        assert(duration:GetModRate() == 2)
        local initial = duration
        button.auraData = { expirationTime = 0, duration = 30 }
        AuraButtonPrivateMixin.UpdateAuraDuration(button)
        assert(button.auraDuration == initial and duration:HasSecretValues())
        assert(duration:IsZero())
        local function values(...) return select('#', ...), ... end
        local count, first, second, third = values(secretwrap(8, nil, 3))
        assert(count == 3 and issecretvalue(first) and issecretvalue(second)
            and issecretvalue(third))
        assert(secretunwrap(first, second, third) == 8)
        assert(select('#', secretunwrap(first, second, third)) == 3)
        assert(select(2, secretunwrap(first, second, third)) == nil)
        assert(select(3, secretunwrap(first, second, third)) == 3)
        assert(not canaccessvalue(first) and not canaccessallvalues(1, first))
        local function tainted()
            assert(not issecure())
            assert(issecretvalue(first) and not canaccessvalue(first))
            assert(not pcall(secretwrap, 1))
            assert(not pcall(secretunwrap, first))
            assert(not pcall(duration.GetEndTime, duration))
            assert(not pcall(duration.SetTimeFromEnd, duration, first, 2, 1))
        end
        debug.setobjecttaint(tainted, 'RetailAuraDurationProbe')
        tainted()
        assert(duration:IsZero() and duration:HasSecretValues())
        assert(settablesecurity == nil)
        "#,
    )
    .expect("unchanged Blizzard aura duration consumer preserves wrapped timing and identity");
}

#[test]
fn retail_native_combo_frame_world_entry_updates_nonzero_then_zero() {
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
        assert(ComboFrame:IsEventRegistered('PLAYER_ENTERING_WORLD'))
        TargetUnit('enemy1')
        A_Admin.SetPlayerPower(3, 5, Enum.PowerType.ComboPoints)
        "#,
    )
    .unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(ComboFrame:IsShown()); assert(COMBO_FRAME_LAST_NUM_POINTS == 3)")
        .unwrap();
    env.exec("A_Admin.SetPlayerPower(0, 5, Enum.PowerType.ComboPoints)")
        .unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(not ComboFrame:IsShown()); assert(COMBO_FRAME_LAST_NUM_POINTS == 0)")
        .unwrap();
    assert!(env.state().borrow().lua_errors.is_empty());
}
