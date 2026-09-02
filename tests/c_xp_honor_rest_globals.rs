//! Integration tests for the XP / honor / rest globals registered in
//! `src/lua_api/globals/real/xp_honor_rest.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn get_xp_exhaustion_returns_nil_by_default() {
    let env = WowLuaEnv::new().expect("env");
    let is_nil: bool = env.eval("return GetXPExhaustion() == nil").unwrap();
    assert!(is_nil);
}

#[test]
fn get_xp_exhaustion_returns_banked_rest_xp() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player_xp.exhaustion = Some(12_345);
    let value: f64 = env.eval("return GetXPExhaustion()").unwrap();
    assert!((value - 12_345.0).abs() < 1e-6);
}

#[test]
fn get_rest_state_defaults_to_rested() {
    let env = WowLuaEnv::new().expect("env");
    env.exec("rs, name, mult = GetRestState()").unwrap();
    let rs: f64 = env.eval("return rs").unwrap();
    let name: String = env.eval("return name").unwrap();
    let mult: f64 = env.eval("return mult").unwrap();
    assert!((rs - 1.0).abs() < 1e-6);
    assert_eq!(name, "Rested");
    assert!((mult - 1.5).abs() < 1e-6);
}

#[test]
fn get_rest_state_reflects_normal_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.player_xp.rest_state = 2;
        state.player_xp.rest_state_name = "Normal".to_string();
        state.player_xp.rest_multiplier = 1.0;
    }
    let rs: f64 = env.eval("return (GetRestState())").unwrap();
    let name: String = env.eval("local _, n = GetRestState(); return n").unwrap();
    let mult: f64 = env
        .eval("local _, _, m = GetRestState(); return m")
        .unwrap();
    assert!((rs - 2.0).abs() < 1e-6);
    assert_eq!(name, "Normal");
    assert!((mult - 1.0).abs() < 1e-6);
}

#[test]
fn is_player_at_effective_max_level_reads_flag() {
    let env = WowLuaEnv::new().expect("env");
    let before: bool = env.eval("return IsPlayerAtEffectiveMaxLevel()").unwrap();
    assert!(!before);

    env.state().borrow_mut().player_xp.is_max_level = true;
    let after: bool = env.eval("return IsPlayerAtEffectiveMaxLevel()").unwrap();
    assert!(after);
}

#[test]
fn game_limited_mode_globals_read_state() {
    let env = WowLuaEnv::new().expect("env");
    let active_default: bool = env
        .eval("return GameLimitedMode_IsBankedXPActive()")
        .unwrap();
    assert!(!active_default);
    let limit_default: f64 = env.eval("return GameLimitedMode_GetLevelLimit()").unwrap();
    assert!((limit_default - 20.0).abs() < 1e-6);

    {
        let mut state = env.state().borrow_mut();
        state.player_xp.banked_xp_active = true;
        state.player_xp.level_limit = 60;
    }
    let active: bool = env
        .eval("return GameLimitedMode_IsBankedXPActive()")
        .unwrap();
    assert!(active);
    let limit: f64 = env.eval("return GameLimitedMode_GetLevelLimit()").unwrap();
    assert!((limit - 60.0).abs() < 1e-6);
}

#[test]
fn unit_honor_returns_player_honor() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.player_xp.honor = 4_200;
        state.player_xp.honor_max = 8_500;
    }
    let honor: f64 = env.eval("return UnitHonor('player')").unwrap();
    let honor_max: f64 = env.eval("return UnitHonorMax('player')").unwrap();
    assert!((honor - 4_200.0).abs() < 1e-6);
    assert!((honor_max - 8_500.0).abs() < 1e-6);
}

#[test]
fn unit_honor_returns_zero_for_non_player_units() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player_xp.honor = 4_200;
    let honor: f64 = env.eval("return UnitHonor('target')").unwrap();
    let honor_max: f64 = env.eval("return UnitHonorMax('party1')").unwrap();
    assert!(honor.abs() < 1e-6);
    assert!(honor_max.abs() < 1e-6);
}

#[test]
fn unit_trial_xp_and_banked_levels_read_state() {
    let env = WowLuaEnv::new().expect("env");
    {
        let mut state = env.state().borrow_mut();
        state.player_xp.trial_xp = 9_999;
        state.player_xp.trial_banked_levels = 3;
    }
    let xp: f64 = env.eval("return UnitTrialXP('player')").unwrap();
    let levels: f64 = env.eval("return UnitTrialBankedLevels('player')").unwrap();
    assert!((xp - 9_999.0).abs() < 1e-6);
    assert!((levels - 3.0).abs() < 1e-6);

    let other_xp: f64 = env.eval("return UnitTrialXP('target')").unwrap();
    let other_levels: f64 = env.eval("return UnitTrialBankedLevels('target')").unwrap();
    assert!(other_xp.abs() < 1e-6);
    assert!(other_levels.abs() < 1e-6);
}

#[test]
fn get_restricted_account_data_returns_three_values() {
    let env = WowLuaEnv::new().expect("env");
    env.exec("lvl, money, prof = GetRestrictedAccountData()")
        .unwrap();
    let lvl: f64 = env.eval("return lvl").unwrap();
    let money: f64 = env.eval("return money").unwrap();
    let prof: f64 = env.eval("return prof").unwrap();
    assert!((lvl - 20.0).abs() < 1e-6);
    assert!(money.abs() < 1e-6);
    assert!(prof.abs() < 1e-6);

    {
        let mut state = env.state().borrow_mut();
        state.player_xp.restricted_level = 60;
        state.player_xp.restricted_money = 250_000;
        state.player_xp.restricted_profession = 100;
    }
    env.exec("lvl, money, prof = GetRestrictedAccountData()")
        .unwrap();
    let lvl: f64 = env.eval("return lvl").unwrap();
    let money: f64 = env.eval("return money").unwrap();
    let prof: f64 = env.eval("return prof").unwrap();
    assert!((lvl - 60.0).abs() < 1e-6);
    assert!((money - 250_000.0).abs() < 1e-6);
    assert!((prof - 100.0).abs() < 1e-6);
}

#[test]
fn get_xp_exhaustion_returns_nil_after_clearing() {
    let env = WowLuaEnv::new().expect("env");
    env.state().borrow_mut().player_xp.exhaustion = Some(12_345);
    let banked_xp: f64 = env.eval("return GetXPExhaustion()").unwrap();
    assert_eq!(banked_xp, 12_345.0);

    env.state().borrow_mut().player_xp.exhaustion = None;
    let is_nil: bool = env.eval("return GetXPExhaustion() == nil").unwrap();
    assert!(is_nil);
}
