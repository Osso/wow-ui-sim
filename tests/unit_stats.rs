//! Integration tests for `src/lua_api/globals/unit_stats.rs`.
//!
//! Each probe has two shapes worth covering:
//! 1. Returns numbers matching the seeded `PlayerState` / `TargetInfo`.
//! 2. Falls back to 0 for units the sim doesn't model (e.g. `"mouseover"`).

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::SecondaryPowerState;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── Armor / AP / Ranged AP ────────────────────────────────────────────────────

#[test]
fn unit_armor_returns_five_values_for_player() {
    let env = env();
    let (base, effective, armor, pos, neg): (i32, i32, i32, i32, i32) =
        env.eval(r#"return UnitArmor("player")"#).unwrap();
    assert!(armor > 0, "seeded player has gear-derived armor");
    assert_eq!(base, effective);
    assert_eq!(base, armor);
    assert_eq!(pos, 0);
    assert_eq!(neg, 0);
}

#[test]
fn unit_attack_power_returns_three_values() {
    let env = env();
    let (base, pos, neg): (i32, i32, i32) =
        env.eval(r#"return UnitAttackPower("player")"#).unwrap();
    assert!(base > 0);
    assert_eq!(pos, 0);
    assert_eq!(neg, 0);
}

#[test]
fn unit_ranged_attack_power_matches_melee_in_sim() {
    let env = env();
    let (melee, ranged): (i32, i32) = env
        .eval(r#"return UnitAttackPower("player"), UnitRangedAttackPower("player")"#)
        .unwrap();
    assert_eq!(melee, ranged);
}

// ── Crit / Haste ──────────────────────────────────────────────────────────────

#[test]
fn unit_critical_strike_reads_player_crit_rating() {
    let env = env();
    let pct: f64 = env.eval(r#"return UnitCriticalStrike("player")"#).unwrap();
    assert!(pct > 0.0, "seeded gear gives a positive crit chance");
}

#[test]
fn unit_ranged_critical_strike_matches_melee_in_sim() {
    let env = env();
    let (melee, ranged): (f64, f64) = env
        .eval(r#"return UnitCriticalStrike("player"), UnitRangedCriticalStrike("player")"#)
        .unwrap();
    assert_eq!(melee, ranged);
}

#[test]
fn unit_spell_haste_derives_from_haste_rating() {
    let env = env();
    let pct: f64 = env.eval(r#"return UnitSpellHaste("player")"#).unwrap();
    assert!(pct > 0.0);
}

// ── Damage ────────────────────────────────────────────────────────────────────

#[test]
fn unit_damage_returns_seven_values() {
    let env = env();
    let (min_dmg, max_dmg, off_min, off_max, phys_pos, phys_neg, pct): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = env.eval(r#"return UnitDamage("player")"#).unwrap();
    assert!(min_dmg > 0.0);
    assert!(max_dmg >= min_dmg);
    assert_eq!(off_min, 0.0);
    assert_eq!(off_max, 0.0);
    assert_eq!(phys_pos, 0.0);
    assert_eq!(phys_neg, 0.0);
    assert_eq!(pct, 1.0, "damage multiplier defaults to 1x");
}

#[cfg(feature = "client-mists")]
#[test]
fn unit_ranged_damage_returns_legacy_ranged_shape() {
    let env = env();
    let (speed, min_dmg, max_dmg, pos, neg, pct): (f64, f64, f64, f64, f64, f64) =
        env.eval(r#"return UnitRangedDamage("player")"#).unwrap();
    assert!(speed > 0.0);
    assert!(min_dmg > 0.0);
    assert!(max_dmg >= min_dmg);
    assert_eq!(pos, 0.0);
    assert_eq!(neg, 0.0);
    assert_eq!(pct, 1.0);
}

#[cfg(feature = "client-mists")]
#[test]
fn legacy_weapon_speed_and_ranged_attack_are_numeric() {
    let env = env();
    let (main_speed, off_speed, ranged_base, ranged_mod): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local mainSpeed, offSpeed = UnitAttackSpeed("player")
            local rangedBase, rangedMod = UnitRangedAttack("player")
            return mainSpeed, offSpeed, rangedBase, rangedMod
            "#,
        )
        .unwrap();
    assert!(main_speed > 0.0);
    assert!(off_speed > 0.0);
    assert!(ranged_base > 0.0);
    assert_eq!(ranged_mod, 0.0);
}

// ── Defense / Dodge / Parry ───────────────────────────────────────────────────

#[test]
fn unit_defense_scales_with_level() {
    let env = env();
    let def: i32 = env.eval(r#"return UnitDefense("player")"#).unwrap();
    // Player defaults to level 70 → 350.
    assert_eq!(def, 350);
}

#[test]
fn unit_dodge_and_parry_return_base_percents() {
    let env = env();
    let (dodge, parry): (f64, f64) = env
        .eval(r#"return UnitDodge("player"), UnitParry("player")"#)
        .unwrap();
    assert!(dodge >= 5.0, "dodge starts at the 5% baseline");
    assert!(parry >= 5.0, "parry starts at the 5% baseline");
}

// ── Reaction ──────────────────────────────────────────────────────────────────

#[test]
fn unit_reaction_player_to_self_is_friendly() {
    let env = env();
    let reaction: i32 = env
        .eval(r#"return UnitReaction("player", "player")"#)
        .unwrap();
    assert_eq!(reaction, 5);
}

#[test]
fn unit_reaction_nonexistent_unit_returns_neutral() {
    let env = env();
    let reaction: i32 = env
        .eval(r#"return UnitReaction("mouseover", "player")"#)
        .unwrap();
    assert_eq!(reaction, 4);
}

// ── Health / Power max ────────────────────────────────────────────────────────

#[test]
fn unit_health_max_reads_player_health_max() {
    let env = env();
    env.state().borrow_mut().player.health_max = 500_000;
    let hp: i32 = env.eval(r#"return UnitHealthMax("player")"#).unwrap();
    assert_eq!(hp, 500_000);
}

#[test]
fn unit_power_max_returns_only_maximum() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.power_max = 1000;
        st.player.power_type = 3; // ENERGY
    }
    let (count, max): (i32, i32) = env
        .eval(r#"return select('#', UnitPowerMax("player")), UnitPowerMax("player")"#)
        .unwrap();
    assert_eq!(count, 1, "UnitPowerMax returns a maximum, not a power type");
    assert_eq!(max, 1000);
}

#[test]
fn unit_power_max_returns_requested_secondary_pool_only() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.power_max = 1000;
        st.player.power_type = 0;
        st.player
            .secondary_powers
            .insert(9, SecondaryPowerState { current: 3, max: 5 });
    }
    let holy_max: i32 = env.eval(r#"return UnitPowerMax("player", 9)"#).unwrap();
    assert_eq!(holy_max, 5);
}

// ── XP / XPMax ────────────────────────────────────────────────────────────────

#[test]
fn unit_xp_reads_player_state() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.xp = 12_345;
        st.player.xp_max = 200_000;
    }
    let (xp, xp_max): (i64, i64) = env
        .eval(r#"return UnitXP("player"), UnitXPMax("player")"#)
        .unwrap();
    assert_eq!(xp, 12_345);
    assert_eq!(xp_max, 200_000);
}

#[test]
fn unit_xp_zero_for_non_player_units() {
    let env = env();
    env.state().borrow_mut().player.xp = 999;
    let (xp, xp_max): (i64, i64) = env
        .eval(r#"return UnitXP("target"), UnitXPMax("target")"#)
        .unwrap();
    assert_eq!(xp, 0);
    assert_eq!(xp_max, 0);
}

// ── UnitStat ──────────────────────────────────────────────────────────────────

#[test]
fn unit_stat_indexes_map_to_strength_agility_stamina_intellect() {
    let env = env();
    let (strength, agility, stamina, intellect, unused): (f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local s = select(1, UnitStat("player", 1))
            local a = select(1, UnitStat("player", 2))
            local st = select(1, UnitStat("player", 3))
            local i = select(1, UnitStat("player", 4))
            local bogus = select(1, UnitStat("player", 99))
            return s, a, st, i, bogus
            "#,
        )
        .unwrap();
    assert!(strength > 0.0);
    assert!(agility > 0.0);
    assert!(stamina > 0.0);
    assert!(intellect > 0.0);
    assert_eq!(unused, 0.0, "unknown stat indexes return 0");
}

// ── UnitResistance ────────────────────────────────────────────────────────────

#[test]
fn unit_resistance_returns_four_zero_values() {
    let env = env();
    let (base, res, pos, neg): (i32, i32, i32, i32) =
        env.eval(r#"return UnitResistance("player", 2)"#).unwrap();
    assert_eq!((base, res, pos, neg), (0, 0, 0, 0));
}

// ── PaperDoll helpers ────────────────────────────────────────────────────────

#[test]
fn attack_power_for_stat_is_defined_for_strength_and_agility() {
    let env = env();
    let (str_ap, agi_ap, int_ap): (f64, f64, f64) = env
        .eval(
            r#"
            return GetAttackPowerForStat(1, 150),
                   GetAttackPowerForStat(2, 120),
                   GetAttackPowerForStat(4, 999)
            "#,
        )
        .unwrap();
    // Seeded player is Paladin spec index 2 (Protection): Strength primary.
    assert_eq!(str_ap, 150.0);
    assert_eq!(agi_ap, 0.0);
    assert_eq!(int_ap, 0.0);
}

#[test]
fn paperdoll_attribute_helpers_return_numbers() {
    let env = env();
    let (dodge_from_attr, parry_from_attr): (f64, f64) = env
        .eval(r#"return GetDodgeChanceFromAttribute(), GetParryChanceFromAttribute()"#)
        .unwrap();
    assert!(dodge_from_attr >= 0.0);
    assert!(parry_from_attr >= 0.0);
}

#[test]
fn paperdoll_health_and_ap_sp_helpers_are_available() {
    let env = env();
    let (stamina, hp_max, hp_per_stam, health_mod, health_mod_percent, ap_to_sp, sp_to_ap): (
        f64,
        f64,
        f64,
        f64,
        f64,
        bool,
        bool,
    ) = env
        .eval(
            r#"
            return select(1, UnitStat("player", 3)),
                   UnitHealthMax("player"),
                   UnitHPPerStamina("player"),
                   GetUnitMaxHealthModifier("player"),
                   GetUnitTotalModifiedMaxHealthPercent("player"),
                   HasAPEffectsSpellPower(),
                   HasSPEffectsAttackPower()
            "#,
        )
        .unwrap();
    let projected_hp = stamina * hp_per_stam * health_mod;
    assert!(
        (projected_hp - hp_max).abs() <= 0.5,
        "paperdoll stamina projection should match UnitHealthMax"
    );
    assert!(hp_per_stam > 0.0);
    assert!(health_mod > 0.0);
    assert_eq!(health_mod_percent, 0.0);
    assert!(ap_to_sp, "seeded protection paladin is a physical spec");
    assert!(!sp_to_ap);
}

#[test]
fn paperdoll_helpers_track_primary_stat_by_spec() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.class_index = 11; // Druid
        st.player.active_spec_index = 2; // Feral (Agility)
    }
    let (str_ap, agi_ap, ap_to_sp): (f64, f64, bool) = env
        .eval(
            r#"
            return GetAttackPowerForStat(1, 200),
                   GetAttackPowerForStat(2, 200),
                   HasAPEffectsSpellPower()
            "#,
        )
        .unwrap();
    assert_eq!(str_ap, 0.0);
    assert_eq!(agi_ap, 200.0);
    assert!(ap_to_sp);
}

#[test]
fn paperdoll_helpers_disable_ap_to_sp_on_intellect_spec() {
    let env = env();
    {
        let mut st = env.state().borrow_mut();
        st.player.class_index = 2; // Paladin
        st.player.active_spec_index = 1; // Holy (Intellect)
    }
    let (str_ap, int_ap, ap_to_sp, sp_to_ap): (f64, f64, bool, bool) = env
        .eval(
            r#"
            return GetAttackPowerForStat(1, 200),
                   GetAttackPowerForStat(4, 200),
                   HasAPEffectsSpellPower(),
                   HasSPEffectsAttackPower()
            "#,
        )
        .unwrap();
    assert_eq!(str_ap, 0.0);
    assert_eq!(int_ap, 0.0);
    assert!(!ap_to_sp);
    assert!(!sp_to_ap);
}

// ── Unknown unit tokens ───────────────────────────────────────────────────────

#[test]
fn stats_for_unknown_unit_fall_back_to_zero() {
    let env = env();
    let (armor, ap, damage_min): (i32, i32, f64) = env
        .eval(
            r#"
            local _, armor = UnitArmor("unknown_unit")
            local ap = UnitAttackPower("unknown_unit")
            local min_dmg = UnitDamage("unknown_unit")
            return armor, ap, min_dmg
            "#,
        )
        .unwrap();
    assert_eq!(armor, 0);
    assert_eq!(ap, 0);
    assert_eq!(damage_min, 0.0);
}

#[test]
fn paper_doll_defensive_chance_globals_return_numbers() {
    let env = env();
    let (dodge, parry, block, shield_block): (f64, f64, f64, f64) = env
        .eval(r#"return GetDodgeChance(), GetParryChance(), GetBlockChance(), GetShieldBlock()"#)
        .unwrap();

    assert!(dodge >= 0.0, "dodge={dodge}");
    assert!(parry >= 0.0, "parry={parry}");
    assert!(block >= 0.0, "block={block}");
    assert!(shield_block >= 0.0, "shield block={shield_block}");
}
