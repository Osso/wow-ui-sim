//! Tests for character stats: base + gear computation, stat API queries.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn combat_rating_value_converts_with_crit_divisor() {
    let env = env();

    let bonus: f64 = env
        .eval("return GetCombatRatingBonusForCombatRatingValue(9, 360)")
        .unwrap();

    assert_eq!(bonus, 2.0);
}

// ============================================================================
// UnitStat
// ============================================================================

#[test]
fn test_unit_stat_strength_positive_with_default_gear() {
    let env = env();
    let str_val: f64 = env
        .eval("local b, e = UnitStat('player', 1); return b")
        .unwrap();
    assert!(
        str_val > 100.0,
        "strength={str_val}, expected > 100 with gear"
    );
}

#[test]
fn test_unit_stat_stamina_high_with_gear() {
    let env = env();
    let sta: f64 = env
        .eval("local b, e = UnitStat('player', 3); return b")
        .unwrap();
    assert!(
        sta > 1000.0,
        "stamina={sta}, expected > 1000 with full gear"
    );
}

#[test]
fn test_unit_stat_strength_is_primary_for_paladin() {
    let env = env();
    let (str_val, agi, int): (f64, f64, f64) = env
        .eval(
            "local s = UnitStat('player', 1); \
             local a = UnitStat('player', 2); \
             local i = UnitStat('player', 4); \
             return s, a, i",
        )
        .unwrap();
    assert!(
        str_val > agi,
        "Paladin: str={str_val} should be > agi={agi}"
    );
    assert!(
        str_val > int,
        "Paladin: str={str_val} should be > int={int}"
    );
}

#[test]
fn test_unit_stat_returns_four_values() {
    let env = env();
    let (base, eff, pos, neg): (f64, f64, f64, f64) =
        env.eval("return UnitStat('player', 1)").unwrap();
    assert_eq!(base, eff);
    assert_eq!(pos, 0.0);
    assert_eq!(neg, 0.0);
}

// ============================================================================
// Combat Ratings
// ============================================================================

#[test]
fn test_combat_rating_crit_positive() {
    let env = env();
    let crit: i32 = env.eval("return GetCombatRating(9)").unwrap();
    assert!(crit > 0, "crit rating={crit}, expected > 0 with gear");
}

#[test]
fn test_combat_rating_haste_positive() {
    let env = env();
    let haste: i32 = env.eval("return GetCombatRating(6)").unwrap();
    assert!(haste > 0, "haste rating={haste}");
}

#[test]
fn test_combat_rating_mastery_positive() {
    let env = env();
    let mastery: i32 = env.eval("return GetCombatRating(26)").unwrap();
    assert!(mastery > 0, "mastery rating={mastery}");
}

#[test]
fn test_combat_rating_versatility_positive() {
    let env = env();
    let vers: i32 = env.eval("return GetCombatRating(14)").unwrap();
    assert!(vers > 0, "vers rating={vers}");
}

#[test]
fn test_combat_rating_bonus_is_percentage() {
    let env = env();
    let crit_pct: f64 = env.eval("return GetCombatRatingBonus(9)").unwrap();
    assert!(crit_pct > 0.0, "crit %={crit_pct}");
    assert!(crit_pct < 100.0, "crit % should be reasonable: {crit_pct}");
}

// ============================================================================
// Secondary Stats
// ============================================================================

#[test]
fn test_get_crit_chance_includes_base() {
    let env = env();
    let crit: f64 = env.eval("return GetCritChance()").unwrap();
    // 5% base + rating bonus
    assert!(crit > 5.0, "crit chance={crit}, expected > 5%");
}

#[test]
fn test_get_haste_positive() {
    let env = env();
    let haste: f64 = env.eval("return GetHaste()").unwrap();
    assert!(haste > 0.0, "haste={haste}");
}

#[test]
fn test_get_mastery_effect_two_values() {
    let env = env();
    let (total, from_rating): (f64, f64) = env.eval("return GetMasteryEffect()").unwrap();
    assert!(total > 0.0, "total mastery={total}");
    assert!(from_rating > 0.0, "mastery from rating={from_rating}");
    assert!(
        total > from_rating,
        "total={total} should include base mastery"
    );
}

#[test]
fn test_get_versatility_bonus_positive() {
    let env = env();
    let vers: f64 = env.eval("return GetVersatilityBonus(1)").unwrap();
    assert!(vers > 0.0, "vers bonus={vers}");
}

#[test]
fn test_paper_doll_combat_stat_helpers_return_safe_numbers() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local base, combat = GetManaRegen()
            local values = {
                GetSpellCritChance(2),
                GetSpellCritChance(),
                GetRangedCritChance(),
                GetCombatRatingBonusForCombatRatingValue(CR_CRIT_MELEE or 9, 180),
                GetSpeed(),
                GetLifesteal(),
                GetAvoidance(),
                base,
                combat,
                GetCritChanceFromAgility("player"),
                GetSpellCritChanceFromIntellect("player"),
                GetUnitManaRegenRateFromSpirit("player"),
                GetMeleeHaste(),
                GetRangedHaste(),
                GetHitModifier(),
                GetSpellHitModifier(),
                (GetExpertise()),
                (GetExpertisePercent()),
                GetModResilienceDamageReduction(),
                GetPvpPowerDamage(),
                GetPvpPowerHealing(),
                GetMeleeMissChance(),
                GetRangedMissChance(),
                GetSpellMissChance(),
                GetEnemyDodgeChance(),
                GetEnemyParryChance(),
            }
            for _, value in ipairs(values) do
                if type(value) ~= "number" or value < 0 then
                    return false
                end
            end
            return GetCritChanceProvidesParryEffect() == false
            "#,
        )
        .unwrap();

    assert!(
        ok,
        "PaperDoll combat stat helpers should return safe numeric values"
    );
}

// ============================================================================
// Stats change with equipment
// ============================================================================

#[test]
fn test_unequip_reduces_stats() {
    let env = env();
    let before: f64 = env.eval("return UnitStat('player', 1)").unwrap();
    env.exec("A_Admin.UnequipItem(1)").unwrap(); // Remove helm
    let after: f64 = env.eval("return UnitStat('player', 1)").unwrap();
    assert!(
        after < before,
        "str after unequip={after} should be < before={before}"
    );
}

#[test]
fn test_equip_increases_stats() {
    let env = env();
    env.exec("A_Admin.UnequipItem(1)").unwrap();
    let before: f64 = env.eval("return UnitStat('player', 1)").unwrap();
    env.exec("A_Admin.EquipItem(1, 211993)").unwrap(); // Re-equip helm
    let after: f64 = env.eval("return UnitStat('player', 1)").unwrap();
    assert!(
        after > before,
        "str after equip={after} should be > before={before}"
    );
}

// ============================================================================
// Stats with no gear (via Lua API)
// ============================================================================

#[test]
fn test_stats_drop_to_base_with_no_gear() {
    let env = env();
    // Unequip all slots
    env.exec(
        "for _, slot in ipairs({1,2,3,5,6,7,8,9,10,11,12,13,14,15,16}) do \
             A_Admin.UnequipItem(slot) \
         end",
    )
    .unwrap();
    let str_val: f64 = env.eval("return UnitStat('player', 1)").unwrap();
    let crit: i32 = env.eval("return GetCombatRating(9)").unwrap();
    // Base stats only, no gear
    assert!(str_val < 200.0, "base str without gear={str_val}");
    assert_eq!(crit, 0, "no gear = no crit rating");
}
