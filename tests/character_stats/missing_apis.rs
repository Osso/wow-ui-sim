//! Explicit stat-input contracts. Numeric policies are guesses, not native parity.

use super::env;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state_types::{EffectiveAttackPower, PowerRegen, WeaponAttackPower};

fn assert_outputs(env: &WowLuaEnv, query: &str, expected: &[f64]) {
    let values = expected
        .iter()
        .map(f64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    env.exec(&format!(
        r#"
        local expected = {{{values}}}
        local function check(...)
            assert(select('#', ...) == #expected, {query:?} .. ' arity')
            for index, number in ipairs(expected) do
                local value = select(index, ...)
                assert(type(value) == 'number' and value == number,
                    {query:?} .. ' value ' .. index)
                assert(not issecretvalue(value))
            end
        end
        check({query})
    "#
    ))
    .unwrap_or_else(|error| panic!("{query}: {error}"));
}

fn configured_env() -> WowLuaEnv {
    let env = env();
    let guid: String = env.eval("return UnitGUID('player')").unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.stats.mastery_rating = 520;
        state.player.stats.spell_power_to_attack_power = 135.0;
        state.player.stats.attack_power_to_spell_power = 42.0;
        state.player.stats.spell_penetration = 77.0;
        state.player.stats.sturdiness_pct = 12.5;
        state.player.stats.haste_rating = 1700;
        state.pet.melee_haste_pct = Some(23.5);
        state.player.power_type = 3;
        state.player.power_regen.insert(
            0,
            PowerRegen {
                base: 11.0,
                casting: 7.0,
            },
        );
        state.player.power_regen.insert(
            3,
            PowerRegen {
                base: 19.0,
                casting: 13.0,
            },
        );
        state.player.effective_attack_power = Some(EffectiveAttackPower {
            main_hand: 101.0,
            off_hand: 202.0,
            ranged: 303.0,
            base: 404.0,
            base_ranged: 505.0,
        });
        state.weapon_attack_power.insert(
            guid,
            WeaponAttackPower {
                main_hand: 31.0,
                off_hand: 47.0,
                ranged: 59.0,
            },
        );
    }
    env
}

#[test]
fn mastery_tracks_existing_rating_and_effect_component() {
    let env = env();
    for (rating, expected) in [(520, 4.0), (1040, 8.0)] {
        env.state().borrow_mut().player.stats.mastery_rating = rating;
        assert_outputs(&env, "GetMastery()", &[expected]);
        let (total, component): (f64, f64) = env.eval("return GetMasteryEffect()").unwrap();
        assert_eq!((total, component), (8.0 + expected, expected));
    }
}

#[test]
fn override_ap_by_spell_power_tracks_conversion_percentage() {
    let env = configured_env();
    assert_outputs(&env, "GetOverrideAPBySpellPower()", &[135.0]);
    env.state()
        .borrow_mut()
        .player
        .stats
        .spell_power_to_attack_power = 175.0;
    assert_outputs(&env, "GetOverrideAPBySpellPower()", &[175.0]);
}

#[test]
fn override_spell_power_by_ap_tracks_independent_conversion_percentage() {
    let env = configured_env();
    assert_outputs(&env, "GetOverrideSpellPowerByAP()", &[42.0]);
    env.state()
        .borrow_mut()
        .player
        .stats
        .attack_power_to_spell_power = 67.0;
    assert_outputs(&env, "GetOverrideSpellPowerByAP()", &[67.0]);
    assert_outputs(&env, "GetOverrideAPBySpellPower()", &[135.0]);
}

#[test]
fn pet_melee_haste_is_independent_and_missing_pet_is_zero_guess() {
    let env = configured_env();
    assert_outputs(&env, "GetPetMeleeHaste()", &[23.5]);
    env.state().borrow_mut().player.stats.haste_rating = 3400;
    assert_outputs(&env, "GetPetMeleeHaste()", &[23.5]);
    env.state().borrow_mut().pet.melee_haste_pct = Some(31.0);
    assert_outputs(&env, "GetPetMeleeHaste()", &[31.0]);
    env.state().borrow_mut().pet.melee_haste_pct = None;
    assert_outputs(&env, "GetPetMeleeHaste()", &[0.0]);
}

#[test]
fn active_power_regen_selects_pool_and_tracks_updates() {
    let env = configured_env();
    assert_outputs(&env, "GetPowerRegen()", &[19.0, 13.0]);
    env.state().borrow_mut().player.power_type = 0;
    assert_outputs(&env, "GetPowerRegen()", &[11.0, 7.0]);
    env.state()
        .borrow_mut()
        .player
        .power_regen
        .get_mut(&0)
        .unwrap()
        .casting = 5.0;
    assert_outputs(&env, "GetPowerRegen()", &[11.0, 5.0]);
    env.state().borrow_mut().player.power_type = 99;
    assert_outputs(&env, "GetPowerRegen()", &[0.0, 0.0]);
}

#[test]
fn requested_power_regen_selects_input_not_active_pool() {
    let env = configured_env();
    assert_outputs(&env, "GetPowerRegenForPowerType(0)", &[11.0, 7.0]);
    assert_outputs(&env, "GetPowerRegenForPowerType(3)", &[19.0, 13.0]);
    env.state().borrow_mut().player.power_regen.insert(
        3,
        PowerRegen {
            base: 29.0,
            casting: 17.0,
        },
    );
    assert_outputs(&env, "GetPowerRegenForPowerType(3)", &[29.0, 17.0]);
    assert_outputs(&env, "GetPowerRegenForPowerType(99)", &[0.0, 0.0]);
    env.exec("assert(not pcall(GetPowerRegenForPowerType))")
        .unwrap();
}

#[test]
fn spell_penetration_tracks_explicit_amount() {
    let env = configured_env();
    assert_outputs(&env, "GetSpellPenetration()", &[77.0]);
    env.state().borrow_mut().player.stats.spell_penetration = 91.0;
    assert_outputs(&env, "GetSpellPenetration()", &[91.0]);
}

#[test]
fn sturdiness_tracks_explicit_percentage_not_avoidance() {
    let env = configured_env();
    assert_outputs(&env, "GetSturdiness()", &[12.5]);
    env.state().borrow_mut().player.stats.avoidance_rating = 999;
    env.state().borrow_mut().player.stats.sturdiness_pct = 18.0;
    assert_outputs(&env, "GetSturdiness()", &[18.0]);
}

#[test]
fn effective_attack_power_preserves_five_distinct_fields_and_unavailable_shape() {
    let env = configured_env();
    assert_outputs(
        &env,
        "PlayerEffectiveAttackPower()",
        &[101.0, 202.0, 303.0, 404.0, 505.0],
    );
    env.state()
        .borrow_mut()
        .player
        .effective_attack_power
        .as_mut()
        .unwrap()
        .off_hand = 212.0;
    assert_outputs(
        &env,
        "PlayerEffectiveAttackPower()",
        &[101.0, 212.0, 303.0, 404.0, 505.0],
    );
    env.state().borrow_mut().player.effective_attack_power = None;
    assert_outputs(&env, "PlayerEffectiveAttackPower()", &[]);
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec("assert(select('#', PlayerEffectiveAttackPower()) == 0)")
        .unwrap();
}

#[test]
fn weapon_attack_power_uses_existing_guid_and_never_creates_entities() {
    let env = configured_env();
    assert_outputs(&env, "UnitWeaponAttackPower('player')", &[31.0, 47.0, 59.0]);
    // Concrete target fixture: aliasing focus must preserve the same GUID-backed inputs.
    env.state().borrow_mut().current_target = Some(wow_ui_sim::lua_api::state::TargetInfo {
        unit_id: "target".into(),
        name: "Stat target".into(),
        class_index: 1,
        level: 10,
        health: 100,
        health_max: 100,
        power: 0,
        power_max: 100,
        power_type: 1,
        power_type_name: "RAGE".into(),
        is_player: false,
        is_enemy: true,
        guid: "Creature-0-0-0-0-448-000042".into(),
        classification: "normal".into(),
        creature_type: "Humanoid".into(),
        reaction: 2,
        interaction: Default::default(),
    });
    {
        let mut state = env.state().borrow_mut();
        state.current_focus = state.current_target.clone();
        state.weapon_attack_power.insert(
            "Creature-0-0-0-0-448-000042".into(),
            WeaponAttackPower {
                main_hand: 71.0,
                off_hand: 83.0,
                ranged: 97.0,
            },
        );
    }
    assert_outputs(&env, "UnitWeaponAttackPower('target')", &[71.0, 83.0, 97.0]);
    assert_outputs(&env, "UnitWeaponAttackPower('focus')", &[71.0, 83.0, 97.0]);
    env.state()
        .borrow_mut()
        .weapon_attack_power
        .get_mut("Creature-0-0-0-0-448-000042")
        .unwrap()
        .ranged = 107.0;
    assert_outputs(
        &env,
        "UnitWeaponAttackPower('target')",
        &[71.0, 83.0, 107.0],
    );
    assert_outputs(&env, "UnitWeaponAttackPower('player')", &[31.0, 47.0, 59.0]);
    env.state().borrow_mut().current_target = None;
    assert_outputs(&env, "UnitWeaponAttackPower('target')", &[0.0, 0.0, 0.0]);
    assert_outputs(&env, "UnitWeaponAttackPower('missing')", &[0.0, 0.0, 0.0]);
    env.exec("assert(not UnitExists('missing')); assert(not pcall(UnitWeaponAttackPower))")
        .unwrap();
}

const SECRET_FIXTURES: &str = r#"
    missingStatFixtures = {
        {'GetMastery', {}, {4}},
        {'GetOverrideAPBySpellPower', {}, {135}},
        {'GetOverrideSpellPowerByAP', {}, {42}},
        {'GetPetMeleeHaste', {}, {23.5}},
        {'GetPowerRegen', {}, {19, 13}},
        {'GetPowerRegenForPowerType', {0}, {11, 7}},
        {'GetSpellPenetration', {}, {77}},
        {'GetSturdiness', {}, {12.5}},
        {'PlayerEffectiveAttackPower', {}, {101, 202, 303, 404, 505}},
        {'UnitWeaponAttackPower', {'player'}, {31, 47, 59}},
    }
    function assertMissingStatOutputs(restricted)
        for _, fixture in ipairs(missingStatFixtures) do
            local name, args, expected = unpack(fixture)
            local function check(...)
                assert(select('#', ...) == #expected, name .. ' arity')
                for index, number in ipairs(expected) do
                    local value = select(index, ...)
                    assert(issecretvalue(value) == restricted, name .. ' secrecy ' .. index)
                    assert(secretunwrap(value) == number, name .. ' value ' .. index)
                end
            end
            check(_G[name](unpack(args)))
        end
    end
"#;

#[test]
fn missing_stats_toggle_preserves_all_values_and_arities() {
    let env = configured_env();
    env.exec(SECRET_FIXTURES).unwrap();
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!("assertMissingStatOutputs({restricted})"))
            .unwrap();
    }
}

#[test]
fn missing_stats_tainted_callers_receive_opaque_host_outputs() {
    let env = configured_env();
    env.exec(SECRET_FIXTURES).unwrap();
    env.exec("powerInput = secretwrap(0); weaponUnitInput = secretwrap('player')")
        .unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        local function probe()
            for _, fixture in ipairs(missingStatFixtures) do
                local name, args, expected = unpack(fixture)
                local function check(...)
                    assert(select('#', ...) == #expected, name)
                    for index = 1, #expected do
                        local value = select(index, ...)
                        assert(issecretvalue(value) and not canaccessvalue(value), name)
                        assert(not pcall(secretunwrap, value), name .. ' unwrap')
                        assert(not pcall(function() return value + 1 end), name .. ' arithmetic')
                    end
                end
                check(_G[name](unpack(args)))
                assert(debug.getstacktaint() == 'MissingStatProbe')
            end
            assert(not pcall(GetPowerRegenForPowerType, powerInput))
            assert(not pcall(UnitWeaponAttackPower, weaponUnitInput))
        end
        debug.setobjecttaint(probe, 'MissingStatProbe')
        probe()
        assertMissingStatOutputs(true)
    "#,
    )
    .unwrap();
}

#[test]
fn secret_selection_inputs_are_allowed_when_untainted() {
    let env = configured_env();
    assert_outputs(
        &env,
        "GetPowerRegenForPowerType(secretwrap(0))",
        &[11.0, 7.0],
    );
    assert_outputs(
        &env,
        "UnitWeaponAttackPower(secretwrap('player'))",
        &[31.0, 47.0, 59.0],
    );
}
