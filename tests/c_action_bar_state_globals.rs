//! Integration tests for the action-bar transition globals registered in
//! `src/lua_api/globals/real/action_bar_state.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn override_bar_index_is_visible_when_override_state_is_active() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.has_override_action_bar = true;
        state.override_bar_index = 14;
    }

    let (has_override_bar, override_bar_index): (bool, Option<i32>) = env
        .eval(
            r#"
            return C_ActionBar.HasOverrideActionBar(), C_ActionBar.GetOverrideBarIndex()
            "#,
        )
        .unwrap();

    assert!(has_override_bar);
    assert_eq!(override_bar_index, Some(14));
}

#[test]
fn action_bar_busy_defaults_to_false() {
    let env = WowLuaEnv::new().expect("env");
    let busy: bool = env.eval("return ActionBarBusy()").unwrap();
    assert!(!busy);
}

#[test]
fn action_bar_busy_reflects_state_flag() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().action_bar_state.busy = true;
    let busy: bool = env.eval("return ActionBarBusy()").unwrap();
    assert!(busy);
}

#[test]
fn current_action_bar_state_defaults_to_main() {
    let env = WowLuaEnv::new().expect("env");
    let current: f64 = env
        .eval("return ActionBarController_GetCurrentActionBarState()")
        .unwrap();
    assert!((current - 1.0).abs() < 1e-6);
    let matches_main: bool = env
        .eval("return ActionBarController_GetCurrentActionBarState() == LE_ACTIONBAR_STATE_MAIN")
        .unwrap();
    assert!(matches_main);
}

#[test]
fn current_action_bar_state_reports_override() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().action_bar_state.current_state = 2;
    let current: f64 = env
        .eval("return ActionBarController_GetCurrentActionBarState()")
        .unwrap();
    assert!((current - 2.0).abs() < 1e-6);
    let matches_override: bool = env
        .eval(
            "return ActionBarController_GetCurrentActionBarState() == LE_ACTIONBAR_STATE_OVERRIDE",
        )
        .unwrap();
    assert!(matches_override);
}

#[test]
fn current_action_bar_state_reports_skinned_vehicle_override() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.has_vehicle_action_bar = true;
        state.player.vehicle_skin = Some("MechagonShredder".to_string());
    }

    let matches_override: bool = env
        .eval(
            "return ActionBarController_GetCurrentActionBarState() == LE_ACTIONBAR_STATE_OVERRIDE",
        )
        .unwrap();

    assert!(matches_override);
}

#[test]
fn vehicle_bar_index_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.has_vehicle_action_bar = true;
        state.vehicle_bar_index = 7;
    }

    let vehicle_bar_index: Option<i32> =
        env.eval("return C_ActionBar.GetVehicleBarIndex()").unwrap();

    assert_eq!(vehicle_bar_index, Some(7));
}

#[test]
fn special_bar_indexes_are_nil_when_inactive() {
    let env = WowLuaEnv::new().expect("env");

    let indexes: (Option<i32>, Option<i32>, Option<i32>) = env
        .eval(
            r#"
            return C_ActionBar.GetVehicleBarIndex(),
                C_ActionBar.GetOverrideBarIndex(),
                C_ActionBar.GetTempShapeshiftBarIndex()
            "#,
        )
        .unwrap();

    assert_eq!(indexes, (None, None, None));
}

#[test]
fn temp_shapeshift_bar_index_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.has_temp_shapeshift_action_bar = true;
        state.temp_shapeshift_bar_index = 9;
    }

    let (has_temp_bar, temp_bar_index): (bool, Option<i32>) = env
        .eval(
            r#"
            return C_ActionBar.HasTempShapeshiftActionBar(),
                C_ActionBar.GetTempShapeshiftBarIndex()
            "#,
        )
        .unwrap();

    assert!(has_temp_bar);
    assert_eq!(temp_bar_index, Some(9));
}

#[test]
fn bonus_bar_index_reads_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.has_bonus_action_bar = true;
        state.bonus_bar_index = 5;
    }

    let (has_bonus_bar, bonus_bar_index): (bool, i32) = env
        .eval(
            r#"
            return C_ActionBar.HasBonusActionBar(),
                C_ActionBar.GetBonusBarIndex()
            "#,
        )
        .unwrap();

    assert!(has_bonus_bar);
    assert_eq!(bonus_bar_index, 5);
}

#[test]
fn le_actionbar_state_constants_are_seeded() {
    let env = WowLuaEnv::new().expect("env");
    let main: f64 = env.eval("return LE_ACTIONBAR_STATE_MAIN").unwrap();
    let over: f64 = env.eval("return LE_ACTIONBAR_STATE_OVERRIDE").unwrap();
    assert!((main - 1.0).abs() < 1e-6);
    assert!((over - 2.0).abs() < 1e-6);
}
