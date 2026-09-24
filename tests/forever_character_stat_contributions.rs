//! Bounded Forever Camelot PaperDoll primary-stat contribution contract.

#[cfg(feature = "client-wowforever")]
mod forever {
    use wow_ui_sim::lua_api::WowLuaEnv;

    fn env() -> WowLuaEnv {
        WowLuaEnv::new().expect("Forever Lua environment")
    }

    #[test]
    fn crit_contributions_use_passed_primary_stat_and_fraction_units() {
        let env = env();
        let (melee, doubled, wrong, spell, spell_wrong): (f64, f64, f64, f64, f64) = env
            .eval(
                "return GetCritChanceFromStat(2, 100), GetCritChanceFromStat(2, 200), \
                 GetCritChanceFromStat(4, 100), GetSpellCritChanceFromStat(4, 100), \
                 GetSpellCritChanceFromStat(2, 100)",
            )
            .unwrap();
        assert_eq!(
            (melee, doubled, wrong, spell, spell_wrong),
            (0.01, 0.02, 0.0, 0.01, 0.0)
        );
        assert_eq!(melee * 100.0, 1.0); // Camelot tooltip multiplies by 100.
        assert_eq!(
            env.eval::<f64>("return GetCritChanceFromStat(2, -100)")
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn ranged_attack_power_uses_class_and_passed_agility() {
        let env = env();
        for (class, expected) in [(3, 200.0), (1, 100.0), (4, 100.0), (2, 0.0)] {
            env.state().borrow_mut().player.class_index = class;
            let (one, doubled, wrong): (f64, f64, f64) = env
                .eval("return GetRangedAttackPowerForStat(2, 100), GetRangedAttackPowerForStat(2, 200), GetRangedAttackPowerForStat(1, 100)")
                .unwrap();
            assert_eq!((one, doubled, wrong), (expected, expected * 2.0, 0.0));
        }
    }

    #[test]
    fn spirit_state_drives_unit_stat_and_regen_without_changing_casting_baseline() {
        let env = env();
        let initial: (f64, f64, f64, f64) = env.eval("return UnitStat('player', 5)").unwrap();
        assert_eq!(initial, (0.0, 0.0, 0.0, 0.0));
        let (baseline, casting_before): (f64, f64) = env.eval("return GetManaRegen()").unwrap();
        env.state().borrow_mut().player.stats.spirit = 50.0;
        let stats: (f64, f64, f64, f64) = env.eval("return UnitStat('player', 5)").unwrap();
        assert_eq!(stats, (50.0, 50.0, 0.0, 0.0));
        let (health, health_combat, mana, mana_combat, total_health, total_health_combat, total_mana, casting_after): (f64, f64, f64, f64, f64, f64, f64, f64) = env.eval(
            "local h, hc = GetHealthRegenFromSpirit(); local m, mc = GetManaRegenFromSpirit(); \
             local th, thc = GetHealthRegen(); local tm, tc = GetManaRegen(); \
             return h, hc, m, mc, th, thc, tm, tc",
        ).unwrap();
        assert_eq!(
            (health, health_combat, mana, mana_combat),
            (10.0, 0.0, 5.0, 0.0)
        );
        assert_eq!((total_health, total_health_combat), (health, 0.0));
        assert_eq!(total_mana, baseline + mana);
        assert_eq!(casting_after, casting_before);
        assert_eq!(
            env.eval::<f64>("return UnitStat('target', 5)").unwrap(),
            0.0
        );
    }

    #[test]
    fn spirit_is_per_environment_and_restores_to_explicit_unseeded_zero() {
        let first = env();
        first.state().borrow_mut().player.stats.spirit = 20.0;
        let second = env();
        assert_eq!(
            second
                .eval::<f64>("return GetManaRegenFromSpirit()")
                .unwrap(),
            0.0
        );
        assert_eq!(
            first
                .eval::<f64>("return GetManaRegenFromSpirit()")
                .unwrap(),
            2.0
        );
        first.state().borrow_mut().player.stats.spirit = 0.0;
        assert_eq!(
            first.eval::<f64>("return UnitStat('player', 5)").unwrap(),
            0.0
        );
    }
}
