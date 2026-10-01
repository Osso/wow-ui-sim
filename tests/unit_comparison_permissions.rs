//! 12.0.5 comparison permissions, independent of modeled unit existence.

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_symmetric(env: &WowLuaEnv, lhs: &str, rhs: &str, expected: Option<bool>) {
    for (first, second) in [(lhs, rhs), (rhs, lhs)] {
        let actual: Option<bool> = env
            .eval(&format!("return UnitIsUnit('{first}', '{second}')"))
            .unwrap();
        assert_eq!(actual, expected, "{first} versus {second}");
    }
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn comparison_permissions_base_tokens_allow_even_restricted_counterparts() {
    let env = WowLuaEnv::new().unwrap();
    for base in [
        "player",
        "pet",
        "vehicle",
        "mouseover",
        "target",
        "softenemy",
        "softfriend",
        "softinteract",
        "focus",
        "none",
        "npc",
        "questnpc",
    ] {
        for other in ["boss1target", "nameplate1", "targettarget", "focustarget"] {
            assert_symmetric(&env, base, other, Some(false));
        }
    }
    env.exec("TargetUnit('player'); FocusUnit('target')")
        .unwrap();
    assert_symmetric(&env, "player", "target", Some(true));
    assert_symmetric(&env, "player", "focus", Some(true));
    // Permission cannot fabricate a GUID for currently unsupported compounds.
    assert_symmetric(&env, "player", "targettarget", Some(false));
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn comparison_permissions_group_and_pet_tokens_allow_simple_counterparts() {
    let env = WowLuaEnv::new().unwrap();
    for group in [
        "party1",
        "party4",
        "raid1",
        "raid40",
        "partypet1",
        "partypet4",
        "raidpet1",
        "raidpet40",
    ] {
        for simple in [
            "arena1",
            "arenapet1",
            "boss1",
            "party2",
            "raid2",
            "partypet2",
            "raidpet2",
        ] {
            assert_symmetric(&env, group, simple, Some(false));
        }
    }
    env.exec("A_Admin.SetPartySize(2); TargetUnit('party2'); FocusUnit('target')")
        .unwrap();
    assert_symmetric(&env, "party2", "target", Some(true));
    assert_symmetric(&env, "party2", "focus", Some(true));
    assert_symmetric(&env, "party1", "party2", Some(false));
    assert_symmetric(&env, "target", "focus", Some(true));
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn comparison_permissions_group_and_pet_tokens_deny_restricted_counterparts() {
    let env = WowLuaEnv::new().unwrap();
    for group in ["party1", "raid40", "partypet4", "raidpet1"] {
        for restricted in [
            "boss1target",
            "arena1target",
            "party2target",
            "raid2target",
            "pettarget",
            "targettarget",
            "focustarget",
            "targettargettarget",
            "nameplate1",
            "nameplate1target",
        ] {
            assert_symmetric(&env, group, restricted, None);
        }
    }
    env.exec("assert(select('#', UnitIsUnit('party1', 'nameplate1')) == 0)")
        .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn comparison_permissions_nonbase_pairs_are_denied_even_when_identical() {
    let env = WowLuaEnv::new().unwrap();
    for lhs in [
        "arena1",
        "arenapet1",
        "boss1",
        "nameplate1",
        "targettarget",
        "unknown",
    ] {
        for rhs in [
            "arena1",
            "arenapet1",
            "boss1",
            "nameplate1",
            "focustarget",
            "unknown",
        ] {
            assert_symmetric(&env, lhs, rhs, None);
        }
    }
    for invalid_group in [
        "party0",
        "party5",
        "party01",
        "raid0",
        "raid41",
        "raid01",
        "partypet5",
        "raidpet41",
    ] {
        assert_symmetric(&env, invalid_group, "boss1", None);
    }
}

#[test]
fn comparison_permissions_preserve_missing_argument_and_type_conversion_controls() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(UnitIsUnit(nil, nil) == false)
        assert(UnitIsUnit() == false)
        assert(UnitIsUnit(nil, 'player') == false)
        assert(UnitIsUnit('player', nil) == false)
        assert(not pcall(UnitIsUnit, 1, 'player'))
        assert(not pcall(UnitIsUnit, 'player', {}))
        assert(not pcall(UnitIsUnit, true, 'player'))
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn comparison_permissions_typed_secrets_require_untainted_callers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local secretPlayer = secretwrap('player')
        local secretPlate = secretwrap('nameplate1')
        assert(UnitIsUnit(secretPlayer, 'player') == true)
        assert(UnitIsUnit('player', secretPlayer) == true)
        assert(UnitIsUnit(secretPlate, 'boss1') == nil)
        assert(UnitIsUnit(secretPlayer, secretPlate) == false)
        assert(select('#', UnitIsUnit(secretPlate, 'boss1')) == 0)
        for _, value in ipairs({secretwrap(1), secretwrap(true), secretwrap({})}) do
            assert(not pcall(UnitIsUnit, value, 'player'))
        end
        assert(not pcall(UnitIsUnit, secretwrap(nil), 'player'))
        assert(not pcall(UnitIsUnit, CreateFrame('Frame'), 'player'))
        local function tainted()
            assert(not issecure())
            assert(not pcall(UnitIsUnit, secretPlayer, 'player'))
            assert(not pcall(UnitIsUnit, 'player', secretPlayer))
            assert(not pcall(UnitIsUnit, secretPlate, 'boss1'))
            assert(UnitIsUnit('player', 'player') == true)
            assert(debug.getstacktaint() == 'UnitComparisonProbe')
        end
        debug.setobjecttaint(tainted, 'UnitComparisonProbe')
        tainted()
        assert(issecretvalue(secretPlayer))
        "#,
    )
    .unwrap();
}

#[cfg(not(feature = "retail-12-0-5"))]
#[test]
fn comparison_permissions_older_profiles_keep_boolean_results() {
    let env = WowLuaEnv::new().unwrap();
    for (lhs, rhs) in [
        ("boss1", "boss1"),
        ("nameplate1", "arena1"),
        ("party1", "boss1target"),
        ("unknown", "unknown"),
    ] {
        assert_symmetric(&env, lhs, rhs, Some(false));
    }
}
