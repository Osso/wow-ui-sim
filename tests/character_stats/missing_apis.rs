//! Missing patch stat contracts. Numeric models other than mastery await explicit inputs.
//! Mastery percentage is inferred from the existing rating-derived effect component.

use super::env;
use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_numeric_returns(env: &WowLuaEnv, query: &str, count: usize, may_return_nothing: bool) {
    let script = format!(
        r#"
        local function check(...)
            local count = select('#', ...)
            assert(count == {count} or ({may_return_nothing} and count == 0),
                {query:?} .. ': unexpected return count ' .. count)
            for index = 1, count do
                assert(type(select(index, ...)) == 'number',
                    {query:?} .. ': nonnumeric return ' .. index)
            end
        end
        check({query})
        "#,
    );
    env.exec(&script)
        .unwrap_or_else(|error| panic!("{query}: {error}"));
}

#[test]
fn mastery_tracks_existing_rating_and_effect_component() {
    let env = env();
    for (rating, expected) in [(520, 4.0), (1040, 8.0)] {
        env.state().borrow_mut().player.stats.mastery_rating = rating;
        assert_numeric_returns(&env, "GetMastery()", 1, false);
        let (mastery, total_effect, rating_effect): (f64, f64, f64) = env
            .eval("local total, rating = GetMasteryEffect(); return GetMastery(), total, rating")
            .unwrap();
        assert_eq!(mastery, expected, "mastery rating {rating}");
        assert_eq!(mastery, rating_effect);
        assert_eq!(total_effect, 8.0 + expected);
    }
}

#[test]
fn override_ap_by_spell_power_returns_one_number() {
    assert_numeric_returns(&env(), "GetOverrideAPBySpellPower()", 1, false);
}

#[test]
fn override_spell_power_by_ap_returns_one_number() {
    assert_numeric_returns(&env(), "GetOverrideSpellPowerByAP()", 1, false);
}

#[test]
fn pet_melee_haste_returns_one_number() {
    assert_numeric_returns(&env(), "GetPetMeleeHaste()", 1, false);
}

#[test]
fn active_power_regen_returns_base_and_casting_numbers() {
    assert_numeric_returns(&env(), "GetPowerRegen()", 2, false);
}

#[test]
fn requested_power_regen_accepts_mana_and_energy_returns_two_numbers() {
    let env = env();
    // WoW power IDs: mana = 0, energy = 3. Selection/value fixtures await the model.
    assert_numeric_returns(&env, "GetPowerRegenForPowerType(0)", 2, false);
    assert_numeric_returns(&env, "GetPowerRegenForPowerType(3)", 2, false);
}

#[test]
fn spell_penetration_returns_one_number() {
    assert_numeric_returns(&env(), "GetSpellPenetration()", 1, false);
}

#[test]
fn sturdiness_returns_one_number() {
    assert_numeric_returns(&env(), "GetSturdiness()", 1, false);
}

#[test]
fn effective_attack_power_returns_five_numbers_or_no_values() {
    assert_numeric_returns(&env(), "PlayerEffectiveAttackPower()", 5, true);
}

#[test]
fn player_weapon_attack_power_returns_three_numbers() {
    assert_numeric_returns(&env(), "UnitWeaponAttackPower('player')", 3, false);
}
