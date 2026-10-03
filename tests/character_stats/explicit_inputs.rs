//! Explicit inputs replacing constant and proxy stat producers. Defaults and
//! units are simulator policy, not native parity.

use super::env;
use wow_ui_sim::lua_api::WowLuaEnv;

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
    {
        let mut state = env.state().borrow_mut();
        let stats = &mut state.player.stats;
        stats.armor = 1234;
        stats.intellect = 999.0;
        stats.block_chance = 12.5;
        stats.shield_block = 79.0;
        stats.hit_modifier = 1.25;
        stats.spell_hit_modifier = 2.75;
        stats.expertise = [11.0, 23.0, 37.0];
        stats.expertise_percent = [1.5, 2.5, 3.5];
        stats.mod_resilience_damage_reduction = 17.5;
        stats.pvp_power_damage = 11.5;
        stats.pvp_power_healing = 23.5;
        state.pet.spell_bonus_damage = Some(41.5);
    }
    env
}

const SCALAR_QUERIES: [(&str, f64); 8] = [
    ("GetBlockChance()", 12.5),
    ("GetShieldBlock()", 79.0),
    ("GetHitModifier()", 1.25),
    ("GetSpellHitModifier()", 2.75),
    ("GetModResilienceDamageReduction()", 17.5),
    ("GetPvpPowerDamage()", 11.5),
    ("GetPvpPowerHealing()", 23.5),
    ("GetPetSpellBonusDamage()", 41.5),
];

#[test]
fn unconfigured_inputs_return_public_zeros_with_declared_arity() {
    let env = env();
    for (query, _) in SCALAR_QUERIES {
        assert_outputs(&env, query, &[0.0]);
    }
    assert_outputs(&env, "GetExpertise()", &[0.0, 0.0, 0.0]);
    assert_outputs(&env, "GetExpertisePercent()", &[0.0, 0.0, 0.0]);
}

#[test]
fn each_scalar_reads_its_own_configured_input() {
    let env = configured_env();
    for (query, expected) in SCALAR_QUERIES {
        assert_outputs(&env, query, &[expected]);
    }
}

#[test]
fn scalar_inputs_update_live_without_moving_neighbours() {
    let env = configured_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.stats.hit_modifier = 3.5;
        state.player.stats.pvp_power_damage = 13.0;
        state.player.stats.block_chance = 18.0;
    }
    assert_outputs(&env, "GetHitModifier()", &[3.5]);
    assert_outputs(&env, "GetSpellHitModifier()", &[2.75]);
    assert_outputs(&env, "GetPvpPowerDamage()", &[13.0]);
    assert_outputs(&env, "GetPvpPowerHealing()", &[23.5]);
    assert_outputs(&env, "GetModResilienceDamageReduction()", &[17.5]);
    assert_outputs(&env, "GetBlockChance()", &[18.0]);
    assert_outputs(&env, "GetShieldBlock()", &[79.0]);
}

#[test]
fn shield_block_is_not_armor_and_takes_no_unit() {
    let env = configured_env();
    assert_outputs(&env, "GetShieldBlock()", &[79.0]);
    env.state().borrow_mut().player.stats.armor = 9999;
    assert_outputs(&env, "GetShieldBlock()", &[79.0]);
    assert_outputs(&env, "GetShieldBlock('target')", &[79.0]);
    assert_outputs(&env, "GetShieldBlock('missing')", &[79.0]);
    env.state().borrow_mut().player.stats.shield_block = 83.0;
    assert_outputs(&env, "GetShieldBlock()", &[83.0]);
}

#[test]
fn expertise_triples_are_ordered_and_independent() {
    let env = configured_env();
    assert_outputs(&env, "GetExpertise()", &[11.0, 23.0, 37.0]);
    assert_outputs(&env, "GetExpertisePercent()", &[1.5, 2.5, 3.5]);
    env.state().borrow_mut().player.stats.expertise[1] = 29.0;
    assert_outputs(&env, "GetExpertise()", &[11.0, 29.0, 37.0]);
    assert_outputs(&env, "GetExpertisePercent()", &[1.5, 2.5, 3.5]);
    env.state().borrow_mut().player.stats.expertise_percent[2] = 4.5;
    assert_outputs(&env, "GetExpertisePercent()", &[1.5, 2.5, 4.5]);
    assert_outputs(&env, "GetExpertise()", &[11.0, 29.0, 37.0]);
}

#[test]
fn pet_spell_bonus_is_optional_and_independent_of_player_stats() {
    let env = configured_env();
    assert_outputs(&env, "GetPetSpellBonusDamage()", &[41.5]);
    env.state().borrow_mut().player.stats.intellect = 5.0;
    assert_outputs(&env, "GetPetSpellBonusDamage()", &[41.5]);
    env.state().borrow_mut().pet.spell_bonus_damage = Some(63.0);
    assert_outputs(&env, "GetPetSpellBonusDamage()", &[63.0]);
    env.state().borrow_mut().pet.spell_bonus_damage = None;
    assert_outputs(&env, "GetPetSpellBonusDamage()", &[0.0]);
}

#[test]
fn environments_do_not_share_inputs() {
    let configured = configured_env();
    let fresh = env();
    assert_outputs(&configured, "GetBlockChance()", &[12.5]);
    assert_outputs(&fresh, "GetBlockChance()", &[0.0]);
    assert_outputs(&fresh, "GetExpertise()", &[0.0, 0.0, 0.0]);
}

const SECRET_FIXTURES: &str = r#"
    explicitStatFixtures = {
        {'GetBlockChance', {12.5}},
        {'GetShieldBlock', {79}},
        {'GetHitModifier', {1.25}},
        {'GetSpellHitModifier', {2.75}},
        {'GetExpertise', {11, 23, 37}},
        {'GetExpertisePercent', {1.5, 2.5, 3.5}},
        {'GetModResilienceDamageReduction', {17.5}},
        {'GetPvpPowerDamage', {11.5}},
        {'GetPvpPowerHealing', {23.5}},
        {'GetPetSpellBonusDamage', {41.5}},
    }
    function assertExplicitStatOutputs(restricted)
        for _, fixture in ipairs(explicitStatFixtures) do
            local name, expected = unpack(fixture)
            local function check(...)
                assert(select('#', ...) == #expected, name .. ' arity')
                for index, number in ipairs(expected) do
                    local value = select(index, ...)
                    assert(issecretvalue(value) == restricted, name .. ' secrecy ' .. index)
                    assert(canaccessvalue(value) == not restricted, name .. ' access ' .. index)
                    assert(secretunwrap(value) == number, name .. ' value ' .. index)
                end
            end
            check(_G[name]())
        end
    end
"#;

#[test]
fn restriction_toggle_preserves_configured_values_and_arities() {
    let env = configured_env();
    env.exec(SECRET_FIXTURES).unwrap();
    for restricted in [false, true, false] {
        env.state().borrow_mut().unit_stats_restricted = restricted;
        env.exec(&format!("assertExplicitStatOutputs({restricted})"))
            .unwrap();
    }
}

#[test]
fn restricted_tainted_callers_receive_opaque_configured_outputs() {
    let env = configured_env();
    env.exec(SECRET_FIXTURES).unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        local function probe()
            for _, fixture in ipairs(explicitStatFixtures) do
                local name, expected = unpack(fixture)
                local function check(...)
                    assert(select('#', ...) == #expected, name)
                    for index = 1, #expected do
                        local value = select(index, ...)
                        assert(issecretvalue(value) and not canaccessvalue(value), name)
                        assert(not pcall(secretunwrap, value), name .. ' unwrap')
                        assert(not pcall(function() return value + 1 end), name .. ' arithmetic')
                    end
                end
                check(_G[name]())
                assert(debug.getstacktaint() == 'ExplicitStatProbe')
            end
        end
        debug.setobjecttaint(probe, 'ExplicitStatProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assertExplicitStatOutputs(true)
    "#,
    )
    .unwrap();
}
