//! Explicit retail input, not inferred combat/aura activation.

use wow_ui_sim::lua_api::WowLuaEnv;

const FIXTURES: &str = include_str!("stat_restriction_fixtures.lua");

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let player = &mut state.player;
        player.level = 10;
        player.class_index = 2;
        player.active_spec_index = 3;
        player.stats.strength = 300.0;
        player.stats.agility = 200.0;
        player.stats.intellect = 500.0;
        player.stats.armor = 1234;
        player.stats.crit_rating = 360;
        player.stats.haste_rating = 510;
        player.stats.mastery_rating = 520;
        player.stats.versatility_rating = 1025;
        #[cfg(feature = "client-retail")]
        {
            player.movement.moving = true;
            player.movement_speeds.run = 9.0;
            player.movement_speeds.flight = 14.0;
            player.movement_speeds.swim = 4.0;
        }
    }
    env.exec(FIXTURES).unwrap();
    #[cfg(feature = "client-retail")]
    env.exec("table.insert(statFixtures, {'GetUnitSpeed', {'player'}, {9, 9, 14, 4}})")
        .unwrap();
    env
}

#[test]
fn stat_restriction_all_supported_outputs_preserve_values_arity_and_toggle() {
    let env = seeded_env();
    env.exec("assertStatOutputs(false)").unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        assert(C_Secrets.ShouldUnitStatsBeSecret() == true)
        assertStatOutputs(true)
        for _, value in ipairs({
            GetCombatRatingBonusForCombatRatingValue(9, 360),
            GetCritChanceFromAgility('player'),
            GetSpellCritChanceFromIntellect('player'),
            UnitCriticalStrike('player'), UnitResistance('player', 2),
            UnitHPPerStamina('player'), GetMeleeMissChance(),
            GetRangedMissChance(), GetEnemyDodgeChance(), UnitXP('player'),
        }) do
            assert(not issecretvalue(value) and canaccessvalue(value))
        end
        assert(not issecretvalue(0) and not issecretvalue(300))
        assert(not issecretvalue(C_Secrets.ShouldUnitStatsBeSecret()))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().unit_stats_restricted = false;
    env.exec("assert(not C_Secrets.ShouldUnitStatsBeSecret()); assertStatOutputs(false)")
        .unwrap();
}

#[test]
fn stat_restriction_tainted_callers_receive_opaque_host_results() {
    let env = seeded_env();
    env.exec("ratingInput = secretwrap(9); unitInput = secretwrap('player')")
        .unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        local function tainted()
            assert(not issecure())
            assert(C_Secrets.ShouldUnitStatsBeSecret())
            for _, fixture in ipairs(statFixtures) do
                local name, args, expected = unpack(fixture)
                local function check(...)
                    assert(select('#', ...) == #expected, name .. ' tainted arity')
                    for index = 1, select('#', ...) do
                        local value = select(index, ...)
                        assert(issecretvalue(value) and not canaccessvalue(value), name)
                        assert(not pcall(secretunwrap, value), name .. ' unwrap')
                        assert(not pcall(function() return value + 1 end), name .. ' arithmetic')
                    end
                end
                check(_G[name](unpack(args)))
                assert(debug.getstacktaint() == 'StatRestrictionProbe')
            end
            assert(not pcall(GetCombatRating, ratingInput))
            if GetUnitSpeed then assert(not pcall(GetUnitSpeed, unitInput)) end
        end
        debug.setobjecttaint(tainted, 'StatRestrictionProbe')
        tainted()
        assertStatOutputs(true)
        "#,
    )
    .unwrap();
}

#[test]
fn stat_restriction_absent_unit_shapes_and_constant_outputs() {
    let env = seeded_env();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec(
        r#"
        for _, query in ipairs({
            function() return UnitStat('missing', 1) end,
            function() return UnitArmor('missing') end,
            function() return UnitAttackPower('missing') end,
        }) do
            local function check(...)
                for index = 1, select('#', ...) do
                    local value = select(index, ...)
                    assert(issecretvalue(value) and secretunwrap(value) == 0)
                end
            end
            check(query())
        end
        assert(select('#', UnitStat('missing', 1)) == 4)
        assert(select('#', UnitArmor('missing')) == 5)
        assert(select('#', UnitAttackPower('missing')) == 3)
        if GetUnitSpeed then
            assert(select('#', GetUnitSpeed('missing')) == 4)
            local a, b, c, d = GetUnitSpeed('missing')
            for _, value in ipairs({a, b, c, d}) do
                assert(issecretvalue(value) and secretunwrap(value) == 0)
            end
        end
        assert(secretunwrap(GetBlockChance()) == 0)
        assert(secretunwrap(GetAttackPowerForStat(4, 37)) == 0)
        "#,
    )
    .unwrap();
}

#[test]
fn stat_restriction_predicate_defaults_plain() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_Secrets.ShouldUnitStatsBeSecret) == 'function')
        assert(select('#', C_Secrets.ShouldUnitStatsBeSecret()) == 1)
        assert(C_Secrets.ShouldUnitStatsBeSecret() == false)
        assert(not issecretvalue(GetCombatRating(9)))
        assert(not issecretvalue(GetBlockChance()))
        "#,
    )
    .unwrap();
    env.state().borrow_mut().unit_stats_restricted = true;
    env.exec("assert(C_Secrets.ShouldUnitStatsBeSecret()); assert(not issecretvalue(C_Secrets.ShouldUnitStatsBeSecret()))")
        .unwrap();
    env.state().borrow_mut().unit_stats_restricted = false;
    env.exec("assert(not C_Secrets.ShouldUnitStatsBeSecret())")
        .unwrap();
}
