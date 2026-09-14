//! Tests for A_Admin health and power API.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// SetPlayerHealth
// ============================================================================

#[test]
fn test_set_player_health_current() {
    let env = env();
    let hp: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(5000, 10000)
            return UnitHealth("player")
            "#,
        )
        .unwrap();
    assert_eq!(hp, 5000);
}

#[test]
fn test_set_player_health_max() {
    let env = env();
    let hp_max: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(5000, 10000)
            return UnitHealthMax("player")
            "#,
        )
        .unwrap();
    assert_eq!(hp_max, 10000);
}

#[test]
fn test_set_player_health_both_values() {
    let env = env();
    let (cur, max): (i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(75000, 200000)
            return UnitHealth("player"), UnitHealthMax("player")
            "#,
        )
        .unwrap();
    assert_eq!(cur, 75000);
    assert_eq!(max, 200000);
}

#[test]
fn unit_health_percent_uses_player_health_values() {
    let env = env();
    let percent: f64 = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(5000, 20000)
            return UnitHealthPercent("player")
            "#,
        )
        .unwrap();
    assert_eq!(percent, 25.0);
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn unit_health_percent_ignores_legacy_truthy_curve_argument() {
    let env = env();
    let percent: f64 = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(12345, 100000)
            return UnitHealthPercent("player", true, true)
            "#,
        )
        .unwrap();
    assert_eq!(percent, 12.345);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_percent_rejects_invalid_curves() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetPlayerHealth(12345, 100000)
        for _, curve in ipairs({true, false, 17, {}}) do
            local ok, message = pcall(UnitHealthPercent, 'player', false, curve)
            assert(not ok, 'invalid curve must fail')
            assert(string.find(message, 'expected LuaCurveObjectBase', 1, true))
        end
        "#,
    )
    .expect("invalid supplied curves are rejected");
}

#[test]
fn unit_detailed_heal_prediction_populates_calculator() {
    let env = env();
    let (
        return_count,
        health,
        health_max,
        damage_absorbs,
        heal_absorbs,
        heals,
        healer_heals,
        incoming_amount,
        incoming_from_healer,
        incoming_from_others,
        incoming_clamped,
        current_health,
        maximum_health,
        has_secret_values,
    ): (i32, i32, i32, i32, i32, i32, i32, i32, i32, i32, bool, i32, i32, bool) = env
        .eval(
            r##"
            A_Admin.SetPlayerHealth(75000, 200000)
            local calculator = CreateUnitHealPredictionCalculator()
            local returnCount = select("#", UnitGetDetailedHealPrediction("player", nil, calculator))
            local predicted = calculator:GetPredictedValues()
            local amount, fromHealer, fromOthers, clamped = calculator:GetIncomingHeals()
            return returnCount,
                   predicted.health,
                   predicted.healthMax,
                   predicted.totalDamageAbsorbs,
                   predicted.totalHealAbsorbs,
                   predicted.totalIncomingHeals,
                   predicted.totalIncomingHealsFromHealer,
                   amount,
                   fromHealer,
                   fromOthers,
                   clamped,
                   calculator:GetCurrentHealth(),
                   calculator:GetMaximumHealth(),
                   calculator:HasSecretValues()
        "##,
        )
        .unwrap();
    assert_eq!(return_count, 0);
    assert_eq!(health, 75000);
    assert_eq!(health_max, 200000);
    assert_eq!(damage_absorbs, 0);
    assert_eq!(heal_absorbs, 0);
    assert_eq!(heals, 0);
    assert_eq!(healer_heals, 0);
    assert_eq!(incoming_amount, 0);
    assert_eq!(incoming_from_healer, 0);
    assert_eq!(incoming_from_others, 0);
    assert!(!incoming_clamped);
    assert_eq!(current_health, 75000);
    assert_eq!(maximum_health, 200000);
    assert!(!has_secret_values);
}

#[test]
fn test_set_player_health_full() {
    let env = env();
    let (cur, max): (i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerHealth(100000, 100000)
            return UnitHealth("player"), UnitHealthMax("player")
            "#,
        )
        .unwrap();
    assert_eq!(cur, 100000);
    assert_eq!(max, 100000);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_missing_player_tracks_health_without_mutation() {
    env()
        .eval::<()>(
            r##"
            local function check(current, maximum, expected)
                A_Admin.SetPlayerHealth(current, maximum)
                assert(UnitHealth("player") == current)
                assert(UnitHealthMax("player") == maximum)
                local missing = UnitHealthMissing("player", false)
                assert(type(missing) == "number", "missing health must be numeric")
                assert(missing == expected, "unexpected player missing health")
                assert(select("#", UnitHealthMissing("player", false)) == 1)
                assert(UnitHealth("player") == current, "query changed current health")
                assert(UnitHealthMax("player") == maximum, "query changed maximum health")
            end
            check(10000, 10000, 0)
            check(3500, 10000, 6500)
            check(7200, 12000, 4800)
            "##,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_missing_target_tracks_health_without_mutation() {
    env()
        .eval::<()>(
            r##"
            A_Admin.SetTarget("Boss", 63, 1, true)
            local function check(current, maximum, expected)
                A_Admin.SetTargetHealth(current, maximum)
                assert(UnitHealth("target") == current)
                assert(UnitHealthMax("target") == maximum)
                local missing = UnitHealthMissing("target", false)
                assert(type(missing) == "number", "missing health must be numeric")
                assert(missing == expected, "unexpected target missing health")
                assert(select("#", UnitHealthMissing("target", false)) == 1)
                assert(UnitHealth("target") == current, "query changed current health")
                assert(UnitHealthMax("target") == maximum, "query changed maximum health")
            end
            check(80000, 80000, 0)
            check(27500, 80000, 52500)
            check(61000, 95000, 34000)
            "##,
        )
        .unwrap();
}

// ============================================================================
// SetPlayerPower
// ============================================================================

#[test]
fn test_set_player_power_current() {
    let env = env();
    let power: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(500, 1000, 0)
            return UnitPower("player", 0)
            "#,
        )
        .unwrap();
    assert_eq!(power, 500);
}

#[test]
fn unit_power_bar_id_defaults_to_zero() {
    let env = env();
    let power_bar_id: i32 = env.eval(r#"return UnitPowerBarID("player")"#).unwrap();
    assert_eq!(power_bar_id, 0);
}

#[test]
fn test_set_player_power_max() {
    let env = env();
    let power_max: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(500, 1000, 0)
            return UnitPowerMax("player", 0)
            "#,
        )
        .unwrap();
    assert_eq!(power_max, 1000);
}

#[test]
fn test_set_player_power_both_values() {
    let env = env();
    let (cur, max): (i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(30000, 80000, 0)
            return UnitPower("player"), UnitPowerMax("player")
            "#,
        )
        .unwrap();
    assert_eq!(cur, 30000);
    assert_eq!(max, 80000);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_power_missing_player_primary_tracks_pool_without_mutation() {
    env()
        .eval::<()>(
            r##"
            local function check(current, maximum, expected)
                A_Admin.SetPlayerPower(current, maximum, 0)
                assert(UnitPower("player") == current)
                assert(UnitPowerMax("player") == maximum)
                local missing = UnitPowerMissing("player", nil, false)
                assert(type(missing) == "number")
                assert(missing == expected, "unexpected player missing power")
                assert(select("#", UnitPowerMissing("player", nil, false)) == 1)
                assert(UnitPowerMissing("player", 0, false) == expected)
                assert(UnitPower("player") == current, "query changed current power")
                assert(UnitPowerMax("player") == maximum, "query changed maximum power")
            end
            check(3000, 8000, 5000)
            check(6000, 8000, 2000)
            check(6000, 10000, 4000)
            check(10000, 10000, 0)
            "##,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_power_missing_target_primary_tracks_pool_without_mutation() {
    env()
        .eval::<()>(
            r##"
            A_Admin.SetTarget("Power Dummy", 63, 1, true)
            local function check(current, maximum, expected)
                A_Admin.SetTargetPower(current, maximum, 0)
                assert(UnitPower("target") == current)
                assert(UnitPowerMax("target") == maximum)
                local missing = UnitPowerMissing("target", nil, false)
                assert(type(missing) == "number")
                assert(missing == expected, "unexpected target missing power")
                assert(select("#", UnitPowerMissing("target", nil, false)) == 1)
                assert(UnitPowerMissing("target", 0, false) == expected)
                assert(UnitPower("target") == current, "query changed current power")
                assert(UnitPowerMax("target") == maximum, "query changed maximum power")
            end
            check(1200, 5000, 3800)
            check(3500, 5000, 1500)
            check(3500, 7000, 3500)
            check(7000, 7000, 0)
            "##,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_power_missing_player_secondary_preserves_primary_pool() {
    env()
        .eval::<()>(
            r##"
            A_Admin.SetPlayerPower(42000, 90000, 0)
            local function check(current, maximum, expected)
                A_Admin.SetPlayerPower(current, maximum, 9)
                assert(UnitPower("player", 9) == current)
                assert(UnitPowerMax("player", 9) == maximum)
                local missing = UnitPowerMissing("player", 9, false)
                assert(type(missing) == "number")
                assert(missing == expected, "unexpected secondary missing power")
                assert(select("#", UnitPowerMissing("player", 9, false)) == 1)
                assert(UnitPower("player", 9) == current, "query changed secondary power")
                assert(UnitPowerMax("player", 9) == maximum, "query changed secondary maximum")
                assert(UnitPower("player") == 42000, "query changed primary power")
                assert(UnitPowerMax("player") == 90000, "query changed primary maximum")
                assert(UnitPowerType("player") == 0, "query changed primary power type")
            end
            check(2, 5, 3)
            check(4, 5, 1)
            check(4, 6, 2)
            check(6, 6, 0)
            "##,
        )
        .unwrap();
}

#[test]
fn unit_power_percent_uses_current_over_max_power() {
    let env = env();
    let (default_percent, updated_percent): (f64, f64) = env
        .eval(
            r#"
            local defaultPercent = UnitPowerPercent("player", 0, true, 1)
            A_Admin.SetPlayerPower(300, 1000, 0)
            return defaultPercent, UnitPowerPercent("player", 0, true, 1)
            "#,
        )
        .unwrap();
    assert_eq!(default_percent, 50.0);
    assert_eq!(updated_percent, 30.0);
}

#[test]
fn test_set_player_power_type_changes() {
    let env = env();
    let power_type: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(100, 100, 1)
            local pt, name = UnitPowerType("player")
            return pt
            "#,
        )
        .unwrap();
    assert_eq!(power_type, 1);
}

#[test]
fn test_set_player_power_without_type_keeps_existing() {
    let env = env();
    // SetPlayerPower with nil power_type should not change the power type
    let power: i32 = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(42000, 90000)
            return UnitPower("player")
            "#,
        )
        .unwrap();
    assert_eq!(power, 42000);
}

#[test]
fn test_set_player_holy_power_uses_separate_pool() {
    let env = env();
    let (mana, mana_max, holy, holy_max, power_type): (i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            A_Admin.SetPlayerPower(42000, 90000, 0)
            A_Admin.SetPlayerPower(3, 5, 9)
            local pt = select(1, UnitPowerType("player"))
            return UnitPower("player"), UnitPowerMax("player"), UnitPower("player", 9), UnitPowerMax("player", 9), pt
            "#,
        )
        .unwrap();
    assert_eq!(mana, 42000);
    assert_eq!(mana_max, 90000);
    assert_eq!(holy, 3);
    assert_eq!(holy_max, 5);
    assert_eq!(power_type, 0);
}

// ============================================================================
// SetTargetHealth
// ============================================================================

#[test]
fn test_set_target_health_requires_target_first() {
    let env = env();
    let hp: i32 = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            A_Admin.SetTargetHealth(50000, 100000)
            return UnitHealth("target")
            "#,
        )
        .unwrap();
    assert_eq!(hp, 50000);
}

#[test]
fn test_set_target_health_max() {
    let env = env();
    let hp_max: i32 = env
        .eval(
            r#"
            A_Admin.SetTarget("Boss", 63, 1, true)
            A_Admin.SetTargetHealth(50000, 100000)
            return UnitHealthMax("target")
            "#,
        )
        .unwrap();
    assert_eq!(hp_max, 100000);
}

#[test]
fn test_set_target_health_silently_ignored_without_target() {
    let env = env();
    // Without a target, SetTargetHealth should not crash
    let ok: bool = env
        .eval(
            r#"
            A_Admin.SetTargetHealth(50000, 100000)
            return true
            "#,
        )
        .unwrap();
    assert!(ok);
}

#[test]
fn test_set_target_health_default_before_set() {
    let env = env();
    let hp: i32 = env
        .eval(
            r#"
            A_Admin.SetTarget("Miniboss", 20, 4, true)
            return UnitHealth("target")
            "#,
        )
        .unwrap();
    // Default health for new target is 100_000
    assert_eq!(hp, 100_000);
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_percent_supplied_scalar_curve_tracks_live_health() {
    let env = env();
    env.exec(
        r#"
        local function one_result(...)
            assert(select('#', ...) == 1, 'health percentage returns one result')
            return ...
        end
        -- The 0..100 input scale is simulator policy, not native evidence.
        local curve = C_CurveUtil.CreateCurve()
        curve:AddPoint(0, 7)
        curve:AddPoint(25, 80)
        curve:AddPoint(100, 20)
        A_Admin.SetPlayerHealth(5000, 20000)
        assert(one_result(UnitHealthPercent('player', false, curve)) == 80,
            '25 percent must evaluate the supplied scalar curve to 80')
        A_Admin.SetPlayerHealth(12500, 20000)
        assert(one_result(UnitHealthPercent('player', false, curve)) == 50,
            '62.5 percent must interpolate the supplied scalar curve to 50')
        A_Admin.SetPlayerHealth(20000, 20000)
        assert(one_result(UnitHealthPercent('player', false, curve)) == 20)
        "#,
    )
    .expect("supplied scalar curve evaluates current modeled health percentage");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_percent_supplied_color_curve_tracks_live_target_health() {
    let env = env();
    env.exec(
        r#"
        local function one_result(...)
            assert(select('#', ...) == 1, 'health percentage returns one result')
            return ...
        end
        -- The 0..100 input scale is simulator policy, not native evidence.
        local curve = C_CurveUtil.CreateColorCurve()
        curve:AddPoint(0, CreateColor(1, 0, 0, 1))
        curve:AddPoint(25, CreateColor(0, 1, 0.5, 0.5))
        curve:AddPoint(100, CreateColor(1, 0, 1, 1))
        A_Admin.SetTarget('Boss', 63, 1, true)
        A_Admin.SetTargetHealth(5000, 20000)
        local color = one_result(UnitHealthPercent('target', false, curve))
        assert(type(color) == 'table', 'supplied color curve must return a color')
        assert(color.r == 0 and color.g == 1 and color.b == 0.5 and color.a == 0.5)
        A_Admin.SetTargetHealth(12500, 20000)
        color = one_result(UnitHealthPercent('target', false, curve))
        assert(color.r == 0.5 and color.g == 0.5 and color.b == 0.75 and color.a == 0.75)
        A_Admin.SetTargetHealth(20000, 20000)
        color = one_result(UnitHealthPercent('target', false, curve))
        assert(color.r == 1 and color.g == 0 and color.b == 1 and color.a == 1)
        "#,
    )
    .expect("supplied color curve evaluates current modeled target health percentage");
}

#[cfg(feature = "retail-12-0-0")]
#[test]
fn unit_health_percent_supplied_nil_preserves_uncurved_queries() {
    let env = env();
    env.exec(
        r#"
        local function one_result(...)
            assert(select('#', ...) == 1, 'health percentage returns one result')
            return ...
        end
        A_Admin.SetPlayerHealth(5000, 20000)
        assert(one_result(UnitHealthPercent('player')) == 25)
        assert(one_result(UnitHealthPercent('player', false, nil)) == 25)
        A_Admin.SetPlayerHealth(12500, 20000)
        assert(one_result(UnitHealthPercent('player')) == 62.5)
        assert(one_result(UnitHealthPercent('player', false, nil)) == 62.5)
        "#,
    )
    .expect("nil and omitted curves preserve ordinary numeric queries");
}
